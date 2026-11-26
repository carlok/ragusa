# Scouting: combinatorics

Source: research-agent survey (web pages only), with load-bearing facts re-checked on
2026-09-14. Marks as defined in [README.md](README.md): **checked**, **reported**, **unverified**.

## 1. Matroids on at most 9 elements (Mayhew–Royle) — agent's first choice

| | |
|---|---|
| where | https://zenodo.org/records/6825419 ; Mayhew & Royle, *Matroids with nine elements*, JCTB 2008 (arXiv:math/0702316) |
| objects | "the 385370 pairwise non-isomorphic matroids from 0 to 9 elements inclusive" — **checked**; by size 1, 2, 4, 8, 17, 38, 98, 306, 1724, 383,172 (OEIS A055545) — reported, sums to 385,370. The table is almost entirely 9-element matroids |
| access | `matroids09_rankLine` 197.1 MB plus `readme.md` — **checked**; SageMath reads it (`matroids.AllMatroids()`) — reported |
| licence | CC BY 4.0 — **checked** |
| invariants | none stored; cheap at 9 elements from the rank function: rank, numbers of bases, circuits, flats and hyperplanes, girth, cogirth, Tutte evaluations T(1,1), T(2,0), T(0,2), T(2,2), β and Möbius invariants, f- and h-vector entries; booleans binary, ternary, regular, graphic, cographic, paving, sparse paving, self-dual, connected, 3-connected, has a U(2,4) minor — reported, 30–60 in total |

**Literature:**
- Beke, Csáji, Csikvári, Pituk, *The Merino–Welsh conjecture is false for matroids*
  (arXiv:2311.01932, 2023) — **checked**: "We show a counter-example for these inequalities."
- The inequalities still hold for split matroids (arXiv:2204.07132) — reported. Class-conditioned
  versions of a failed inequality are exactly ragusa's "bound under a boolean hypothesis".
- Adiprasito–Huh–Katz log-concavity (arXiv:1511.02888); De Loera–Kemper–Klee on Stanley's h-vector
  conjecture over all matroids with at most 9 elements (arXiv:1106.2576); Oxley's *Matroid Theory*
  closes with sixty unsolved problems — reported.

**Prior automated work:** none found for Graffiti-style conjecturing — reported.
**Partial oracle:** no catalogue; a small hand-curated one is feasible (Oxley's problems,
log-concavity and Tutte-evaluation results, excluded-minor characterisations) — reported.

**Criteria:** 1 meets · 2 meets once computed · 3 meets · 4 meets, weak partial oracle.

**Risks:** 9-element evidence can mislead (the Merino–Welsh counterexamples are large), so
survivors need checking against infinite families (spikes, swirls, projective geometries).
Computing the invariants needs SageMath or our own rank-function code.

## 2. Posets (McKay; Brinkmann–McKay) — agent's second choice

| | |
|---|---|
| objects | all posets on n points as Hasse diagrams: 16,999 (n = 8), 183,231 (n = 9), 2,567,284 (n = 10) — **checked** (https://users.cecs.anu.edu.au/~bdm/data/digraphs.html) |
| licence | McKay releases all data files of the collection under CC BY 4.0 (not the HTML pages; data compiled by others may differ) — **checked** (https://users.cecs.anu.edu.au/~bdm/data/) |
| invariants | none stored; height, width, linear extensions, jump number, dimension (costly), balance constant, antichains, ideals; booleans N-free, series-parallel, interval order, semiorder, lattice, distributive, graded, width 2 — reported |

**Literature (reported):** the 1/3–2/3 conjecture (Olson–Sagan survey, arXiv:1706.04985) and its
class-restricted versions (width two, arXiv:1811.01500; N-free, arXiv:1107.5626).
**Prior automated work:** none found — reported.

**Criteria:** 1 meets · 2 meets once computed · 3 meets · 4 meets, no catalogue.
**Risks:** everything must be computed; the famous conjecture is a constant bound on one
invariant, which ragusa's language covers only as a degenerate case.

## 3. Graphs — only as a niche (agent's third choice)

House of Graphs (over 29,000 graphs, 52 invariants, REST API), McKay's censuses (12,005,168 graphs
on 10 vertices; CC BY 4.0) and PHOEG (all graphs to order 10) — reported.
**Crowded:** AutoGraphForge (arXiv:2609.03478) already runs generation, refutation against 348,207
graphs (as reported at scouting time; the run report quoted in PROMPT.md §1.2 ends with 348530) and Lean proving, with a 559-relation novelty filter (**checked** in Sprint 1: 559 entries).
Worth it only for classes and invariants others skipped (hypohamiltonian, planar or chordal
censuses). Graphs remain ragusa's control, not its target.

## 4. Strongly regular and distance-regular graphs; block designs

Brouwer's parameter tables, DistanceRegular.org, DesignTheory.org — reported. Parameters are
functionally dependent and the literature is about existence rather than inequalities.
**Criteria:** 2 fails or partly · 3 partly. Not pursued.
