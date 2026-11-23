"""Held-out sweep: does sharp coverage select bounds that fail on unseen objects?

For every target and seed, runs `ragusa floor <table> --target T --holdout fraction=F,seed=S
--draws 0` and reads the per-stage counts from heldout.json. Writes one JSON with every run and
the pooled rates, and prints a Markdown summary. Stdlib only; needs a release build of the engine.

    python -m ragusa_a.heldout_sweep graph_data --out results/rows/heldout_sweep_graph_data.json
    python -m ragusa_a.heldout_sweep knotinfo_le12 --targets <columns> \
        --out external/results/heldout_sweep_knotinfo_le12.json
"""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
import tempfile
from pathlib import Path

REPO = Path(__file__).resolve().parents[3]
ENGINE = REPO / "engine"
BINARY = ENGINE / "target" / "release" / "ragusa"
STAGES = ["validity", "table_duplicates", "sharp_coverage", "generality"]


def ragusa(*args: str) -> str:
    done = subprocess.run([str(BINARY), *args], capture_output=True, text=True)
    if done.returncode != 0:
        raise RuntimeError(f"ragusa {' '.join(args)}: {done.stderr.strip()}")
    return done.stdout


def git_rev() -> str:
    return subprocess.run(
        ["git", "-C", str(REPO), "rev-parse", "HEAD"], capture_output=True, text=True, check=True
    ).stdout.strip()


def main() -> int:
    p = argparse.ArgumentParser()
    p.add_argument("table")
    p.add_argument("--targets", help="comma-separated; default every numeric column")
    p.add_argument("--seeds", type=int, default=5)
    p.add_argument("--seed-from", type=int, default=0)
    p.add_argument("--fraction", type=float, default=0.2)
    p.add_argument("--engine-args", default="",
                   help="extra `ragusa floor` arguments, space-separated, e.g. '--min-tight 10'")
    p.add_argument("--out", required=True)
    a = p.parse_args()

    subprocess.run(["cargo", "build", "--release", "-q", "-p", "ragusa-cli"], cwd=ENGINE, check=True)
    targets = a.targets.split(",") if a.targets else ragusa("columns", a.table).split()
    runs, skipped = [], []
    with tempfile.TemporaryDirectory() as tmp:
        for target in targets:
            for seed in range(a.seed_from, a.seed_from + a.seeds):
                out = Path(tmp) / f"{target}_{seed}"
                rule = f"fraction={a.fraction},seed={seed}"
                try:
                    ragusa("floor", a.table, "--target", target, "--holdout", rule,
                           "--draws", "0", "--out", str(out), *a.engine_args.split())
                except RuntimeError as e:
                    skipped.append({"target": target, "seed": seed, "error": str(e)})
                    continue
                h = json.loads((out / "heldout.json").read_text())
                runs.append({
                    "target": target,
                    "seed": seed,
                    "train_objects": h["train"]["n_objects"],
                    "held_out_objects": h["held_out"]["n_objects"],
                    "stages": {
                        s["filter"]: {k: s[k] for k in
                                      ("kept", "refuted_on_held_out", "supported_on_held_out",
                                       "untested_on_held_out")}
                        for s in h["stages"]
                    },
                })
                print(f"{target} seed {seed}: " + ", ".join(
                    f"{s} {runs[-1]['stages'][s]['refuted_on_held_out']}/{runs[-1]['stages'][s]['kept']}"
                    for s in runs[-1]["stages"]), file=sys.stderr)

    # Stage names come from the runs, so optional stages (touch) are reported when present.
    stages = list(runs[0]["stages"]) if runs else STAGES
    last = stages[-1]

    def pooled(stage: str) -> dict:
        kept = sum(r["stages"][stage]["kept"] for r in runs)
        refuted = sum(r["stages"][stage]["refuted_on_held_out"] for r in runs)
        untested = sum(r["stages"][stage]["untested_on_held_out"] for r in runs)
        tested = kept - untested
        return {"kept": kept, "refuted": refuted, "untested": untested,
                "refuted_rate_of_tested": refuted / tested if tested else None}

    def rate(r: dict, stage: str) -> float | None:
        s = r["stages"][stage]
        tested = s["kept"] - s["untested_on_held_out"]
        return s["refuted_on_held_out"] / tested if tested else None

    comparable = [r for r in runs if rate(r, "validity") is not None and rate(r, last) is not None]
    worse = sum(1 for r in comparable if rate(r, last) > rate(r, "validity"))
    better = sum(1 for r in comparable if rate(r, last) < rate(r, "validity"))
    result = {
        "table": a.table,
        "engine_git_rev": git_rev(),
        "engine_args": a.engine_args,
        "fraction": a.fraction,
        "seeds": a.seeds,
        "seed_from": a.seed_from,
        "targets": targets,
        "runs": runs,
        "skipped": skipped,
        "pooled": {s: pooled(s) for s in stages},
        "survivors_vs_validity": {"runs_compared": len(comparable), "survivors_worse": worse,
                                  "survivors_better": better,
                                  "equal": len(comparable) - worse - better},
    }
    Path(a.out).parent.mkdir(parents=True, exist_ok=True)
    Path(a.out).write_text(json.dumps(result, indent=1) + "\n")

    print(f"# held-out sweep: {a.table}, {len(runs)} runs ({len(skipped)} skipped), seeds "
          f"{a.seed_from}-{a.seed_from + a.seeds - 1}, {a.fraction:.0%} held out, engine "
          f"{result['engine_git_rev'][:7]}, args '{a.engine_args}'\n")
    print("| stage | kept | refuted | untested | refuted / tested |")
    print("|---|---:|---:|---:|---:|")
    for s in stages:
        q = result["pooled"][s]
        r = "n/a" if q["refuted_rate_of_tested"] is None else f"{q['refuted_rate_of_tested']:.3f}"
        print(f"| {s} | {q['kept']} | {q['refuted']} | {q['untested']} | {r} |")
    c = result["survivors_vs_validity"]
    print(f"\nPer run, survivors' refuted rate vs validity's: worse in {c['survivors_worse']}, "
          f"better in {c['survivors_better']}, equal in {c['equal']} of {c['runs_compared']}.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
