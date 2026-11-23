# Scouting: finite algebra

Source: research-agent survey (web pages only), with load-bearing facts re-checked on
2026-09-14. Marks as defined in [README.md](README.md): **checked**, **reported**, **unverified**.

## 1. LMFDB abstract groups — agent's first choice

| | |
|---|---|
| where | https://www.lmfdb.org/Groups/Abstract/ ; data described in arXiv:2409.09189 (reported) |
| objects | 544,831 groups, 275,339,797 subgroups, 39,901,075 irreducible complex characters; largest is S₄₇ — **checked** (main page) |
| completeness | all groups of order ≤ 2000 except orders > 500 divisible by 128; all groups with a faithful transitive permutation representation of degree ≤ 47 (degree 32 excepted in an order range); faithful permutation degree ≤ 15; rational matrix dimension ≤ 6; perfect groups of order ≤ 50,000; groups of Lie type and sporadic groups within stated bounds — **checked** (https://www.lmfdb.org/Groups/Abstract/Completeness) |
| numeric invariants (stored, searchable) | order, exponent, automorphism group order, outer automorphism group order, nilpotency class, derived length, rank, commutator length, minimum transitive degree, minimum permutation degree, numbers of subgroups / normal subgroups / characteristic subgroups / conjugacy classes / automorphism classes / divisions, minimum degree of complex and of rational irreps — 18, **checked** (main page) |
| boolean properties | abelian, cyclic, nilpotent, supersolvable, solvable, perfect, simple, almost simple, quasisimple, metabelian, metacyclic, A-group, Z-group, monomial, rational, direct product, semidirect product, wreath product — 18, **checked** (main page) |
| cheaply derived | commuting probability k(G)/\|G\|, \|Z(G)\|, subgroup-lattice counts — reported |
| access | JSON/YAML API (100 per page, about 10,000 per query) — reported (https://www.lmfdb.org/api/); read-only PostgreSQL mirror `devmirror.lmfdb.xyz` — reported (LMFDB GettingStarted.md); Python client lmfdb-lite — reported |
| licence | data CC BY-SA 4.0; site code GPL v2; LMFDB asks bulk users to prefer efficient access over scraping and to rate-limit — **checked** (`lmfdb/templates/license.html` in the LMFDB GitHub repository; the rendered page was blank) |
| citation | "The LMFDB Collaboration, *The L-functions and modular forms database*, https://www.lmfdb.org, <year>, [Online; accessed <date>]" — **checked** (https://www.lmfdb.org/citation) |
| download size | unverified |

**Literature (reported):**
- Guralnick & Robinson, *On the commuting probability in finite groups*, J. Algebra 300 (2006).
- Lower bounds on the number of conjugacy classes (Pyber; Héthelyi–Külshammer; Maróti): arXiv:0708.2281, arXiv:2307.05579.
- Cameron, *Graphs defined on groups*, arXiv:2102.11177: power, commuting and enhanced power graphs, whose graph invariants ragusa's graph pipeline already handles.

**Prior automated or ML work (reported):** He et al., *Learning to be Simple* (arXiv:2312.05299);
He & Kim, *Learning algebraic structures* (arXiv:1905.02263). An AlphaEvolve group-theory claim is
unverified.

**Graffiti-style work exists — checked, contradicting the agent's "none found".**
`RandyRDavila/TxGraffiti2/papers/graffiti3/Graffiti3_Group_Theory_Experiment.ipynb` exports GAP
SmallGroups invariants to a CSV and runs Graffiti3 on targets such as `nrConjClasses` and the
commuting probability, with ratio, LP, polynomial, square-root, logarithmic and exponential
templates. Five of its published conjectures (C20, C21, C22, C27, C28) have explicit counterexample
families: U₇(q) for every prime power q, and dihedral groups D₂ₙ for n ≥ 32 a power of two or
n ≥ 25 odd (`ZackaryLoevseth/graffiti3-counterexample-families`, public preprint release, August
2026, not peer reviewed). Statements true on a small group table failed on larger groups: the
sampling-bias risk in its plainest form.

**Partial oracle (reported):** Kourovka Notebook (arXiv:1401.0300, open problems); Groupprops wiki
(CC BY-SA). There is no machine-readable catalogue of known inequalities.

**Criteria:** 1 meets · 2 meets · 3 meets · 4 meets (oracle partial and unstructured).

**Main risk:** rediscovery. Group theory is deeply mined; simple linear bounds between standard
invariants are likely known, and the real theorems often have log or square-root shapes that
ragusa's linear templates cannot state. Novelty checks would be manual.

## 2. GAP SmallGroups library

| | |
|---|---|
| where | https://docs.gap-system.org/pkg/smallgrp/doc/chap1.html |
| objects | 423,164,062 groups of order ≤ 2000 except 1024; plus cubefree orders ≤ 50,000 (395,703), orders p⁷ for p = 3, 5, 7, 11 (907,489), pⁿ for n ≤ 6, and other families — **checked** |
| invariants | none stored; everything computed in GAP — reported |
| licence | Artistic-2.0 — reported |

**Criteria:** 1 meets · 2 partly (compute, not stored) · 3 meets · 4 meets.
**Risk:** counts dominated by 2-groups; computing invariants for hundreds of millions of groups is
an engineering project. LMFDB is the precomputed layer on top of this library.

## 3. GroupNames (Dokchitser)

16,309 groups of order ≤ 500 except 256 and 384, with character tables and subgroup lattices —
reported. HTML only; bulk download and licence unverified. A subset of LMFDB; useful for
cross-checks only. **Criteria:** 1 fails/partly · 2 meets · 3 meets · 4 meets.

## 4. Finite semigroups (GAP smallsemi)

Semigroups up to isomorphism and anti-isomorphism by order: 1, 4, 18, 126, 1160, 15,973, 836,021,
1,843,120,128 (orders 1–8) — reported (smallsemi manual ch. 1). About 26 boolean properties
(band, inverse, regular, orthodox, Clifford, …) and a few numeric attributes — reported.
GPL-3.0-or-later — reported. The agent found little literature on inequalities between invariants
of arbitrary small semigroups; order 8 is dominated by nilpotent semigroups of degree 3.
**Criteria:** 1 meets · 2 meets (boolean) / partly (numeric) · 3 partly · 4 meets.
**Risk:** new but uninteresting output.

## 5. Quasigroups and loops (GAP LOOPS; McKay's data)

Class libraries (Moufang loops to order 64 plus 81 and 243, left Bol loops below 17, …) and all
loops of order ≤ 7 — reported. Rich numeric invariants (nucleus, centre, multiplication group) and
boolean varieties (Moufang, Bol, Bruck, flexible, …) computed by LOOPS — reported. GPL-2.0-or-later
— reported. Open questions exist (e.g. odd-order Moufang loops with trivial nucleus) — reported.
**Criteria:** 1 partly · 2 meets · 3 meets (implications) / partly (inequalities) · 4 partly.
**Risk:** the implication side is ragusa's magma validation again, largely decidable with
Prover9/Mace4; numeric bounds within a class are where anything new would be.

## 6. Small rings

GAP holds rings of order ≤ 15 only; DaRT (MIT licence) catalogues about 217 mostly infinite rings
— reported. **Criteria:** 1 fails. Too few finite objects.

## 7. Posets (McKay) — agent's second choice

| | |
|---|---|
| where | https://users.cecs.anu.edu.au/~bdm/data/digraphs.html |
| objects | Hasse diagrams of all posets on n points: 1, 2, 5, 16, 63, 318, 2045, 16,999, 183,231, 2,567,284 (n = 1…10), as `hasse<n>.txt[.gz]` — **checked** |
| licence | none on the poset page itself, but the collection's index states that McKay releases all data files (not the HTML pages) under CC BY 4.0 — **checked** (https://users.cecs.anu.edu.au/~bdm/data/) |
| invariants | width, height, linear extensions, antichains, dimension, jump number, balance constant; booleans graded, series-parallel, N-free, interval order, lattice, distributive, modular — all cheap at n ≤ 10, reported |
| literature | the 1/3–2/3 conjecture (open since 1968; balance constant ≥ 1/3), e.g. Olson–Sagan arXiv:1706.04985 — reported |
| prior automated work | none found — reported |

**Criteria:** 1 meets · 2 meets · 3 meets · 4 meets.
**Risk:** n ≤ 10 may hide large-n behaviour; no invariants are stored. Posets are also covered in
[combinatorics.md](combinatorics.md).

## Agent's shortlist

1. LMFDB abstract groups.
2. Posets on at most 10 points.
3. Loops and quasigroups.

## Notes from checking

- LMFDB's completeness claims, invariant lists and licence all hold on its own pages, and
  CC BY-SA 4.0 suits a public repository (share-alike applies to the data we redistribute).
- LMFDB asks for efficient access and rate limiting; any load test should use the read-only
  mirror or bulk access, never scraping.
- The poset page states no licence, but the collection index releases all McKay data files under
  CC BY 4.0 (corrected after first writing this page, which said reuse needed permission).
