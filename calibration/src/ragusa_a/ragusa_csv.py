"""Writer for ragusa CSV, the neutral table format the engine loads, plus its manifest.

Format (one file per table):

- Header: `name`, then one field per column as `<column>:num` or `<column>:bool`.
- One line per object; the first field is the object's name. No quoting: a field containing a
  comma, a quote or a newline is rejected when writing.
- `num` cells: empty (missing); a point value (decimal, or an exact fraction `p/q`); or a closed
  interval `lo..hi` with lo <= hi, each side a decimal or fraction.
- `bool` cells: `1`, `0`, or empty (missing).

The manifest records, per column, how many cells are points, intervals or missing, and sha256 of
the lower and upper values as little-endian float64 with NaN written as 0x7ff8000000000000 (a
point has lower = upper). Booleans hash one byte per object: 0, 1, or 2 for missing. Gate G-B6
requires the engine to reproduce every hash.
"""

from __future__ import annotations

import hashlib
import json
import pathlib
from dataclasses import dataclass
from fractions import Fraction

import numpy as np

CANONICAL_NAN = np.frombuffer(np.uint64(0x7FF8000000000000).tobytes(), dtype="<f8")[0]


@dataclass(frozen=True)
class Num:
    """A numeric cell: point when lo == hi. `text_lo`/`text_hi` keep exact source text (e.g. fractions)."""

    lo: float
    hi: float
    text_lo: str | None = None
    text_hi: str | None = None


def point(value: float | Fraction, text: str | None = None) -> Num:
    v = float(value)
    return Num(v, v, text, text)


def interval(lo: float, hi: float) -> Num:
    if lo > hi:
        raise ValueError(f"interval with lo > hi: {lo}..{hi}")
    return Num(float(lo), float(hi))


def _fmt(v: float, text: str | None) -> str:
    return text if text is not None else repr(v)


def _cell_num(cell: Num | None) -> str:
    if cell is None:
        return ""
    if cell.lo == cell.hi and cell.text_lo == cell.text_hi:
        return _fmt(cell.lo, cell.text_lo)
    return f"{_fmt(cell.lo, cell.text_lo)}..{_fmt(cell.hi, cell.text_hi)}"


def _hash_f8(values: list[float]) -> str:
    a = np.array(values, dtype="<f8")
    a[np.isnan(a)] = CANONICAL_NAN
    return hashlib.sha256(a.tobytes()).hexdigest()


def write_table(
    path: pathlib.Path,
    names: list[str],
    numeric: dict[str, list[Num | None]],
    boolean: dict[str, list[bool | None]],
    source: dict,
) -> dict:
    """Write `path` (CSV) and `path.with_suffix('.manifest.json')`; return the manifest."""
    n = len(names)
    for col, cells in [*numeric.items(), *boolean.items()]:
        if len(cells) != n:
            raise ValueError(f"column {col}: {len(cells)} cells for {n} objects")
    for field in [*names, *numeric, *boolean]:
        if any(ch in field for ch in ',"\n\r'):
            raise ValueError(f"field needs quoting, which ragusa CSV does not allow: {field!r}")

    path.parent.mkdir(parents=True, exist_ok=True)
    header = ["name", *(f"{c}:num" for c in numeric), *(f"{c}:bool" for c in boolean)]
    num_cols = list(numeric.values())
    bool_cols = list(boolean.values())
    with path.open("w", newline="\n") as fh:
        fh.write(",".join(header) + "\n")
        for r, name in enumerate(names):
            fields = [name]
            fields += [_cell_num(col[r]) for col in num_cols]
            fields += ["" if col[r] is None else ("1" if col[r] else "0") for col in bool_cols]
            fh.write(",".join(fields) + "\n")

    columns = []
    for col, cells in numeric.items():
        lo = [np.nan if c is None else c.lo for c in cells]
        hi = [np.nan if c is None else c.hi for c in cells]
        columns.append({
            "name": col, "kind": "num",
            "n_point": sum(1 for c in cells if c is not None and c.lo == c.hi),
            "n_interval": sum(1 for c in cells if c is not None and c.lo != c.hi),
            "n_missing": sum(1 for c in cells if c is None),
            "sha256_lower": _hash_f8(lo), "sha256_upper": _hash_f8(hi),
        })
    for col, cells in boolean.items():
        codes = bytes(2 if c is None else int(c) for c in cells)
        columns.append({
            "name": col, "kind": "bool",
            "n_true": sum(1 for c in cells if c is True),
            "n_false": sum(1 for c in cells if c is False),
            "n_missing": sum(1 for c in cells if c is None),
            "sha256": hashlib.sha256(codes).hexdigest(),
        })
    manifest = {
        "format": "ragusa-csv/1",
        "file": path.name,
        "file_sha256": hashlib.sha256(path.read_bytes()).hexdigest(),
        "n_objects": n,
        "names_sha256": hashlib.sha256("\n".join(names).encode()).hexdigest(),
        "source": source,
        "columns": columns,
    }
    path.with_suffix(".manifest.json").write_text(json.dumps(manifest, indent=1) + "\n")
    return manifest
