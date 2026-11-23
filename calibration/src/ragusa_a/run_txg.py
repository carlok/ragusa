"""Gate G-A1: reproduce PROMPT.md section 1.1 with the shipped txgraffiti 0.4.1 code.

Mirrors `txgraffiti.systems.txgraffiti2` stage by stage, calling the shipped
generators and filters unmodified, so each stage's input and output can be counted.
Shipped filters return lists, so aliases cannot be told apart from discards:
every removal is recorded as `n_discarded`, and `n_alias` is 0.

    uv run python -m ragusa_a.run_txg                                    # section 1.1 scope
    uv run python -m ragusa_a.run_txg --generators convex_hull,ratios,linear_programming
"""

from __future__ import annotations

import argparse
import importlib.resources
import time

import txgraffiti
from txgraffiti.example_data import graph_data
from txgraffiti.logic import Property
from txgraffiti.processing import auto_wrap, filter_with_dalmatian, filter_with_morgan, remove_duplicates

from ragusa_a import pin
from ragusa_a.pool import generate, setup_solver

EXPECTED_1_1 = {"generated": 453, "remove_duplicates": 384, "filter_with_dalmatian": 216, "filter_with_morgan": 141}


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--target", default="independence_number")
    ap.add_argument("--generators", default="convex_hull,ratios")
    ap.add_argument("--n-features", type=int, default=12)
    ap.add_argument("--n-preds", type=int, default=6)
    ap.add_argument("--feature-size", type=int, default=1)
    args = ap.parse_args()

    df = graph_data
    table_bytes = importlib.resources.files("txgraffiti.example_data").joinpath("graph_data.csv").read_bytes()
    gen_names = args.generators.split(",")
    solver_note = setup_solver(gen_names)

    numeric_props, bool_preds = auto_wrap(df)
    target = Property(args.target, lambda d, c=args.target: d[c])
    features = [p for p in numeric_props if p.name != target.name][: args.n_features]
    preds = bool_preds[: args.n_preds]

    t0 = time.perf_counter()
    raw, per_gen = generate(df, target, features, preds, gen_names, args.feature_size)
    stages = [("generated", raw, time.perf_counter() - t0)]
    print(f"candidates per generator: {per_gen}")

    current = raw
    for fn in (remove_duplicates, filter_with_dalmatian, filter_with_morgan):
        t = time.perf_counter()
        current = fn(current, df)
        stages.append((fn.__name__, current, time.perf_counter() - t))

    rid = pin.run_id("txgraffiti")
    order = ">".join(name for name, _, _ in stages[1:])
    common = dict(
        run_id=rid, system="txgraffiti", system_version=txgraffiti.__version__,
        table="graph_data", table_sha256=pin.sha256_bytes(table_bytes),
        feature_subset=[p.name for p in features], predicate_subset=[p.name for p in preds],
        target=args.target, generator="+".join(gen_names), order=order, n_alias=0, seed=None,
    )
    rows = []
    prev = None
    for i, (name, kept, wall) in enumerate(stages):
        n_in = len(kept) if prev is None else prev
        rows.append(dict(
            common, filter=name, filter_kind="generation" if i == 0 else "redundancy", stage=i,
            n_in=n_in, n_representative=len(kept), n_discarded=n_in - len(kept), wall_s=round(wall, 3),
            notes=solver_note if i == 0 else "shipped filter returns a list; aliases not distinguishable from discards",
        ))
        prev = len(kept)

    tag = "-".join(gen_names)
    path = pin.write_rows(f"txg_{args.target}_{tag}_f{args.n_features}_p{args.n_preds}", rows)
    survivors = path.with_suffix(".survivors.txt")
    survivors.write_text("".join(f"{name}\n" for name in sorted(c.name for c in current)))

    print(f"rows -> {path}")
    print(f"survivors -> {survivors}")
    print(f"{'stage':<24}{'n_in':>6}{'kept':>6}{'killed':>8}{'wall_s':>9}")
    for r in rows:
        print(f"{r['filter']:<24}{r['n_in']:>6}{r['n_representative']:>6}{r['n_discarded']:>8}{r['wall_s']:>9.2f}")

    in_scope = (args.target == "independence_number" and gen_names == ["convex_hull", "ratios"]
                and args.n_features == 12 and args.n_preds == 6 and args.feature_size == 1)
    if in_scope:
        got = {r["filter"]: r["n_representative"] for r in rows}
        ok = got == EXPECTED_1_1
        print(f"G-A1 {'PASS' if ok else 'FAIL'}: got {got}, expected {EXPECTED_1_1}")


if __name__ == "__main__":
    main()
