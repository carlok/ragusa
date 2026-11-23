"""Reference manifest for gate G-B1: how pandas + txgraffiti type the six shipped tables.

For each table, as loaded by `txgraffiti.example_data` and wrapped by `auto_wrap`:
row count, CSV sha256, boolean and numeric column names in order, skipped columns,
and per column a value hash the Rust loader must reproduce:

- numeric: sha256 of the float64 values, little-endian, NaN as 0x7ff8000000000000;
- boolean: sha256 of one byte per row (0 or 1).

    uv run python -m ragusa_a.export_manifest      # writes engine/fixtures/txgraffiti_manifest.json
"""

from __future__ import annotations

import hashlib
import importlib.resources
import json

import numpy as np
import pandas as pd
import txgraffiti
from txgraffiti import example_data
from txgraffiti.processing import auto_wrap

from ragusa_a import pin

TABLES = {
    "graph_data": "graph_data.csv",
    "integer_data": "integer_data.csv",
    "polytope_data": "polytope_data.csv",
    "qubits_data": "Nqubits_data.csv",
    "calabi_yau_data": "calabi_yau_data.csv",
    "nba_game_data": "nba_game_data.csv",
}
CANONICAL_NAN = np.frombuffer(np.uint64(0x7FF8000000000000).tobytes(), dtype="<f8")[0]
OUT = pin.REPO / "engine" / "fixtures" / "txgraffiti_manifest.json"


def _f8(s: pd.Series) -> np.ndarray:
    return s.to_numpy(dtype="<f8", na_value=np.nan).copy()


def numeric_hash(v: np.ndarray) -> str:
    v = v.copy()
    v[np.isnan(v)] = CANONICAL_NAN
    return hashlib.sha256(v.tobytes()).hexdigest()


def numeric_entry(name: str, default: pd.Series, round_trip: pd.Series) -> dict:
    """pandas' default float parser is not always correctly rounded; `round_trip` is
    (it equals Python's float()). Record both, and how far apart they are."""
    a, b = _f8(default), _f8(round_trip)
    both = ~np.isnan(a) & ~np.isnan(b)
    differs = both & (a != b)
    ulps = np.abs(a[differs] - b[differs]) / np.spacing(np.abs(b[differs])) if differs.any() else np.array([0.0])
    return {
        "name": name,
        "n_nan": int(np.isnan(a).sum()),
        "sha256": numeric_hash(a),
        "sha256_round_trip": numeric_hash(b),
        "n_default_parse_differs": int(differs.sum()),
        "max_default_parse_ulps": int(round(float(ulps.max()))),
    }


def bool_hash(s: pd.Series) -> str:
    return hashlib.sha256(s.to_numpy(dtype=np.uint8).tobytes()).hexdigest()


def main() -> None:
    base = importlib.resources.files("txgraffiti.example_data")
    tables = {}
    for name, filename in TABLES.items():
        df = getattr(example_data, name)
        round_trip = pd.read_csv(base.joinpath(filename), float_precision="round_trip")
        numeric, bools = auto_wrap(df)
        wrapped = {p.name for p in numeric} | {p.name for p in bools}
        tables[name] = {
            "csv": filename,
            "csv_sha256": pin.sha256_bytes(base.joinpath(filename).read_bytes()),
            "n_rows": len(df),
            "boolean": [{"name": p.name, "n_true": int(df[p.name].sum()), "sha256": bool_hash(df[p.name])}
                        for p in bools],
            "numeric": [numeric_entry(p.name, df[p.name], round_trip[p.name]) for p in numeric],
            "skipped": [c for c in df.columns if c not in wrapped],
        }
    manifest = {
        "generated_by": "calibration/src/ragusa_a/export_manifest.py",
        "txgraffiti": txgraffiti.__version__,
        "pandas": pd.__version__,
        "hash_rules": {"numeric": "sha256 of <f8 values, NaN = 0x7ff8000000000000; "
                                  "`sha256` = pandas default parser (what txgraffiti computes on), "
                                  "`sha256_round_trip` = float_precision='round_trip' (correctly rounded)",
                       "boolean": "sha256 of one u8 per row"},
        "tables": tables,
    }
    OUT.parent.mkdir(parents=True, exist_ok=True)
    OUT.write_text(json.dumps(manifest, indent=1) + "\n")
    for name, t in tables.items():
        dev = [c for c in t["numeric"] if c["n_default_parse_differs"]]
        print(f"{name}: rows={t['n_rows']} boolean={len(t['boolean'])} numeric={len(t['numeric'])} "
              f"skipped={len(t['skipped'])} default-parse deviations: {len(dev)} columns, "
              f"{sum(c['n_default_parse_differs'] for c in dev)} values, "
              f"max {max((c['max_default_parse_ulps'] for c in dev), default=0)} ulps")
    print(f"manifest -> {OUT}")


if __name__ == "__main__":
    main()
