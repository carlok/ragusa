"""Export KnotInfo's spreadsheet to ragusa CSV, with manifests (Sprint 7, phase A).

Every output stays in the git-ignored `external/knotinfo/export/`, permanently. KnotInfo carries no
copyright, but its maintainers asked that the database not be reposted, since a copy goes out of date
and readers take it for a mirror (answer of 2026-10-02, recorded in CONTROLS.md). Download it from
https://knotinfo.org and cite it; the manifests record the sha256 of the file each export came from.

Cell rules, fixed from the Sprint 7 census of every column:

- point: a decimal such as `3`, `1.0`, `-.0467543`;
- interval: `[a,b]`, `[a, b]` or `[a; b]` with a <= b, and `a or b`;
- missing: empty, `Not Hyperbolic`, or any other text;
- boolean: `Y`/`Yes` true, `N`/`No` false, anything else (e.g. `Unknown`) missing.

Columns are an explicit list. Excluded on purpose: `name_rank` and `dt_rank` (row identifiers),
`nu` (a pair of values, not an interval: 1390 cells have a > b), `second_alexander_polynomial`
(a polynomial).

Tables: `knotinfo_le12` (prime knots with 3 to 12 crossings) and `knotinfo_le13` (3 to 13), and
`knotinfo_le13_mirror` (Sprint 12): the mirror image of every knot of `knotinfo_le13`. KnotInfo
stores one chirality per knot, so a statement about a class closed under mirroring must also hold on
the mirrors. In the mirror table:

- `SIGNED` columns are negated (signature, Rasmussen s, Ozsváth–Szabó τ, Hom's ε all change sign
  under mirroring);
- `UNKNOWN_IN_MIRROR` columns are missing (the Chern–Simons invariant and the imaginary part of the
  complex volume change sign modulo a normalisation this exporter does not reproduce);
- `CHIRAL` properties are unknown (a knot's mirror need not be positive, quasipositive, and so on);
- every other column is copied: the other invariants do not depend on chirality.

Names get the suffix `_mirror`.

    uv run python -m ragusa_a.export_knotinfo
"""

from __future__ import annotations

import re

import xlrd

from ragusa_a import pin
from ragusa_a.ragusa_csv import interval, point, write_table

SOURCE_DIR = pin.REPO / "external" / "knotinfo"
XLS = SOURCE_DIR / "knotinfo_data_complete.xls"
OUT = SOURCE_DIR / "export"

NUMERIC = [
    "crossing_number", "unknotting_number", "three_genus", "crosscap_number", "bridge_index",
    "braid_index", "braid_length", "signature", "nakanishi_index", "super_bridge_index", "arc_index",
    "polygon_index", "tunnel_number", "morse_novikov_number", "smooth_four_genus",
    "topological_four_genus", "smooth_4d_crosscap_number", "topological_4d_crosscap_number",
    "smooth_concordance_genus", "determinant", "rasmussen_invariant", "ozsvath_szabo_tau_invariant",
    "volume", "maximum_cusp_volume", "longitude_length", "meridian_length", "chern_simons_invariant",
    "volume_imaginary_part", "arf_invariant", "turaev_genus", "fd_clasp_number", "width",
    "td_clasp_number", "epsilon", "double_slice_genus", "unknotting_number_algebraic", "ropelength",
]
BOOLEAN = [
    "alternating", "fibered", "positive_braid", "positive", "strongly_quasipositive", "quasipositive",
    "l_space", "quasi_alternating", "almost_alternating", "adequate", "almost_strongly_qp",
    "cosmetic_crossing",
]

SIGNED = ["signature", "rasmussen_invariant", "ozsvath_szabo_tau_invariant", "epsilon"]
UNKNOWN_IN_MIRROR = ["chern_simons_invariant", "volume_imaginary_part"]
CHIRAL = ["positive_braid", "positive", "strongly_quasipositive", "quasipositive", "almost_strongly_qp",
          "l_space"]

NUMBER = r"-?(?:\d+(?:\.\d*)?|\.\d+)"
POINT = re.compile(NUMBER)
BRACKET = re.compile(rf"\[\s*({NUMBER})\s*[,;]\s*({NUMBER})\s*\]")
OR = re.compile(rf"({NUMBER})\s+or\s+({NUMBER})")


