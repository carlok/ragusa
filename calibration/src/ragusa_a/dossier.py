"""Candidate dossier for an external language model (Sprint 12).

Reads the output of `ragusa_a.candidate_checks` and writes a folder meant to be handed, as is, to
another model before any person reads it:

- `README.md`: the task, the scope of the statements, conventions, definitions of every invariant
  and property that occurs, what has already been checked, and the required answer format;
- `candidates.md` and one `Kxx.md` per candidate: the statement, the bounds it came from, the
  evidence and the checks it passed;
- `candidates.json`: the same, machine-readable;
- `candidates.tex` and `candidates.pdf`: the same for reading as a document.

The folder is derived from KnotInfo, so it belongs under `external/` until KnotInfo's reuse terms
are settled. Earlier literature notes are deliberately not included, so that the model's answers are
not anchored on unverified claims.

    python -m ragusa_a.dossier ../external/results/candidates_knotinfo_le13.json \
        --out ../external/candidates/knotinfo_le13
"""

from __future__ import annotations

import argparse
import json
import re
import shutil
import subprocess
from fractions import Fraction
from pathlib import Path

# KnotInfo columns: (symbol, definition). Definitions are the standard ones; KnotInfo's own
# description pages (https://knotinfo.org) are authoritative for how each column was filled.
INVARIANTS = {
    "crossing_number": ("c", "minimal number of crossings over all diagrams of the knot"),
    "unknotting_number": ("u", "minimal number of crossing changes, over all diagrams, that turn the knot into the unknot"),
    "unknotting_number_algebraic": ("u_a", "algebraic unknotting number: minimal number of crossing changes needed to reach a knot with Alexander polynomial 1"),
    "three_genus": ("g", "Seifert genus: minimal genus of an orientable surface in S^3 bounded by the knot"),
    "smooth_four_genus": ("g_4", "smooth slice genus: minimal genus of a smoothly embedded orientable surface in B^4 bounded by the knot"),
    "topological_four_genus": ("g_4^top", "topological slice genus: as g_4 with locally flat surfaces"),
    "smooth_concordance_genus": ("g_c", "concordance genus: minimal Seifert genus of a knot smoothly concordant to the knot"),
    "crosscap_number": ("gamma", "crosscap number: minimal first Betti number of a non-orientable surface in S^3 bounded by the knot"),
    "smooth_4d_crosscap_number": ("gamma_4", "smooth 4-dimensional crosscap number: minimal first Betti number of a smoothly embedded non-orientable surface in B^4 bounded by the knot"),
    "topological_4d_crosscap_number": ("gamma_4^top", "as gamma_4 with locally flat surfaces"),
    "bridge_index": ("br", "bridge index: minimal number of bridges (local maxima) over all embeddings"),
    "braid_index": ("b", "braid index: minimal number of strands of a braid whose closure is the knot"),
    "braid_length": ("L", "minimal length of a braid word whose closure is the knot (as recorded by KnotInfo)"),
    "super_bridge_index": ("sb", "super bridge index: minimum over embeddings of the maximum number of local maxima in a direction"),
    "arc_index": ("alpha", "arc index: minimal number of pages of an arc presentation (equivalently, grid number)"),
    "polygon_index": ("p", "stick number: minimal number of edges of a polygonal representative"),
    "tunnel_number": ("t", "tunnel number"),
    "morse_novikov_number": ("MN", "Morse-Novikov number"),
    "signature": ("sigma", "knot signature, with the convention that the right-handed trefoil has sigma = -2"),
    "rasmussen_invariant": ("s", "Rasmussen s-invariant from Khovanov homology, right-handed trefoil s = 2"),
    "ozsvath_szabo_tau_invariant": ("tau", "Ozsvath-Szabo tau-invariant from knot Floer homology, right-handed trefoil tau = 1"),
    "epsilon": ("epsilon", "Hom's epsilon invariant"),
    "arf_invariant": ("Arf", "Arf invariant"),
    "determinant": ("det", "determinant |Delta(-1)|"),
    "nakanishi_index": ("nak", "Nakanishi index: minimal number of generators of the Alexander module"),
    "turaev_genus": ("g_T", "Turaev genus: minimal genus of the Turaev surface over all diagrams"),
    "td_clasp_number": ("cl", "clasp number: minimal number of clasp singularities of a singular disk in S^3 bounded by the knot"),
    "fd_clasp_number": ("cl_4", "4-dimensional clasp number: minimal number of double points of a normally immersed disk in B^4 bounded by the knot"),
    "double_slice_genus": ("g_ds", "doubly slice genus (as recorded by KnotInfo)"),
    "width": ("w", "Gabai width"),
    "volume": ("vol", "hyperbolic volume of the complement (0 for non-hyperbolic knots)"),
    "maximum_cusp_volume": ("cusp", "maximal cusp volume"),
    "longitude_length": ("lon", "length of the longitude in the maximal cusp"),
    "meridian_length": ("mer", "length of the meridian in the maximal cusp"),
    "chern_simons_invariant": ("CS", "Chern-Simons invariant of the complement (KnotInfo's normalisation)"),
    "volume_imaginary_part": ("Im vol", "imaginary part of the complex volume (KnotInfo's normalisation)"),
    "ropelength": ("rope", "ropelength (numerical estimate)"),
}
PROPERTIES = {
    "TRUE": "every prime knot (no hypothesis)",
    "alternating": "has an alternating diagram",
    "adequate": "has an adequate diagram",
    "fibered": "the complement fibres over the circle with the Seifert surface as fibre",
    "quasipositive": "is the closure of a braid that is a product of conjugates of positive generators",
    "strongly_quasipositive": "is the closure of a braid that is a product of positive band generators",
    "positive": "has a diagram with only positive crossings",
    "positive_braid": "is the closure of a positive braid",
    "quasi_alternating": "is quasi-alternating (recursive definition of Ozsvath-Szabo)",
    "almost_alternating": "is non-alternating and has a diagram that becomes alternating after one crossing change",
    "almost_strongly_qp": "KnotInfo's property 'almost strongly quasipositive' (see KnotInfo's definition)",
    "l_space": "is an L-space knot (has a positive Dehn surgery that is a Heegaard Floer L-space)",
    "cosmetic_crossing": "KnotInfo's cosmetic crossing property",
}
CHIRAL = {"positive_braid", "positive", "strongly_quasipositive", "quasipositive", "almost_strongly_qp", "l_space"}

