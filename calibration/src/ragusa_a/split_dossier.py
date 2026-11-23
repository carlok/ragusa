"""One self-contained prompt per candidate (Sprint 12 follow-up).

The dossier written by `ragusa_a.dossier` is one folder for all 42 candidates: a shared README plus
one `Kxx.md` per candidate. That is too much to hand to an external model in one go, and the
candidates are independent of each other (each is a statement about one relation between KnotInfo
columns; none refers to another), so they can be judged one at a time.

This script re-emits the same dossier as one folder per candidate. Each folder holds a `prompt.md`
that repeats the task, the scope, the checks already run, the conventions and only the definitions
that candidate uses, so it can be pasted into a fresh conversation on its own, and an `answer.md`
skeleton to paste the reply into.

Every number comes from `candidates.json`; nothing here is typed by hand.

    python -m ragusa_a.split_dossier ../external/candidates/knotinfo_le13 \
        --out ../external/candidates/knotinfo_le13_reviewed
"""

from __future__ import annotations

import argparse
import json
import shutil
from pathlib import Path

from .dossier import CHIRAL, INVARIANTS, PROPERTIES

ANSWER_BLOCK = """### {id}
verdict: PROVED | REFUTED | KNOWN | IMPLIED | OPEN
statement as you understand it: <one line; point out any ambiguity in the definitions>
argument: <proof, or derivation from known results, or explanation of why it is open>
counterexample: <knot, its relevant invariant values, and how each value is known; or "none">
references: <author, title, venue, year, theorem or page; only sources you are confident exist>
confidence: high | medium | low
what would settle it: <for OPEN or low confidence: the computation, family or source to check>"""


def definitions(rec: dict) -> str:
    rows = "\n".join(
        f"| `{c}` | {INVARIANTS.get(c, (c, 'see KnotInfo'))[0]} | {INVARIANTS.get(c, (c, 'see KnotInfo'))[1]} |"
        for c in rec["columns"]
    )
    props = "\n".join(
        f"| `{h}` | {PROPERTIES.get(h, 'see KnotInfo')}"
        f"{' (chirality-dependent)' if h in CHIRAL else ''} |"
        for h in sorted(rec["hypotheses"])
    )
    return (
        "| KnotInfo column | symbol | definition |\n|---|---|---|\n"
        f"{rows}\n\n### Properties used\n\n| KnotInfo column | meaning |\n|---|---|\n{props}"
    )


def evidence_table(rec: dict) -> str:
    head = (
        "| inequality | seeds (of 5) | held-out knots confirmed / undecided | "
        "knots with the property | confirmed | undecided | equality holds on | "
        "examples with equality | mirrors confirmed | catalogue class |\n"
        "|---|---:|---|---:|---:|---:|---:|---|---:|---|"
    )
    rows = [
        f"| {e['plain']} | {e['seeds_survived']} | "
        f"{e['held_out_confirmed']} / {e['held_out_undecided']} | {e['table_applicable']} | "
        f"{e['table_confirmed']} | {e['table_undecided']} | {e['table_tight']} | "
        f"{', '.join(e['tight_examples'])} | {e['mirror_confirmed']} | "
        f"{', '.join(e['catalogue_class'])} |"
        for e in rec["evidence"]
    ]
    return "\n".join([head, *rows])


