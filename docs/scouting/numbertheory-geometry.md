# Scouting: number theory, topology and geometry

Source: research-agent survey (web pages only), with load-bearing facts re-checked on
2026-09-14. Marks as defined in [README.md](README.md): **checked**, **reported**, **unverified**.

## 1. KnotInfo — agent's first choice

| | |
|---|---|
| where | https://knotinfo.org (Livingston, Moore; developer Ost) — reported |
| objects | 12,965 non-trivial prime knots up to 13 crossings — **checked** (abstract of arXiv:2605.22652); PD notation downloadable for prime knots through 16 crossings — **checked** (download page). The PyPI snapshot `database-knotinfo` reports 2,979 knots and 4,189 links — reported, conflict with 12,965 unexplained |
| invariants | "well over 100" invariant categories; integer invariants (signature, τ, s, slice and 4-genus, unknotting number, bridge and braid index, Turaev genus, …), hyperbolic volume, polynomials (Alexander, Jones, HOMFLY, seven Khovanov variants, Heegaard Floer) — reported |
| booleans | alternating, fibered, quasipositive, slice, amphichiral, … — reported |
| access | invariants as a zipped Excel spreadsheet (~16 MB, two columns per invariant), polynomial invariants (8 MB), PD notation (40 MB zipped, 382 MB unzipped), 3D coordinates, diagrams — **checked** (https://knotinfo.org/homelinks/database_download.php) |
| licence | none stated on the download page — **checked**; the Python package is GPL — reported; settled with the maintainers on 2026-10-02: no copyright, cite KnotInfo, do not repost the database |

**Partial oracle — the strongest of any candidate:** Michał Jabłonowski, *Integer Knot
Invariants: Inequalities, Computations, and Open Problems* (arXiv:2605.22652, 21 May 2026) gives
"a directed graph consisting of 47 inequalities between 33 knot invariants" and propagates known
values "for all 12965 non-trivial prime knots up to 13 crossings" — **checked** (abstract). That is
a catalogue of known results in ragusa's own statement language, over the same objects. The agent
reports that one open candidate inequality in it is equivalent to the smooth Slice–Ribbon
Conjecture (reported).

**Literature (reported):** Livingston on τ (arXiv:math/0311036); Dasbach–Lowrance, Turaev genus
bounds (arXiv:1002.0898); Lackenby's twist-number volume bound (arXiv:math/0012185).

**Prior ML work (reported):** Davies, Juhász, Lackenby et al., DeepMind signature–slope inequality
(Nature 2021); learned volume from the Jones polynomial (arXiv:1902.05547).

**Graffiti3 on knots — checked.** Davila's public repository `RandyRDavila/TxGraffiti2` (MIT) holds
`papers/graffiti3/Graffiti3_Knot_Theory_Experiment.ipynb`. It loads knots from pyknotid's
catalogue with `MAX_CROSSINGS = 12`, prime only, shape (2978, 12), columns min_crossings,
determinant, signature, unknotting_number, fibered, vassiliev_2, vassiliev_3, hyperbolic_volume,
symmetry, composite (several dropped afterwards). So the domain is touched, but with about ten
columns from a different source, no interval handling and no known-inequality oracle; KnotInfo's
40-plus invariants and Jabłonowski's catalogue are not used.

**Criteria:** 1 meets on content, licence unclear · 2 meets · 3 meets · 4 meets, with a structured
partial oracle.

**Risks:** hard invariants (4-genus, unknotting number) are often intervals or missing, and ragusa's
generators currently reject missing values; the data licence needs asking (since settled, 2026-10-02; cite as `knotinfo` in `CITATIONS.bib`); possible overlap with
Graffiti3.

## 2. Toric canonical Fano 3-folds (Kasprzyk, Graded Ring Database) — agent's second choice

| | |
|---|---|
| where | https://zenodo.org/records/5866330 ; http://www.grdb.co.uk/forms/toricf3c |
| objects | 674,688 isomorphism classes: all 3-dimensional convex lattice polytopes with exactly one interior lattice point — **checked** |
| fields | id, num_vertices, num_faces, num_points, is_terminal, is_simplicial, is_regular, is_reflexive, vertex_list, point_list, dual_list, ehrhart_delta, hilbert_delta, normal_form, volume, degree, gorenstein_index, h1–h10, e1–e10, picard_rank, automorphism_order, is_barycentre_zero, is_dual_barycentre_zero — **checked** (≈ 28 numeric, 6 boolean) |
| access | `toricf3c.txt` 817.1 MB, `toricf3c.sql` 563.7 MB — **checked** |
| licence | CC0 1.0 — **checked** |

**Literature (reported):** Kasprzyk, *Canonical toric Fano threefolds* (Canad. J. Math.);
Stapledon's inequalities for Ehrhart δ-vectors (arXiv:0801.0873, arXiv:0904.3035).
**Prior ML work (reported):** ML of Fano dimension (Nature Comms 2023), terminal singularity
detection (arXiv:2310.20458). No Graffiti-style work found.

**Criteria:** 1 meets · 2 meets (ready columns, few booleans) · 3 meets · 4 meets (partial oracle:
the Ehrhart/h*-inequality literature only).

**Risks:** linear h* bounds are often theorems already; only six booleans for hypotheses. At
674,688 rows it is the scale test ragusa was designed for.

## 3. Hyperbolic 3-manifold censuses (SnapPy, Regina) — agent's third choice

Orientable cusped census to 10 ideal tetrahedra, 11,031 closed manifolds, 1.2 million non-alternating
and 491,327 alternating knot exteriors; a new 10-tetrahedron census under CC BY 4.0 — all reported.
Invariants (volume, systole, cusp shapes, homology, symmetry group) are computed with SnapPy and are
floating-point — reported.
**Criteria:** 1 meets · 2 partly (compute, floating point) · 3 meets · 4 meets, no partial oracle.
**Risks:** verified numerics needed; novelty judged by hand.

## 4. LMFDB arithmetic collections

Elliptic curves over ℚ (3,824,372), number fields (22,444,816), genus-2 curves (66,158), classical
newform orbits (1,141,510), Artin representations (662,882) — reported; data CC BY-SA 4.0 —
**checked** (see [algebra.md](algebra.md)).
**Criteria:** 1 meets · 2 meets · 3 meets · 4 meets, no partial oracle.
**Risk:** the key relations are logarithmic and range-dependent (regulator against discriminant),
so linear bounds on a truncated table tend to be artefacts of the cut-off.

## 5. Kreuzer–Skarke reflexive 4-polytopes

473,800,776 polytopes, about 7 numeric columns and no booleans in the Hugging Face Parquet copy
(15.8 GB, CC BY-SA 4.0) — reported.
**Criteria:** 2 partly (no booleans) · 3 partly. Size without the hypotheses ragusa needs.

## 6. CICY three- and four-folds (Oxford)

7,890 threefolds, 921,497 four-folds — reported; no licence stated — reported.
**Criteria:** 1 partly (licence) · 3 partly.

## 7. polyDB

Collections of lattice polytopes and small matroids behind polymake; the servers were unreachable
during scouting, so counts and licence are unverified.

## Corrections to the agent's report

- The agent says TxGraffiti ships no polytope or Calabi–Yau tables. The PyPI package txgraffiti
  0.4.1 does ship `polytope_data` (496 rows) and `calabi_yau_data` (175 rows): gate G-B1 loads both.
- "Graffiti3 was run on knots, finite groups and Calabi–Yau data" is **confirmed** by the notebooks
  in `RandyRDavila/TxGraffiti2/papers/graffiti3/` (Calabi–Yau, group theory, knot theory, plus
  `calabi_yau_experiment.pdf`). The Calabi–Yau experiment uses weighted-projective-space
  hypersurface data from `TomasSilva/MLcCY7` (weights, h11, h21, Euler characteristic), not the
  toric canonical Fano 3-folds. The Graffiti³ preprint itself was not found on arXiv; a third-party
  repository refers to "version 1 of Randy Davila's Graffiti³ preprint".
