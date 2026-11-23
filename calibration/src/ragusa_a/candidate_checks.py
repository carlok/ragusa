"""Candidate checks (Sprint 12): run a harvest shortlist through four artefact checks and group what
survives into distinct relations.

Checks, each with `ragusa check`:

- mirror: refuted on `knotinfo_le13_mirror` (mirror images, which KnotInfo does not store);
- completed: refuted once missing cells are filled with intervals implied by the knot catalogue;
- range edge: every object where the bound is tight has the table's largest crossing number;
- coverage: the hypothesis is recorded on no knot with the largest crossing number.

A bound that fails any check is dropped. The survivors are grouped: each bound `(H) → (t op c·f + k)`
is written as a linear form `±(t − c·f − k) <= 0`, scaled so the alphabetically first column has
coefficient ±1; the two orientations of one line are one relation (an identity when both occur
under the same hypothesis), and one relation under several hypotheses is one group.

    python -m ragusa_a.candidate_checks ../external/results/harvest_knotinfo_le13.json \
        --out ../external/results/candidates_knotinfo_le13.json
"""

from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
import tempfile
from collections import defaultdict
from fractions import Fraction
from pathlib import Path

REPO = Path(__file__).resolve().parents[3]
ENGINE = REPO / "engine"
BINARY = ENGINE / "target" / "release" / "ragusa"

BOUND = re.compile(r"^\((?P<h>[^)]*)\) → \((?P<t>\S+) (?P<op><=|>=) (?P<rhs>.*)\)$")


def ragusa(*args: str) -> None:
    done = subprocess.run([str(BINARY), *args], capture_output=True, text=True)
    if done.returncode != 0:
        raise SystemExit(f"ragusa {' '.join(args)}: {done.stderr.strip()}")


def parse(bound: str) -> tuple[str, str, str, dict[str, Fraction], Fraction]:
    m = BOUND.match(bound)
    if not m:
        raise ValueError(bound)
    terms, constant = {}, Fraction(0)
    for part in m["rhs"].split(" + "):
        if "·" in part:
            c, f = part.split("·")
            terms[f] = Fraction(c)
        else:
            constant = Fraction(part)
    return m["h"], m["t"], m["op"], terms, constant


def line_and_side(bound: str) -> tuple[tuple, int]:
    """The line `Σ a·x + k = 0` the bound's boundary lies on, normalised, and the side (+1: `<= 0`)."""
    _, t, op, terms, k = parse(bound)
    form = defaultdict(Fraction, {t: Fraction(1)})
    for f, c in terms.items():
        form[f] -= c
    const = -k
    if op == ">=":
        form = {c: -v for c, v in form.items()}
        const = -const
    first = sorted(c for c, v in form.items() if v != 0)[0]
    scale = abs(form[first])
    line = tuple((c, form[c] / scale) for c in sorted(form) if form[c] != 0) + (("1", const / scale),)
    sign = 1 if form[first] > 0 else -1
    # Orient the line by its first coefficient, so both sides of one line share the key.
    key = tuple((c, v * sign) for c, v in line)
    return key, sign


