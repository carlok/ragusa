"""Rule study: which acceptance rule keeps bounds that still hold on objects the run never saw?

For every target and seed, runs `ragusa floor <table> --target T --holdout fraction=F,seed=S
--draws 0` and reads `candidates_heldout.csv`. Every rule is applied to the same pool, the
representatives of the table-duplicates stage, and scored on the held-out objects:

- kept: bounds the rule accepts; tested: of those, bounds whose hypothesis holds on some held-out
  object; refuted: of those, bounds refuted on at least one held-out object.
- The pool itself is the baseline: a rule that picks bounds at random keeps, in expectation, the
  pool's refuted rate. A rule helps only if its rate is lower while it still keeps bounds.

Rules are chosen on one seed range and confirmed on a disjoint one (`--seeds 0-4`, `--seeds 5-9`).
Stdlib only; needs a release build of the engine.

    python -m ragusa_a.rule_study graph_data --seeds 0-4 --out results/rows/rule_study_graph_data_s0-4.json
"""

from __future__ import annotations

import argparse
import csv
import json
import subprocess
import sys
import tempfile
from pathlib import Path

REPO = Path(__file__).resolve().parents[3]
ENGINE = REPO / "engine"
BINARY = ENGINE / "target" / "release" / "ragusa"

RULES = [
    ("duplicates pool", lambda c: True),
    ("sharp >= 1 (current)", lambda c: c["sharp"] >= 1),
    ("sharp >= 2", lambda c: c["sharp"] >= 2),
    ("sharp >= 3", lambda c: c["sharp"] >= 3),
    ("sharp >= 5", lambda c: c["sharp"] >= 5),
    ("sharp >= 10", lambda c: c["sharp"] >= 10),
    ("tight >= 1", lambda c: c["tight"] >= 1),
    ("tight >= 2", lambda c: c["tight"] >= 2),
    ("tight >= 5", lambda c: c["tight"] >= 5),
    ("tight >= 10", lambda c: c["tight"] >= 10),
    ("denominator <= 10", lambda c: c["den"] <= 10),
    ("sharp >= 1 and denominator <= 10", lambda c: c["sharp"] >= 1 and c["den"] <= 10),
    ("sharp >= 1 and denominator <= 100", lambda c: c["sharp"] >= 1 and c["den"] <= 100),
    ("sharp >= 2 and denominator <= 100", lambda c: c["sharp"] >= 2 and c["den"] <= 100),
    ("tight >= 2 and denominator <= 10", lambda c: c["tight"] >= 2 and c["den"] <= 10),
    # Thresholds that scale with the objects where the hypothesis holds on training (Sprint 10).
    ("tight >= 1% of applicable", lambda c: c["tight"] >= 0.01 * c["app"]),
    ("tight >= 2% of applicable", lambda c: c["tight"] >= 0.02 * c["app"]),
    ("tight >= 5% of applicable", lambda c: c["tight"] >= 0.05 * c["app"]),
    ("tight >= 10% of applicable", lambda c: c["tight"] >= 0.10 * c["app"]),
    ("tight >= sqrt(applicable)", lambda c: c["tight"] >= c["app"] ** 0.5),
    ("tight >= sqrt(applicable) / 2", lambda c: c["tight"] >= 0.5 * c["app"] ** 0.5),
    ("tight >= max(2, 2% of applicable)", lambda c: c["tight"] >= max(2, 0.02 * c["app"])),
    ("tight >= max(2, 5% of applicable)", lambda c: c["tight"] >= max(2, 0.05 * c["app"])),
]


def ragusa(*args: str) -> str:
    done = subprocess.run([str(BINARY), *args], capture_output=True, text=True)
    if done.returncode != 0:
        raise RuntimeError(f"ragusa {' '.join(args)}: {done.stderr.strip()}")
    return done.stdout


def seed_range(text: str) -> list[int]:
    lo, _, hi = text.partition("-")
    return list(range(int(lo), int(hi or lo) + 1))


