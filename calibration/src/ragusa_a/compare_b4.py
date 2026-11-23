"""Gate G-B4: ragusa's filter partitions against Project A's shipped-rule partitions on graph_data.

Agreement is reported, not required. ragusa's rules are its own (order-independent, same-direction
comparisons, tolerance-based validity) and are not claimed to equal txgraffiti's. Candidates are
joined by canonical bound text; txgraffiti's names are parsed with the Sprint 3 name parser and
written the way ragusa's `Bound` prints itself.

    uv run python -m ragusa_a.compare_b4
"""

from __future__ import annotations

import collections
import csv
import json

from txgraffiti.example_data import graph_data
from txgraffiti.processing import auto_wrap

from ragusa_a import pin
from ragusa_a.export_generator_fixtures import linear

TARGET = "independence_number"
A_DECISIONS = pin.REPO / "results" / "rows" / f"census_graph_data_{TARGET}.decisions.csv"
# Dalmatian is compared with sharp coverage, its counterpart, which since Sprint 10 is no longer
# ragusa's default redundancy stage: `make reproduce-b` writes this report with it explicitly.
B_DIR = pin.REPO / "results" / "floor" / f"graph_data_{TARGET}_sharp_coverage"
OUT = pin.REPO / "results" / "rows" / f"gb4_graph_data_{TARGET}.json"


def canonical(name: str, hypothesis: str, op: str, features: set[str]) -> str:
    body = name.split(" → ", 1)[1][1:-1]
    prefix = f"{TARGET} {op} "
    assert body.startswith(prefix), name
    coefs, constant = linear(body[len(prefix):], features)
    terms = [f"{v}·{f}" for f, v in sorted(coefs.items())]
    if constant != 0 or not terms:
        terms.append(str(constant))
    return f"({hypothesis}) → ({TARGET} {op} {' + '.join(terms)})"


def confusion(pairs) -> dict:
    c = collections.Counter(pairs)
    return {"both": c[(True, True)], "ragusa_only": c[(True, False)],
            "txgraffiti_only": c[(False, True)], "neither": c[(False, False)]}


def main() -> None:
    numeric, _ = auto_wrap(graph_data)
    features = {p.name for p in numeric}
    a_rows = list(csv.DictReader(A_DECISIONS.open()))
    b_rows = list(csv.DictReader((B_DIR / "candidates.csv").open()))
    report = json.loads((B_DIR / "floor.json").read_text())

    a_by, b_by = collections.defaultdict(list), collections.defaultdict(list)
    for r in a_rows:
        a_by[canonical(r["name"], r["hypothesis"], r["op"], features)].append(r)
    for r in b_rows:
        b_by[r["bound"]].append(r)
    matched = sorted(set(a_by) & set(b_by))

    def b_kept(row, stage):
        return row[f"{stage}_class"] == "representative"

    reached = [k for k in matched if b_by[k][0]["sharp_coverage_class"]]
    result = {
        "gate": "G-B4",
        "table_sha256_a": None,
        "ragusa_pin": report["pin"],
        "ragusa_run": report["run"],
        "pools": {
            "txgraffiti_candidates": len(a_rows),
            "txgraffiti_distinct_bounds": len(a_by),
            "ragusa_candidates": len(b_rows),
            "ragusa_distinct_bounds": len(b_by),
            "distinct_bounds_in_both": len(matched),
            "only_txgraffiti": len(set(a_by) - set(b_by)),
            "only_ragusa": len(set(b_by) - set(a_by)),
        },
        "validity_vs_is_true": confusion(
            (b_kept(b_by[k][0], "validity"), a_by[k][0]["is_false"] == "False") for k in matched),
        "sharp_coverage_vs_dalmatian": confusion(
            (b_kept(b_by[k][0], "sharp_coverage"), a_by[k][0]["dalmatian_class"] == "representative") for k in reached),
        "generality_vs_morgan": confusion(
            (b_kept(b_by[k][0], "generality"), a_by[k][0]["morgan_class"] == "representative")
            for k in matched if b_by[k][0]["generality_class"]),
        "survivors_vs_pipeline": confusion(
            (b_kept(b_by[k][0], "generality"), a_by[k][0]["pipeline_survivor"] == "True") for k in matched),
        "notes": ("confusions are over distinct bounds present in both pools; sharp_coverage and generality "
                  "only over bounds that reached that ragusa stage; txgraffiti's rules each ran on its whole pool"),
    }
    census = json.loads((pin.REPO / "results" / "rows" / f"census_graph_data_{TARGET}.metrics.json").read_text())
    result["table_sha256_a"] = census["pinning"]["table_sha256"]
    csv_sha = next(s["sha256"] for s in report["pin"]["sources"])
    result["same_source_csv"] = csv_sha == result["table_sha256_a"]

    OUT.write_text(json.dumps(result, indent=1, ensure_ascii=False) + "\n")
    print(f"G-B4 -> {OUT}")
    print(f"same source CSV: {result['same_source_csv']}")
    for key in ("pools", "validity_vs_is_true", "sharp_coverage_vs_dalmatian", "generality_vs_morgan",
                "survivors_vs_pipeline"):
        print(f"{key}: {result[key]}")


if __name__ == "__main__":
    main()