def parse_num(text: str, stats: dict):
    t = text.strip()
    if POINT.fullmatch(t):
        return point(float(t))
    m = BRACKET.fullmatch(t) or OR.fullmatch(t)
    if m:
        a, b = float(m.group(1)), float(m.group(2))
        if a > b:
            stats["bracket_lo_gt_hi"] += 1
            return None
        return interval(a, b)
    if t:
        stats["text_as_missing"] += 1
    return None


def negate(num):
    """The negated cell: a point stays a point, an interval [a, b] becomes [-b, -a]."""
    if num is None:
        return None
    lo, hi = -num.hi + 0.0, -num.lo + 0.0  # + 0.0 turns -0.0 into 0.0
    return point(lo) if lo == hi else interval(lo, hi)


def parse_bool(text: str):
    t = text.strip()
    if t in ("Y", "Yes"):
        return True
    if t in ("N", "No"):
        return False
    return None


def main() -> None:
    sheet = xlrd.open_workbook(str(XLS)).sheet_by_index(0)
    header = [str(c.value) for c in sheet.row(0)]
    missing_cols = [c for c in ["name", *NUMERIC, *BOOLEAN] if c not in header]
    if missing_cols:
        raise SystemExit(f"columns not in spreadsheet: {missing_cols}")
    idx = {c: header.index(c) for c in ["name", *NUMERIC, *BOOLEAN]}
    rows = range(2, sheet.nrows)  # row 0 keys, row 1 display names

    for table, max_crossings in (("knotinfo_le12", 12), ("knotinfo_le13", 13)):
        keep = []
        for i in rows:
            c = str(sheet.cell_value(i, idx["crossing_number"])).strip()
            if POINT.fullmatch(c) and 3 <= float(c) <= max_crossings:
                keep.append(i)
        stats = {"bracket_lo_gt_hi": 0, "text_as_missing": 0}
        names = [str(sheet.cell_value(i, idx["name"])).strip() for i in keep]
        numeric = {c: [parse_num(str(sheet.cell_value(i, idx[c])), stats) for i in keep] for c in NUMERIC}
        boolean = {c: [parse_bool(str(sheet.cell_value(i, idx[c]))) for i in keep] for c in BOOLEAN}
        manifest = write_table(
            OUT / f"{table}.csv", names, numeric, boolean,
            source={
                "dataset": "KnotInfo (knotinfo.org), knotinfo_data_complete.xls",
                "xls_sha256": pin.sha256_file(XLS),
                "rows": f"prime knots with 3 to {max_crossings} crossings",
                "exporter": "calibration/src/ragusa_a/export_knotinfo.py",
                "parse_stats": stats,
            },
        )
        n_iv = sum(col.get("n_interval", 0) for col in manifest["columns"])
        n_miss = sum(col["n_missing"] for col in manifest["columns"])
        print(f"{table}: {manifest['n_objects']} knots, {len(NUMERIC)} numeric + {len(BOOLEAN)} boolean columns, "
              f"{n_iv} interval cells, {n_miss} missing cells, parse {stats} -> {OUT / (table + '.csv')}")

        if table == "knotinfo_le13":
            mirror_numeric = {
                c: ([negate(v) for v in values] if c in SIGNED
                    else [None] * len(values) if c in UNKNOWN_IN_MIRROR else values)
                for c, values in numeric.items()
            }
            mirror_boolean = {c: ([None] * len(values) if c in CHIRAL else values)
                              for c, values in boolean.items()}
            mirror = write_table(
                OUT / "knotinfo_le13_mirror.csv", [f"{nm}_mirror" for nm in names], mirror_numeric,
                mirror_boolean,
                source={
                    "dataset": "KnotInfo (knotinfo.org), knotinfo_data_complete.xls",
                    "xls_sha256": pin.sha256_file(XLS),
                    "rows": "mirror images of the prime knots with 3 to 13 crossings",
                    "exporter": "calibration/src/ragusa_a/export_knotinfo.py",
                    "mirror": {"negated": SIGNED, "missing": UNKNOWN_IN_MIRROR, "unknown_properties": CHIRAL},
                },
            )
            print(f"knotinfo_le13_mirror: {mirror['n_objects']} knots -> {OUT / 'knotinfo_le13_mirror.csv'}")


if __name__ == "__main__":
    main()