def main() -> int:
    p = argparse.ArgumentParser()
    p.add_argument("harvest", help="harvest JSON from ragusa_a.harvest")
    p.add_argument("--table", default="knotinfo_le13")
    p.add_argument("--mirror-table", default="knotinfo_le13_mirror")
    p.add_argument("--edge", default="crossing_number")
    p.add_argument("--out", required=True)
    a = p.parse_args()

    subprocess.run(["cargo", "build", "--release", "-q", "-p", "ragusa-cli"], cwd=ENGINE, check=True)
    rev = subprocess.run(["git", "-C", str(REPO), "rev-parse", "HEAD"], capture_output=True,
                         text=True, check=True).stdout.strip()
    harvest = json.loads(Path(a.harvest).read_text())
    shortlist = [r for r in harvest["bounds"] if r["shortlisted"]]
    with tempfile.TemporaryDirectory() as tmp:
        bounds_file = Path(tmp) / "bounds.txt"
        bounds_file.write_text("".join(r["bound"] + "\n" for r in shortlist))
        runs = {
            "plain": ["check", a.table, "--edge", a.edge],
            "completed": ["check", a.table, "--complete-knots"],
            "mirror": ["check", a.mirror_table],
        }
        checks = {}
        for name, args in runs.items():
            out = Path(tmp) / f"{name}.json"
            ragusa(*args, "--bounds", str(bounds_file), "--out", str(out))
            checks[name] = json.loads(out.read_text())

    by_bound = {name: {r["bound"]: r for r in doc["results"]} for name, doc in checks.items()}
    rows = []
    for r in shortlist:
        b = r["bound"]
        plain, completed, mirror = (by_bound[k][b] for k in ("plain", "completed", "mirror"))
        failed = []
        if plain["refuted"]:
            failed.append("refuted_on_full_table")
        if mirror["refuted"]:
            failed.append("refuted_on_mirrors")
        if completed["refuted"]:
            failed.append("refuted_after_catalogue_completion")
        if plain["tight"] and plain["tight_at_edge"] == plain["tight"]:
            failed.append("tight_only_at_range_edge")
        if plain["hypothesis_known_at_edge"] == 0:
            failed.append("hypothesis_unrecorded_at_range_edge")
        rows.append({**r, "checks": {"full_table": plain, "catalogue_completed": completed, "mirrors": mirror},
                     "failed_checks": failed})

    survivors = [r for r in rows if not r["failed_checks"]]
    groups: dict[tuple, dict] = {}
    for r in survivors:
        key, side = line_and_side(r["bound"])
        h = parse(r["bound"])[0]
        g = groups.setdefault(key, {"bounds": [], "hypotheses": defaultdict(set)})
        g["bounds"].append(r)
        g["hypotheses"][h].add(side)
    group_rows = []
    for key, g in groups.items():
        g["bounds"].sort(key=lambda r: (-r["seeds_survived"], -r["held_out_confirmed"], r["bound"]))
        group_rows.append({
            "line": [[c, str(v)] for c, v in key],
            "hypotheses": {h: ("identity" if len(sides) == 2 else "inequality") for h, sides in sorted(g["hypotheses"].items())},
            "bounds": [r["bound"] for r in g["bounds"]],
            "seeds_survived": max(r["seeds_survived"] for r in g["bounds"]),
            "held_out_confirmed": sum(r["held_out_confirmed"] for r in g["bounds"]),
            "full_table_confirmed": sum(r["checks"]["full_table"]["confirmed"] for r in g["bounds"]),
            "novelty": sorted({n for r in g["bounds"] for n in r["novelty"]}),
        })
    group_rows.sort(key=lambda g: (-g["seeds_survived"], -g["held_out_confirmed"], g["bounds"][0]))
    for k, g in enumerate(group_rows, 1):
        g["id"] = f"K{k:02d}"

    failures = defaultdict(int)
    for r in rows:
        for f in r["failed_checks"]:
            failures[f] += 1
    result = {"harvest": a.harvest, "engine_git_rev": rev, "table": a.table, "mirror_table": a.mirror_table,
              "edge": checks["plain"]["edge"], "completion": checks["completed"].get("completion"),
              "shortlist": len(rows), "failures": dict(failures), "survivors": len(survivors),
              "groups": group_rows, "bounds": rows}
    out = Path(a.out)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(result, indent=1) + "\n")

    print(f"# candidate checks: {len(rows)} shortlisted bounds, engine {rev[:7]}")
    print(f"edge {checks['plain']['edge']}; completion {checks['completed'].get('completion')}")
    print("failed (a bound can fail several): " + ", ".join(f"{k} {v}" for k, v in sorted(failures.items())))
    print(f"surviving bounds: {len(survivors)}; distinct relations: {len(group_rows)}\n")
    for g in group_rows:
        hyps = ", ".join(f"{h} ({kind})" for h, kind in g["hypotheses"].items())
        print(f"{g['id']} [{g['seeds_survived']} seeds, {'/'.join(g['novelty'])}] {g['bounds'][0]}  | {hyps}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
