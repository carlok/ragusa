"""Sprint 1 census on one TxGraffiti table: numbers 1-4 of PROMPT.md section 6.1.

    uv run python -m ragusa_a.census          # graph_data, independence_number, all features and predicates

1. partition per acceptance rule, each rule run separately on the same deduplicated pool;
2. object-database floor: leave-one-object-out decision flips per rule;
3. chance baseline: uniform random acceptance at each rule's acceptance count;
4. AutoGraphForge known-results verdicts over the same pool and over the pipeline survivors.

Writes to results/rows/: census_<table>_<target>.jsonl (partition rows),
.metrics.json (floor, chance, gates, pinning) and .decisions.csv (one line per candidate).
"""

from __future__ import annotations

import argparse
import csv
import importlib.resources
import json
import time

import numpy as np
import txgraffiti
from txgraffiti import example_data
from txgraffiti.processing import filter_with_morgan
from txgraffiti.systems import txgraffiti2

from ragusa_a import agf_txg, pin
from ragusa_a.analysis import chance, floor, partition
from ragusa_a.pool import DEFAULT_GENERATORS, GENERATORS, build_pool
from ragusa_a.rules import KERNELS, RULE_LABEL, RULES, shipped

ALIAS_NOTE = ("alias = rejected candidate identical on the table to an accepted one "
              "(same lhs, op, hypothesis rows, rhs values); defined by ragusa_a.analysis, not txgraffiti")


def _json_default(x):
    if isinstance(x, np.integer):
        return int(x)
    if isinstance(x, np.floating):
        return float(x)
    if isinstance(x, np.bool_):
        return bool(x)
    if isinstance(x, np.ndarray):
        return x.tolist()
    raise TypeError(type(x))