def prompt(rec: dict, total: int, rev: str) -> str:
    bounds = "\n".join(f"- `{e['bound']}`" for e in rec["evidence"])
    return f"""# {rec['id']}: one knot invariant conjecture to check

This is candidate {rec['id']} of {total} produced by a program (ragusa) that fits linear
inequalities to a knot database and then discards the ones its automatic checks refute. The other
candidates are not needed here: judge this one on its own. Nobody has reviewed it. It may be a known
theorem, a consequence of known theorems, false, or open. Decide which, as rigorously as you can.

## The statement

**{rec['statement']}**

As the program printed it (hypothesis → inequality on KnotInfo columns):

{bounds}

A statement written with `=` is the pair of inequalities above it; a relation listed under several
hypotheses is one claim per hypothesis.

## Scope

The claim is about **all prime knots** with the given property (or all prime knots), not only the
knots in the database. It was found on prime knots with at most 13 crossings, so treat it as a
conjecture about all prime knots with that property. Say separately whether it also holds, or fails,
for composite knots.

## What the program already checked

1. The inequality holds on every prime knot with 3 to 13 crossings in KnotInfo
   (https://knotinfo.org) that has the property, whenever the invariants are known exactly or up to
   an interval (it counts as violated only if it fails for every value in the recorded intervals).
2. It was learned on 80% of the knots and never failed on the other 20%, in at least 3 of 5 random
   splits, and it was confirmed on at least one held-out knot.
3. It is not implied by the catalogue of inequalities in M. Jabłonowski, *Integer knot invariants:
   inequalities, computations, and open problems*, arXiv:2605.22652 (v4), via transitive closure of
   its relations with the obvious linear bounds.
4. It holds on the mirror images of the knots (KnotInfo stores one chirality per knot): signature,
   s, tau and epsilon were negated; chirality-dependent properties were treated as unknown there.
5. It is not refuted after filling unknown values with intervals implied by that catalogue.
6. The knots where equality holds are not all at the largest crossing number (13), and the property
   is recorded for 13-crossing knots.
7. It is not a constant bound and does not involve an invariant with at most three distinct values.

These checks remove many artefacts, not all. The database stops at 13 crossings, so families such as
torus knots T(2,n), T(3,n), pretzel knots and cables beyond 13 crossings are the first places to look
for a counterexample.

## Evidence on KnotInfo, prime knots with 3 to 13 crossings

{evidence_table(rec)}

Held-out counts add up over the five splits, so a knot can be counted up to five times.
`undecided`: some invariant is unknown, or an interval that does not decide the inequality.

## Conventions

- KnotInfo's chirality for each knot; the right-handed trefoil (3_1 in the table) has signature -2,
  Rasmussen s = 2, tau = 1. So for positive knots s = 2 tau = 2g > 0 and the signature is negative.
- Knot names such as `12n_242` are KnotInfo names; KnotInfo lists one of each mirror pair.

## Invariants used

{definitions(rec)}

## What to return

Return exactly this block, and nothing else:

```
{ANSWER_BLOCK.format(id=rec['id'])}
```

- KNOWN: stated in the literature (give the exact reference). IMPLIED: follows from known results by
  an argument you give. PROVED: your own complete proof. REFUTED: an explicit counterexample or a
  proof of the negation.
- Do not invent references. If you are unsure a paper states something, say so and mark confidence
  low.
- Invariant values of specific knots must say where they come from (a theorem, a computation you
  show, or a database you name).

<!-- ragusa engine rev {rev} -->
"""


def main() -> int:
    p = argparse.ArgumentParser()
    p.add_argument("dossier", help="folder written by ragusa_a.dossier (holds candidates.json)")
    p.add_argument("--out", required=True)
    a = p.parse_args()

    data = json.loads((Path(a.dossier) / "candidates.json").read_text())
    records = data["candidates"]
    rev = data["engine_git_rev"]
    out = Path(a.out)
    if out.exists():
        shutil.rmtree(out)
    out.mkdir(parents=True)

    for rec in records:
        folder = out / rec["id"]
        folder.mkdir()
        (folder / "prompt.md").write_text(prompt(rec, len(records), rev))
        (folder / "answer.md").write_text(
            f"<!-- paste the model's reply below, unedited; one file per candidate -->\n\n"
            f"{ANSWER_BLOCK.format(id=rec['id'])}\n"
        )

    index = "\n".join(f"| [{r['id']}]({r['id']}/prompt.md) | {r['statement']} |" for r in records)
    (out / "README.md").write_text(f"""# Candidates as {len(records)} isolated requests

One folder per candidate, each with a `prompt.md` that stands on its own: the task, the scope, the
checks already run, the conventions and the definitions that candidate uses. Submit them one at a
time, in any order; they are independent statements about different relations and no candidate
refers to another. Paste each reply into the `answer.md` next to its prompt.

Same content as the combined dossier next door (`../knotinfo_le13/`), rewritten by
`python -m ragusa_a.split_dossier`. Derived from KnotInfo, so it stays out of git until KnotInfo's
reuse terms are settled. ragusa engine rev `{rev}`.

| candidate | statement |
|---|---|
{index}
""")
    print(f"split: {len(records)} candidates -> {out}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