BOUND = re.compile(r"^\((?P<h>[^)]*)\) → \((?P<t>\S+) (?P<op><=|>=) (?P<rhs>.*)\)$")


def parse(bound: str):
    m = BOUND.match(bound)
    terms, constant = [], Fraction(0)
    for part in m["rhs"].split(" + "):
        if "·" in part:
            c, f = part.split("·")
            terms.append((f, Fraction(c)))
        else:
            constant = Fraction(part)
    return m["h"], m["t"], m["op"], terms, constant


def symbol(col: str) -> str:
    return INVARIANTS.get(col, (col, ""))[0]


def plain(bound: str) -> str:
    """`g <= 1/2 c - 1/2` style, with symbols."""
    _, t, op, terms, k = parse(bound)
    parts = []
    for f, c in terms:
        coef = "" if c == 1 else "-" if c == -1 else f"{c} "
        parts.append(f"{coef}{symbol(f)}")
    if k != 0 or not parts:
        parts.append(str(k))
    rhs = " + ".join(parts).replace("+ -", "- ")
    return f"{symbol(t)} {op} {rhs}"


def latex(bound: str) -> str:
    _, t, op, terms, k = parse(bound)

    def sym(col: str) -> str:
        s = symbol(col).replace("^top", "^{\\mathrm{top}}")
        s = re.sub(r"_([A-Za-z0-9]+)", r"_{\1}", s)
        s = {"gamma": r"\gamma", "alpha": r"\alpha", "sigma": r"\sigma", "tau": r"\tau",
             "epsilon": r"\varepsilon"}.get(s, s)
        s = s.replace("gamma_", r"\gamma_").replace("Im vol", r"\operatorname{Im}\mathrm{vol}")
        return s if len(s) <= 2 or s.startswith("\\") else rf"\mathrm{{{s}}}"

    def frac(q: Fraction) -> str:
        if q.denominator == 1:
            return str(q.numerator)
        sign = "-" if q < 0 else ""
        return rf"{sign}\tfrac{{{abs(q.numerator)}}}{{{q.denominator}}}"

    parts = []
    for f, c in terms:
        coef = "" if c == 1 else "-" if c == -1 else frac(c) + r"\,"
        parts.append(coef + sym(f))
    if k != 0 or not parts:
        parts.append(frac(k))
    rhs = " + ".join(parts).replace("+ -", "- ")
    rel = r"\le" if op == "<=" else r"\ge"
    return f"{sym(t)} {rel} {rhs}"