def log(msg):
    print(msg, flush=True)


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--table", default="graph_data")
    ap.add_argument("--target", default="independence_number")
    ap.add_argument("--generators", default=",".join(DEFAULT_GENERATORS))
    ap.add_argument("--n-features", type=int, default=None)
    ap.add_argument("--n-preds", type=int, default=None)
    ap.add_argument("--loo-check-rows", type=int, default=4, help="removals re-run through shipped code per rule")
    ap.add_argument("--chance-draws", type=int, default=1000)
    ap.add_argument("--seed", type=int, default=0)
    ap.add_argument("--skip-txgraffiti2-control", action="store_true")
    args = ap.parse_args()

    gen_names = args.generators.split(",")
    df = getattr(example_data, args.table)
    sha = pin.sha256_bytes(importlib.resources.files("txgraffiti.example_data")
                           .joinpath(f"{args.table}.csv").read_bytes())
    pool = build_pool(df, table=args.table, table_sha256=sha, target=args.target, gen_names=gen_names,
                      n_features=args.n_features, n_preds=args.n_preds)
    log(f"pool: raw={pool.n_raw} dedup={pool.M} objects={pool.N} per_gen={pool.per_gen} "
        f"gen_s={pool.gen_wall:.1f} false_in_pool={int(pool.is_false.sum())}")

    rid = pin.run_id("census")
    common = dict(run_id=rid, system="txgraffiti", system_version=txgraffiti.__version__, table=args.table,
                  table_sha256=sha, feature_subset=pool.features, predicate_subset=pool.preds,
                  target=args.target, generator="+".join(gen_names), seed=None)
    rows = [
        dict(common, filter="generated", filter_kind="generation", stage=0, order="-", n_in=pool.n_raw,
             n_representative=pool.n_raw, n_alias=0, n_discarded=0, wall_s=round(pool.gen_wall, 3),
             notes=pool.solver_note),
        dict(common, filter="remove_duplicates", filter_kind="redundancy", stage=1, order="generated>remove_duplicates",
             n_in=pool.n_raw, n_representative=pool.M, n_alias=0, n_discarded=pool.n_raw - pool.M,
             wall_s=round(pool.dedup_wall, 3), notes="shipped list filter keyed on (hypothesis.name, conclusion.name)"),
    ]
    metrics = {
        "run_id": rid,
        "pinning": {"txgraffiti": txgraffiti.__version__, "table": args.table, "table_sha256": sha,
                    "target": args.target, "generators": gen_names, "features": pool.features,
                    "predicates": pool.preds, "feature_size": 1, "seed": args.seed,
                    "chance_draws": args.chance_draws, "solver": pool.solver_note},
        "pool": {"n_raw": pool.n_raw, "n_dedup": pool.M, "n_objects": pool.N, "per_generator": pool.per_gen,
                 "n_false": int(pool.is_false.sum()), "n_table_classes": int(np.unique(pool.class_id).size),
                 "n_hypotheses": int(pool.Hu.shape[0])},
        "rules": {},
        "gates": {},
    }
    accepts, parts = {}, {}
    check_rng = np.random.default_rng(args.seed)

    # ---- numbers 1-3: each rule separately on the deduplicated pool -------------------
    for k, rule in enumerate(RULES):
        t0 = time.perf_counter()
        acc = shipped(rule, pool.conjs, pool.df)
        wall = time.perf_counter() - t0
        part = partition(pool, acc)
        accepts[rule], parts[rule] = acc, part
        log(f"[{rule}] shipped {wall:.1f}s: rep={part['n_representative']} alias={part['n_alias']} "
            f"discarded={part['n_discarded']}")
        rows.append(dict(common, filter=RULE_LABEL[rule], filter_kind="redundancy", stage=2,
                         order=f"remove_duplicates>{RULE_LABEL[rule]}", n_in=part["n_in"],
                         n_representative=part["n_representative"], n_alias=part["n_alias"],
                         n_discarded=part["n_discarded"], wall_s=round(wall, 3), notes=ALIAS_NOTE))
        rm = {"shipped_wall_s": round(wall, 3),
              "partition": {key: v for key, v in part.items() if key not in ("cls", "alias_of")}}

        # kernel must reproduce the shipped decisions before any floor number is used
        attempts, chosen = {}, None
        for exact in ((True, False) if rule == "dalmatian" else (True,)):
            full, loo, extra = KERNELS[rule](pool, exact=exact)
            mism = int((full != acc).sum())
            attempts["exact" if exact else "float"] = mism
            if mism == 0:
                chosen = (loo, extra, exact)
                break
        metrics["gates"][f"G-A3:{rule}"] = {"full_table_mismatches": attempts, "pass": chosen is not None}
        log(f"[{rule}] G-A3 kernel vs shipped on full table: {attempts}")

        if chosen:
            loo, extra, exact = chosen
            fl = floor(pool, acc, loo)
            per_obj = np.asarray(fl["per_object_flips"])
            top = [int(r) for r in np.argsort(-per_obj, kind="stable")[:2] if per_obj[r] > 0]
            rest = np.array([r for r in range(pool.N) if r not in top])
            picks = top + [int(r) for r in check_rng.choice(rest, size=max(args.loo_check_rows - len(top), 0),
                                                            replace=False)]
            checks = []
            for r in picks:
                t0 = time.perf_counter()
                got = shipped(rule, pool.conjs, pool.df.drop(index=pool.df.index[r]))
                checks.append({"row": r, "chosen_as": "most flips" if r in top else "random",
                               "mismatches": int((got != loo[:, r]).sum()),
                               "wall_s": round(time.perf_counter() - t0, 3)})
                log(f"[{rule}] G-A4 removal of row {r}: {checks[-1]['mismatches']} mismatches")
            metrics["gates"][f"G-A4:{rule}"] = {"checks": checks, "pass": all(c["mismatches"] == 0 for c in checks)}
            rm["kernel"] = "exact" if exact else "float"
            rm["floor"] = fl
            if rule == "dalmatian":
                strict, no_others = extra["strict"], extra["no_others"]
                nstrict = strict.sum(axis=1)
                rm["dalmatian"] = {
                    "accepted_without_competitors": int((acc & no_others).sum()),
                    "accepted_single_strict_row": int((acc & ~no_others & (nstrict == 1)).sum()),
                    "strict_witness_objects": int(strict[acc & ~no_others].any(axis=0).sum()),
                    "accepted_lower_bounds": int((acc & ~pool.upper).sum()),
                    "accepted_lower_bounds_without_competitors": int((acc & ~pool.upper & no_others).sum()),
                    "accepted_le_objects": bool(acc.sum() <= pool.N),
                }
                accepts["_dalmatian_nstrict"] = nstrict
        rm["chance"] = chance(pool, acc, np.random.default_rng([args.seed, k]), args.chance_draws)
        metrics["rules"][rule] = rm

    # ---- the documented pipeline order, for continuity with txgraffiti2 -----------------
    dal = accepts["dalmatian"]
    t0 = time.perf_counter()
    kept = filter_with_morgan([c for c, a in zip(pool.conjs, dal) if a], pool.df)
    wall = time.perf_counter() - t0
    pipe = np.zeros(pool.M, dtype=bool)
    pipe[[i for i, c in enumerate(pool.conjs) if any(c is x for x in kept)]] = True
    pipe_part = partition(pool, pipe, among=dal)
    rows.append(dict(common, filter=RULE_LABEL["morgan"], filter_kind="redundancy", stage=3,
                     order="remove_duplicates>filter_with_dalmatian>filter_with_morgan", n_in=pipe_part["n_in"],
                     n_representative=pipe_part["n_representative"], n_alias=pipe_part["n_alias"],
                     n_discarded=pipe_part["n_discarded"], wall_s=round(wall, 3), notes=ALIAS_NOTE))
    metrics["pipeline"] = {"n_after_dalmatian": int(dal.sum()), "n_after_morgan": int(pipe.sum())}
    log(f"[pipeline] dedup>dalmatian>morgan: {pool.M} -> {int(dal.sum())} -> {int(pipe.sum())}")

    if not args.skip_txgraffiti2_control and args.n_features is None and args.n_preds is None:
        t0 = time.perf_counter()
        out = txgraffiti2(pool.df, args.target, generators=tuple(GENERATORS[g] for g in gen_names))
        ok = len(out) == int(pipe.sum())
        metrics["gates"]["G-A5:txgraffiti2"] = {"txgraffiti2_count": len(out), "staged_count": int(pipe.sum()),
                                                "wall_s": round(time.perf_counter() - t0, 3), "pass": ok}
        log(f"[pipeline] G-A5 txgraffiti2() returns {len(out)} vs staged {int(pipe.sum())}")

    # ---- number 4: AutoGraphForge known-results filter (novelty kind) ------------------
    agf, commit = agf_txg.classify(pool.names, list(pool.df.columns))
    known = {label: np.array([v[0] for v in s["verdicts"]]) for label, s in agf.items()}
    metrics["agf"] = {"commit": commit, "sets": {}}
    for label, s in agf.items():
        kn = known[label]
        metrics["agf"]["sets"][label] = {
            "n_cols": s["n_cols"], "errors": s["errors"], "wall_s": s["wall_s"],
            "known_in_pool": int(kn.sum()), "known_in_pipeline_survivors": int((kn & pipe).sum()),
            "known_in_dalmatian_representatives": int((kn & dal).sum()),
        }
        for corpus, mask, stage, order in (("pool", np.ones(pool.M, dtype=bool), 2, "remove_duplicates>classify_statement"),
                                           ("pipeline survivors", pipe, 4,
                                            "remove_duplicates>filter_with_dalmatian>filter_with_morgan>classify_statement")):
            n_in, n_known = int(mask.sum()), int((kn & mask).sum())
            rows.append(dict(run_id=rid, system="autographforge", system_version=commit, table=args.table,
                             table_sha256=sha, target=args.target, generator="+".join(gen_names),
                             filter=f"classify_statement[{label}]", filter_kind="novelty", stage=stage, order=order,
                             n_in=n_in, n_representative=n_in - n_known, n_alias=0, n_discarded=n_known,
                             seed=None, wall_s=s["wall_s"] if corpus == "pool" else None,
                             notes=f"corpus={corpus}; errors={s['errors']}; credit: Ján Pastorek, AutoGraphForge"))
        log(f"[agf] {label}: known in pool={int(kn.sum())} in survivors={int((kn & pipe).sum())} errors={s['errors']}")

    # ---- outputs ----------------------------------------------------------------------
    stem = f"census_{args.table}_{args.target}"
    rows_path = pin.write_rows(stem, rows)
    metrics_path = rows_path.with_suffix(".metrics.json")
    metrics_path.write_text(json.dumps(metrics, indent=1, ensure_ascii=False, default=_json_default))
    dec_path = rows_path.with_suffix(".decisions.csv")
    with dec_path.open("w", newline="") as fh:
        w = csv.writer(fh)
        header = ["i", "name", "hypothesis", "op", "is_false", "table_class"]
        for rule in RULES:
            header += [f"{rule}_class", f"{rule}_alias_of"]
        header += ["dalmatian_strict_rows", "pipeline_survivor"] + [f"agf_known[{label}]" for label in agf]
        w.writerow(header)
        nstrict = accepts.get("_dalmatian_nstrict")
        for i in range(pool.M):
            line = [i, pool.names[i], pool.hyp[i], pool.op[i], bool(pool.is_false[i]), int(pool.class_id[i])]
            for rule in RULES:
                line += [parts[rule]["cls"][i], int(parts[rule]["alias_of"][i])]
            line += ["" if nstrict is None else int(nstrict[i]), bool(pipe[i])]
            line += [bool(known[label][i]) for label in agf]
            w.writerow(line)

    failed = [g for g, v in metrics["gates"].items() if not v["pass"]]
    log(f"rows -> {rows_path}\nmetrics -> {metrics_path}\ndecisions -> {dec_path}")
    log(f"gates failed: {failed}" if failed else "all gates pass")
    if failed:
        raise SystemExit(1)


if __name__ == "__main__":
    main()
