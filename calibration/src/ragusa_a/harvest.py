"""Harvest: survivors that keep surviving, never fail on unseen objects, and are not already known.

For every target and seed, runs `ragusa floor <table> --target T --holdout fraction=F,seed=S
--draws 0 <engine args>` (default engine args: the vetted pipeline, `--min-tight sqrt` with the
default `greedy_touch` redundancy stage), optionally with `--oracle knots`. Survivors of the
training run are joined by their exact bound text across seeds, with their held-out verdicts and,
for knot tables, their catalogue class.

A bound is shortlisted when all of these hold:

- it survives in at least `--min-seeds` seeds;
- it is refuted on no held-out object in any of them, and confirmed on at least one;
- with the knot oracle, it is not implied by the catalogue;
- it is not trivial: it has at least one feature term (a constant bound only restates a column's
  range on this table), and no column it names takes at most `--max-distinct` distinct values on
  the table (a bound on a column with two or three values restates that column's range).

Shortlisted bounds are ranked by the number of seeds, then by held-out confirmations. Nothing here is
a claim of novelty: "not implied" is relative to one catalogue, and the shortlist is the input to a
literature check. `--from-json` recomputes the shortlist from an earlier harvest without rerunning it.

    python -m ragusa_a.harvest knotinfo_le13 --oracle-knots \
        --out ../external/results/harvest_knotinfo_le13.json
    python -m ragusa_a.harvest fano3c --targets degree,volume \
        --engine-args "--min-tight sqrt --hypotheses TRUE,all" --out ../results/rows/harvest_fano3c.json
    python -m ragusa_a.harvest knotinfo_le13 --from-json ../external/results/harvest_knotinfo_le13.json \
        --out ../external/results/harvest_knotinfo_le13.json
"""

from __future__ import annotations

import argparse
import csv
import json
import re
import subprocess
import sys
import tempfile
from collections import Counter, defaultdict
from pathlib import Path

REPO = Path(__file__).resolve().parents[3]
ENGINE = REPO / "engine"
BINARY = ENGINE / "target" / "release" / "ragusa"


def ragusa(*args: str) -> str:
    done = subprocess.run([str(BINARY), *args], capture_output=True, text=True)
    if done.returncode != 0:
        raise RuntimeError(f"ragusa {' '.join(args)}: {done.stderr.strip()}")
    return done.stdout


def columns_of(bound: str) -> tuple[str, list[str]]:
    """Target and feature columns of a bound as `Bound` prints it: `(h) → (t op c·f + ... + k)`."""
    body = bound.split(" → ", 1)[1][1:-1]
    return body.split()[0], re.findall(r"·([A-Za-z_0-9]+)", body)


def harvest(a: argparse.Namespace) -> dict:
    rev = subprocess.run(["git", "-C", str(REPO), "rev-parse", "HEAD"], capture_output=True,
                         text=True, check=True).stdout.strip()
    targets = a.targets.split(",") if a.targets else ragusa("columns", a.table).split()
    seeds = list(range(a.seed_from, a.seed_from + a.seeds))
    bounds: dict[str, dict] = defaultdict(lambda: {
        "target": None, "seeds": [], "refuted_seeds": [], "applicable": 0, "refuted": 0,
        "confirmed": 0, "undecided": 0, "novelty": set(), "implied_by": set(), "examples": set(),
    })
    runs, skipped = [], []
    with tempfile.TemporaryDirectory() as tmp:
        for target in targets:
            for seed in seeds:
                out = Path(tmp) / f"{target}_{seed}"
                args = ["floor", a.table, "--target", target, "--holdout",
                        f"fraction={a.fraction},seed={seed}", "--draws", "0", "--out", str(out),
                        *a.engine_args.split()]
                if a.oracle_knots:
                    args += ["--oracle", "knots"]
                try:
                    ragusa(*args)
                except RuntimeError as e:
                    skipped.append({"target": target, "seed": seed, "error": str(e)})
                    continue
                held = json.loads((out / "heldout.json").read_text())
                classes = {}
                if a.oracle_knots:
                    novelty = json.loads((out / "novelty.json").read_text())
                    classes = {s["bound"]: s for s in novelty["survivors"]}
                for s in held["survivors"]:
                    b = bounds[s["bound"]]
                    b["target"] = target
                    b["seeds"].append(seed)
                    for k in ("applicable", "refuted", "confirmed", "undecided"):
                        b[k] += s[k]
                    if s["refuted"]:
                        b["refuted_seeds"].append(seed)
                        b["examples"].update(s["first_counterexamples"])
                    if s["bound"] in classes:
                        c = classes[s["bound"]]
                        b["novelty"].add(c["class"])
                        if c["implied_by"]:
                            b["implied_by"].add(c["implied_by"]["bound"])
                runs.append({"target": target, "seed": seed, "survivors": len(held["survivors"])})
                print(f"{target} seed {seed}: {len(held['survivors'])} survivors", file=sys.stderr)
    rows = [{
        "bound": text, "target": b["target"], "seeds_survived": len(b["seeds"]),
        "seeds_refuted_on_held_out": len(b["refuted_seeds"]), "held_out_applicable": b["applicable"],
        "held_out_refuted": b["refuted"], "held_out_confirmed": b["confirmed"],
        "held_out_undecided": b["undecided"], "novelty": sorted(b["novelty"]),
        "implied_by": sorted(b["implied_by"]), "counterexamples": sorted(b["examples"])[:5],
    } for text, b in bounds.items()]
    return {"table": a.table, "engine_git_rev": rev, "engine_args": a.engine_args,
            "oracle_knots": a.oracle_knots, "fraction": a.fraction, "seeds": seeds,
            "targets": targets, "runs": runs, "skipped": skipped, "bounds": rows}