def statement(group: dict) -> tuple[str, list[str]]:
    lines = []
    for h, kind in group["hypotheses"].items():
        who = "every prime knot" if h == "TRUE" else f"every prime knot that is {h.replace('_', ' ')}"
        members = [b for b in group["bounds"] if parse(b)[0] == h]
        if kind == "identity":
            le = next((b for b in members if parse(b)[2] == "<="), members[0])
            lines.append(f"For {who}: {plain(le).replace('<=', '=')}.")
        else:
            lines.append(f"For {who}: {plain(members[0])}.")
    return " ".join(lines), lines


def columns(group: dict) -> set[str]:
    cols = set()
    for b in group["bounds"]:
        _, t, _, terms, _ = parse(b)
        cols.add(t)
        cols.update(f for f, _ in terms)
    return cols


def main() -> int:
    p = argparse.ArgumentParser()
    p.add_argument("candidates", help="JSON from ragusa_a.candidate_checks")
    p.add_argument("--out", required=True)
    a = p.parse_args()

    data = json.loads(Path(a.candidates).read_text())
    by_bound = {r["bound"]: r for r in data["bounds"]}
    out = Path(a.out)
    if out.exists():
        shutil.rmtree(out)
    out.mkdir(parents=True)

    groups = data["groups"]
    used_cols = sorted(set().union(*(columns(g) for g in groups)) if groups else set())
    used_props = sorted({h for g in groups for h in g["hypotheses"]})

    records = []
    for g in groups:
        text, lines = statement(g)
        evidence = []
        for b in g["bounds"]:
            r = by_bound[b]
            full, mirror, comp = (r["checks"][k] for k in ("full_table", "mirrors", "catalogue_completed"))
            evidence.append({
                "bound": b, "plain": plain(b), "seeds_survived": r["seeds_survived"],
                "held_out_confirmed": r["held_out_confirmed"], "held_out_undecided": r["held_out_undecided"],
                "table_applicable": full["applicable"], "table_confirmed": full["confirmed"],
                "table_undecided": full["undecided"], "table_tight": full["tight"],
                "tight_examples": full["tight_examples"],
                "mirror_applicable": mirror["applicable"], "mirror_confirmed": mirror["confirmed"],
                "completed_undecided": comp["undecided"], "catalogue_class": r["novelty"],
            })
        records.append({"id": g["id"], "statement": text, "statements": lines,
                        "latex": [latex(b) for b in g["bounds"]], "hypotheses": g["hypotheses"],
                        "columns": sorted(columns(g)), "evidence": evidence})

    (out / "candidates.json").write_text(json.dumps({
        "source": "ragusa (github.com/carlok/ragusa), KnotInfo prime knots with 3 to 13 crossings",
        "engine_git_rev": data["engine_git_rev"], "candidates": records}, indent=1) + "\n")

    defs = "\n".join(f"| `{c}` | {INVARIANTS.get(c, (c, 'see KnotInfo'))[0]} | {INVARIANTS.get(c, (c, 'see KnotInfo'))[1]} |"
                     for c in used_cols)
    props = "\n".join(f"| `{h}` | {PROPERTIES.get(h, 'see KnotInfo')}{' (chirality-dependent)' if h in CHIRAL else ''} |"
                      for h in used_props)
    readme = f"""# Knot invariant conjectures from data: task for a language model

You are given {len(records)} candidate statements about knot invariants. Each was proposed by a
program (ragusa) that fits linear inequalities to a database, and then survived every automatic
check listed below. None has been reviewed by a person. They may be known theorems, consequences of
known theorems, false, or open. Your job is to decide which, for each one, as rigorously as you can.

## Scope of each statement

Each candidate is a claim **for all prime knots** with a given property (or all prime knots), not
only for the knots in the database. Statements were found on prime knots with at most 13 crossings,
so treat them as conjectures about all prime knots with that property. Say separately if a statement
also holds, or fails, for composite knots.

## What the program already checked

1. The inequality holds on every prime knot with 3 to 13 crossings in KnotInfo (https://knotinfo.org)
   that has the property, whenever the invariants are known exactly or up to an interval (a
   statement counts as violated only if it fails for every value in the recorded intervals).
2. It was learned on 80% of the knots and never failed on the other 20%, in at least 3 of 5 random
   splits, and it was confirmed on at least one held-out knot.
3. It is not implied by the catalogue of inequalities in M. Jabłonowski, *Integer knot invariants:
   inequalities, computations, and open problems*, arXiv:2605.22652 (v4), via transitive closure of
   its relations with the obvious linear bounds.
4. It holds on the mirror images of the knots (KnotInfo stores one chirality per knot): signature,
   s, tau and epsilon were negated; chirality-dependent properties were treated as unknown there.
5. It is not refuted after filling unknown values with intervals implied by that catalogue.
6. The knots where it is attained are not all at the largest crossing number (13), and the property
   is recorded for 13-crossing knots.
7. It is not a constant bound and does not involve an invariant with at most three distinct values.

These checks remove many artefacts, not all. In particular, the database stops at 13 crossings, so
families such as torus knots T(2,n), T(3,n), pretzel knots and cables beyond 13 crossings are the
first places to look for counterexamples.

## Conventions

- KnotInfo's chirality for each knot; the right-handed trefoil (3_1 in the table) has signature -2,
  Rasmussen s = 2, tau = 1. So for positive knots s = 2 tau = 2g > 0 and the signature is negative.
- Knot names such as `12n_242` are KnotInfo names; KnotInfo lists one of each mirror pair.

## Invariants that occur

| KnotInfo column | symbol | definition |
|---|---|---|
{defs}

## Properties that occur

| KnotInfo column | meaning |
|---|---|
{props}

## What to return

For each candidate, in order, return exactly this block (Markdown):

```
### Kxx
verdict: PROVED | REFUTED | KNOWN | IMPLIED | OPEN
statement as you understand it: <one line; point out any ambiguity in the definitions>
argument: <proof, or derivation from known results, or explanation of why it is open>
counterexample: <knot, its relevant invariant values, and how each value is known; or "none">
references: <author, title, venue, year, theorem or page; only sources you are confident exist>
confidence: high | medium | low
what would settle it: <for OPEN or low confidence: the computation, family or source to check>
```

- KNOWN: stated in the literature (give the exact reference). IMPLIED: follows from known results by
  an argument you give. PROVED: your own complete proof. REFUTED: an explicit counterexample or a
  proof of the negation.
- Do not invent references. If you are unsure a paper states something, say so and mark confidence low.
- Invariant values of specific knots must say where they come from (a theorem, a computation you
  show, or a database you name).

## Files

- `candidates.md`: all candidates with their evidence; `Kxx.md`: one per candidate.
- `candidates.json`: the same, machine-readable.
- `candidates.pdf` (`candidates.tex`): the same as a document.
"""
    (out / "README.md").write_text(readme)

    def candidate_md(rec: dict) -> str:
        lines = [f"## {rec['id']}", "", f"**Statement.** {rec['statement']}", "",
                 "**Bounds as the program printed them** (hypothesis → inequality on KnotInfo columns):", ""]
        lines += [f"- `{e['bound']}`" for e in rec["evidence"]]
        lines += ["", "**Evidence on KnotInfo, prime knots with 3 to 13 crossings:**", "",
                  "| bound | seeds (of 5) | held-out knots confirmed / undecided | table: knots with the property | confirmed | undecided | attained on | examples attained | mirrors confirmed | catalogue class |",
                  "|---|---:|---|---:|---:|---:|---:|---|---:|---|"]
        for e in rec["evidence"]:
            lines.append(f"| {e['plain']} | {e['seeds_survived']} | {e['held_out_confirmed']} / {e['held_out_undecided']} | "
                         f"{e['table_applicable']} | {e['table_confirmed']} | {e['table_undecided']} | {e['table_tight']} | "
                         f"{', '.join(e['tight_examples'])} | {e['mirror_confirmed']} | {', '.join(e['catalogue_class'])} |")
        lines += ["", "Held-out counts add up over the five splits, so a knot can be counted up to five times. "
                  "`undecided`: some invariant is unknown or only an interval that does not decide the inequality.", ""]
        return "\n".join(lines)

    body = "\n".join(candidate_md(r) for r in records)
    (out / "candidates.md").write_text(f"# Candidates ({len(records)})\n\nSee README.md for the task, scope, "
                                       f"conventions and answer format.\n\n{body}")
    for r in records:
        (out / f"{r['id']}.md").write_text(f"# {r['id']}\n\nSee README.md for the task, scope, conventions and "
                                           f"answer format.\n\n{candidate_md(r)}")

    def tex_escape(s: str) -> str:
        return (s.replace("\\", r"\textbackslash{}").replace("_", r"\_").replace("&", r"\&")
                .replace("%", r"\%").replace("#", r"\#").replace("→", r"$\to$"))

    items = []
    for r in records:
        rows = " \\\\\n".join(
            rf"${latex(e['bound'])}$ & {tex_escape(parse(e['bound'])[0])} & {e['seeds_survived']} & "
            rf"{e['table_confirmed']} & {e['table_undecided']} & {e['table_tight']} & {e['mirror_confirmed']}"
            for e in r["evidence"])
        items.append(rf"""\subsection*{{{r['id']}}}
{tex_escape(r['statement'])}

\smallskip
\begin{{tabular}}{{lllrrrr}}
\toprule
inequality & hypothesis & seeds & confirmed & undecided & attained & mirrors \\
\midrule
{rows} \\
\bottomrule
\end{{tabular}}
""")
    symbols = " \\\\\n".join(rf"\texttt{{{tex_escape(c)}}} & {tex_escape(INVARIANTS.get(c, (c, ''))[0])} & {tex_escape(INVARIANTS.get(c, (c, 'see KnotInfo'))[1])}"
                             for c in used_cols)
    tex = rf"""\documentclass[10pt]{{article}}
\usepackage[margin=2cm]{{geometry}}
\usepackage{{amsmath,amssymb,booktabs,longtable}}
\usepackage[T1]{{fontenc}}
\sloppy
\title{{Knot invariant conjectures from data\\\large {len(records)} candidates for checking}}
\date{{}}
\begin{{document}}
\maketitle
Each statement is a claim for all prime knots with the given property. It was fitted by the ragusa
engine on KnotInfo (prime knots with 3 to 13 crossings), survived held-out checks, mirror images,
catalogue completion of unknown values and range-edge checks, and is not implied by Jab\l{{}}onowski's
catalogue (arXiv:2605.22652). Conventions: the right-handed trefoil has $\sigma=-2$, $s=2$, $\tau=1$.
The task and answer format are in \texttt{{README.md}}. Columns: \emph{{confirmed}} and
\emph{{undecided}} count knots with the property up to 13 crossings; \emph{{attained}} counts knots
where equality holds; \emph{{mirrors}} counts mirror images confirmed.

\section*{{Invariants}}
\begin{{longtable}}{{lp{{2.2cm}}p{{10cm}}}}
\toprule
column & symbol & definition \\
\midrule
{symbols} \\
\bottomrule
\end{{longtable}}

\section*{{Candidates}}
{"".join(items)}
\end{{document}}
"""
    (out / "candidates.tex").write_text(tex)
    done = subprocess.run(["latexmk", "-pdf", "-interaction=nonstopmode", "-quiet", "candidates.tex"],
                          cwd=out, capture_output=True, text=True)
    subprocess.run(["latexmk", "-c"], cwd=out, capture_output=True)
    pdf = (out / "candidates.pdf").exists()
    print(f"dossier: {len(records)} candidates -> {out} (pdf {'ok' if pdf else 'FAILED: ' + done.stdout[-400:]})")
    return 0 if pdf else 1


if __name__ == "__main__":
    raise SystemExit(main())