def pool(path: Path) -> list[dict]:
    with path.open() as f:
        return [
            {
                "sharp": int(r["sharp_witnesses"]),
                "tight": int(r["tight_training"]),
                "den": int(r["max_denominator"]),
                "app": int(r["applicable_training"]),
                "tested": int(r["applicable"]) > 0,
                "refuted": int(r["refuted"]) > 0,
            }
            for r in csv.DictReader(f)
            if r["duplicates_class"] == "representative"
        ]


def score(rows: list[dict]) -> dict:
    tested = [r for r in rows if r["tested"]]
    return {"kept": len(rows), "tested": len(tested), "refuted": sum(r["refuted"] for r in tested)}


def rate(s: dict) -> float | None:
    return s["refuted"] / s["tested"] if s["tested"] else None


def main() -> int:
    p = argparse.ArgumentParser()
    p.add_argument("table")
    p.add_argument("--targets", help="comma-separated; default every numeric column")
    p.add_argument("--seeds", default="0-4")
    p.add_argument("--fraction", type=float, default=0.2)
    p.add_argument("--out", required=True)
    a = p.parse_args()

    subprocess.run(["cargo", "build", "--release", "-q", "-p", "ragusa-cli"], cwd=ENGINE, check=True)
    rev = subprocess.run(["git", "-C", str(REPO), "rev-parse", "HEAD"], capture_output=True,
                         text=True, check=True).stdout.strip()
    targets = a.targets.split(",") if a.targets else ragusa("columns", a.table).split()
    runs, skipped = [], []
    with tempfile.TemporaryDirectory() as tmp:
        for target in targets:
            for seed in seed_range(a.seeds):
                out = Path(tmp) / f"{target}_{seed}"
                try:
                    ragusa("floor", a.table, "--target", target, "--holdout",
                           f"fraction={a.fraction},seed={seed}", "--draws", "0", "--out", str(out),
                           # the rules are stated in terms of sharp-coverage witnesses
                           "--redundancy", "sharp_coverage")
                except RuntimeError as e:
                    skipped.append({"target": target, "seed": seed, "error": str(e)})
                    continue
                rows = pool(out / "candidates_heldout.csv")
                runs.append({"target": target, "seed": seed,
                             "rules": {name: score([r for r in rows if keep(r)]) for name, keep in RULES}})
                print(f"{target} seed {seed}: pool {len(rows)}", file=sys.stderr)

    summary = {}
    for name, _ in RULES:
        total = {k: sum(r["rules"][name][k] for r in runs) for k in ("kept", "tested", "refuted")}
        worse = better = equal = 0
        for r in runs:
            mine, base = rate(r["rules"][name]), rate(r["rules"]["duplicates pool"])
            if mine is None or base is None:
                continue
            worse += mine > base
            better += mine < base
            equal += mine == base
        summary[name] = {**total, "refuted_rate": rate(total), "runs_worse_than_pool": worse,
                         "runs_better_than_pool": better, "runs_equal": equal,
                         "runs_with_none_tested": sum(1 for r in runs if not r["rules"][name]["tested"])}

    result = {"table": a.table, "engine_git_rev": rev, "fraction": a.fraction, "seeds": a.seeds,
              "targets": targets, "runs": runs, "skipped": skipped, "summary": summary}
    Path(a.out).parent.mkdir(parents=True, exist_ok=True)
    Path(a.out).write_text(json.dumps(result, indent=1) + "\n")

    print(f"# rule study: {a.table}, seeds {a.seeds}, {len(runs)} runs ({len(skipped)} skipped), "
          f"{a.fraction:.0%} held out, engine {rev[:7]}\n")
    print("| rule | kept | tested | refuted | rate | runs worse / better / equal vs pool | runs with none tested |")
    print("|---|---:|---:|---:|---:|---|---:|")
    for name, s in summary.items():
        r = "n/a" if s["refuted_rate"] is None else f"{s['refuted_rate']:.3f}"
        print(f"| {name} | {s['kept']} | {s['tested']} | {s['refuted']} | {r} | "
              f"{s['runs_worse_than_pool']} / {s['runs_better_than_pool']} / {s['runs_equal']} | "
              f"{s['runs_with_none_tested']} |")
    return 0


if __name__ == "__main__":
    sys.exit(main())