def main() -> int:
    p = argparse.ArgumentParser()
    p.add_argument("table")
    p.add_argument("--targets", help="comma-separated; default every numeric column")
    p.add_argument("--seeds", type=int, default=5)
    p.add_argument("--seed-from", type=int, default=0)
    p.add_argument("--fraction", type=float, default=0.2)
    p.add_argument("--engine-args", default="--min-tight sqrt")
    p.add_argument("--oracle-knots", action="store_true")
    p.add_argument("--min-seeds", type=int, default=3)
    p.add_argument("--max-distinct", type=int, default=3)
    p.add_argument("--from-json", help="recompute the shortlist from this harvest instead of running")
    p.add_argument("--out", required=True, help="JSON with every survivor; the shortlist CSV is written next to it")
    a = p.parse_args()

    subprocess.run(["cargo", "build", "--release", "-q", "-p", "ragusa-cli"], cwd=ENGINE, check=True)
    result = json.loads(Path(a.from_json).read_text()) if a.from_json else harvest(a)
    rows = result["bounds"]

    distinct = {}
    for line in ragusa("column-stats", result["table"]).splitlines():
        name, count = line.split("\t")
        distinct[name] = int(count)

    def reasons(r: dict) -> list[str]:
        target, features = columns_of(r["bound"])
        out = []
        if r["seeds_survived"] < a.min_seeds:
            out.append("few_seeds")
        if r["seeds_refuted_on_held_out"]:
            out.append("refuted_on_held_out")
        if r["held_out_confirmed"] == 0:
            out.append("never_confirmed")
        if "implied" in r["novelty"]:
            out.append("implied_by_catalogue")
        if not features:
            out.append("constant_only")
        if any(distinct.get(c, a.max_distinct + 1) <= a.max_distinct for c in [target, *features]):
            out.append("low_cardinality_column")
        return out

    for r in rows:
        r["excluded_because"] = reasons(r)
        r["shortlisted"] = not r["excluded_because"]
    rows.sort(key=lambda r: (not r["shortlisted"], -r["seeds_survived"], -r["held_out_confirmed"], r["bound"]))
    shortlist = [r for r in rows if r["shortlisted"]]
    result.update({"min_seeds": a.min_seeds, "max_distinct": a.max_distinct,
                   "distinct_survivors": len(rows), "shortlist_size": len(shortlist), "bounds": rows})

    out = Path(a.out)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(result, indent=1) + "\n")
    with out.with_suffix(".shortlist.csv").open("w", newline="") as f:
        w = csv.writer(f)
        w.writerow(["rank", "bound", "target", "seeds_survived", "held_out_confirmed",
                    "held_out_undecided", "novelty"])
        for k, r in enumerate(shortlist, 1):
            w.writerow([k, r["bound"], r["target"], r["seeds_survived"], r["held_out_confirmed"],
                        r["held_out_undecided"], "|".join(r["novelty"])])

    seeds = result["seeds"]
    print(f"# harvest: {result['table']}, {len(result['runs'])} runs ({len(result['skipped'])} skipped), "
          f"seeds {seeds[0]}-{seeds[-1]}, engine {result['engine_git_rev'][:7]}, "
          f"args '{result['engine_args']}'{' + knot oracle' if result['oracle_knots'] else ''}\n")
    by_seeds = Counter(r["seeds_survived"] for r in rows)
    print("distinct survivors by seeds survived: " + ", ".join(f"{k}: {by_seeds[k]}" for k in sorted(by_seeds)))
    stable = [r for r in rows if r["seeds_survived"] >= a.min_seeds and not r["seeds_refuted_on_held_out"]
              and r["held_out_confirmed"] > 0]
    print(f"in >= {a.min_seeds} seeds, never refuted, confirmed somewhere: {len(stable)}")
    excluded = Counter(reason for r in stable for reason in r["excluded_because"])
    print("of those, excluded because (a bound can have several reasons): "
          + (", ".join(f"{k} {v}" for k, v in sorted(excluded.items())) or "none"))
    print(f"shortlist: {len(shortlist)}")
    if result["oracle_knots"]:
        print("shortlist by catalogue class: " + ", ".join(
            f"{k} {v}" for k, v in sorted(Counter("/".join(r["novelty"]) for r in shortlist).items())))
    print()
    for k, r in enumerate(shortlist[:15], 1):
        print(f"{k:>3}. [{r['seeds_survived']} seeds, {r['held_out_confirmed']} confirmed"
              f"{', ' + '/'.join(r['novelty']) if r['novelty'] else ''}] {r['bound']}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
