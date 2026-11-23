"""AutoGraphForge known-results filter over its own CEGIS output (Sprint 1, number 4 + G-A2).

Calls AutoGraphForge's shipped `pipeline.cegis_novelty.classify_statement`
unmodified. Two verdict sets are reported, never one:

- `gc2table`: standalone call with the module's own 44-entry `_GC2TABLE` columns;
- `stored`:   `metadata.novel` as persisted by the pipeline run itself.

This is a novelty filter (PROMPT.md section 2), credited to Ján Pastorek.
Nothing here is published before contacting him.

    uv run python -m ragusa_a.agf_novelty
"""

from __future__ import annotations

import csv
import json
import subprocess
import sys
import time

from ragusa_a import pin

VENDOR = pin.REPO / "calibration" / "vendor.lock"
AGF = (pin.REPO / ".." / "_ext" / "AutoGraphForge").resolve()
# One row per statement from Pastorek's results (public, MIT, in his repository) paired with our
# verdicts on his filter. It reproduces his statements row by row, so it stays out of the tracked tree:
# external/ is gitignored.
OUTREACH = pin.REPO / "external" / "outreach" / "agf_known.csv"


def pinned_commit() -> str:
    for line in VENDOR.read_text().splitlines():
        if line.startswith("autographforge"):
            return line.split()[2]
    raise RuntimeError("autographforge not in vendor.lock")


def load_cegis_novelty():
    """Import AutoGraphForge's pipeline.cegis_novelty after checking the pinned commit."""
    commit = pinned_commit()
    head = subprocess.run(["git", "-C", str(AGF), "rev-parse", "HEAD"],
                          capture_output=True, text=True, check=True).stdout.strip()
    if head != commit:
        sys.exit(f"AutoGraphForge HEAD {head} != vendor.lock {commit}")
    if str(AGF) not in sys.path:
        sys.path.insert(0, str(AGF))
    from pipeline import cegis_novelty as cn
    return cn, commit


def main() -> None:
    cn, commit = load_cegis_novelty()

    results_path = AGF / "results" / "cegis_results.json"
    rows = json.loads(results_path.read_text())["conjectures"]
    cols = list(cn._GC2TABLE.keys())

    t0 = time.perf_counter()
    standalone = {r["id"]: cn.classify_statement(r["statement"], cols) for r in rows}
    wall = time.perf_counter() - t0

    std_known = {i for i, (known, _) in standalone.items() if known}
    stored_known = {r["id"] for r in rows if r["metadata"].get("novel") is False}

    common = dict(
        system="autographforge", system_version=commit, table="results/cegis_results.json",
        table_sha256=pin.sha256_file(results_path), target=None, generator="all",
        filter_kind="novelty", stage=0, order="-", n_in=len(rows), n_alias=0, seed=None,
    )
    rid = pin.run_id("autographforge")
    out = [
        dict(common, run_id=rid, filter="classify_statement[_GC2TABLE]",
             n_representative=len(rows) - len(std_known), n_discarded=len(std_known),
             wall_s=round(wall, 3), notes=f"columns=_GC2TABLE ({len(cols)} entries)"),
        dict(common, run_id=rid, filter="stored metadata.novel",
             n_representative=len(rows) - len(stored_known), n_discarded=len(stored_known),
             wall_s=None, notes="verdicts persisted by run_cegis.wrap_survivors (seed-frame columns)"),
        dict(common, run_id=rid, filter="overlap[_GC2TABLE & stored]",
             n_representative=len(rows) - len(std_known | stored_known),
             n_discarded=len(std_known & stored_known), n_alias=len(std_known ^ stored_known),
             wall_s=None,
             notes="n_discarded=flagged by both; n_alias=flagged by exactly one (not an alias count)"),
    ]
    path = pin.write_rows("agf_novelty", out)

    OUTREACH.parent.mkdir(parents=True, exist_ok=True)
    byid = {r["id"]: r for r in rows}
    with OUTREACH.open("w", newline="") as fh:
        w = csv.writer(fh)
        w.writerow(["id", "statement", "generation_method", "known_standalone_gc2table",
                    "reason_standalone", "known_stored", "reason_stored"])
        for i in sorted(std_known | stored_known):
            r = byid[i]
            w.writerow([i, r["statement"], r["generation_method"], i in std_known,
                        standalone[i][1], i in stored_known, r["metadata"].get("known_as")])

    print(f"rows -> {path}")
    print(f"csv  -> {OUTREACH}")
    for row in out:
        print(f"  {row['filter']:<34} n_in={row['n_in']} discarded={row['n_discarded']}")


if __name__ == "__main__":
    main()
