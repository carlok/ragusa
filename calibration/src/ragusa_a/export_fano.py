"""Export the toric canonical Fano 3-fold classification to ragusa CSV, with a manifest.

Source: Kasprzyk, Zenodo record 5866330 (CC0), imported into `external/fano3c/toricf3c.db` by the
Sprint 6 load test. Output: `external/fano3c/export/fano3c.csv` (git-ignored; the data is CC0, but
a 674,688-row export does not belong in the repository).

Columns: every integer invariant; `degree` kept as exact fraction text; `ehrhart_a1`, `ehrhart_a2`
from the Ehrhart delta-vector (1, a1, a2, 1); the six booleans. The list columns (vertices,
points, dual vertices, normal form) and the variable-length `hilbert_delta` are left out.

    uv run python -m ragusa_a.export_fano
"""

from __future__ import annotations

import json
import sqlite3
from fractions import Fraction

from ragusa_a import pin
from ragusa_a.ragusa_csv import point, write_table

DIR = pin.REPO / "external" / "fano3c"
DB = DIR / "toricf3c.db"
OUT = DIR / "export" / "fano3c.csv"

INTEGER = [
    "num_vertices", "num_faces", "num_points", "volume", "gorenstein_index", "picard_rank",
    "automorphism_order", *(f"h{i}" for i in range(1, 11)), *(f"e{i}" for i in range(1, 11)),
]
BOOLEAN = ["is_terminal", "is_simplicial", "is_regular", "is_reflexive", "is_barycentre_zero", "is_dual_barycentre_zero"]


def main() -> None:
    if not DB.exists():
        raise SystemExit(f"{DB} missing: run the Sprint 6 load test first (imports toricf3c.sql)")
    con = sqlite3.connect(DB)
    cols = ["id", *INTEGER, "degree", "ehrhart_delta", *BOOLEAN]
    rows = con.execute(f"select {', '.join(cols)} from toricf3c order by id").fetchall()

    names = [str(r[0]) for r in rows]
    numeric = {c: [point(r[1 + k]) for r in rows] for k, c in enumerate(INTEGER)}
    base = 1 + len(INTEGER)
    numeric["degree"] = [point(Fraction(r[base]), text=r[base]) for r in rows]
    deltas = [json.loads(r[base + 1]) for r in rows]
    if any(len(d) != 4 or d[0] != 1 or d[3] != 1 for d in deltas):
        raise SystemExit("an ehrhart_delta is not of the documented form (1, a1, a2, 1)")
    numeric["ehrhart_a1"] = [point(d[1]) for d in deltas]
    numeric["ehrhart_a2"] = [point(d[2]) for d in deltas]
    boolean = {c: [bool(r[base + 2 + k]) for r in rows] for k, c in enumerate(BOOLEAN)}

    manifest = write_table(
        OUT, names, numeric, boolean,
        source={
            "dataset": "Kasprzyk, The classification of toric canonical Fano 3-folds, doi:10.5281/zenodo.5866330 (CC0)",
            "sql_md5": "d38e5d87917a3444a8b5a877e1868f00",
            "exporter": "calibration/src/ragusa_a/export_fano.py",
        },
    )
    print(f"fano3c: {manifest['n_objects']} rows, {len(numeric)} numeric + {len(boolean)} boolean columns -> {OUT}")


if __name__ == "__main__":
    main()
