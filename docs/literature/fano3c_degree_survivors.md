# Literature check: degree inequalities on canonical toric Fano 3-folds

Scope: 14 inequalities that ragusa found to hold on every row of Kasprzyk's classification
(674,688 polytopes, Zenodo 5866330). Checked 2026-09-15.

## Method

**Local files read.** `docs/scouting/loadtest-fano.md`, `calibration/src/ragusa_a/export_fano.py`, and the dataset
`external/fano3c/README.txt`. Column meanings as used below:

- `degree` = (-K)^3 = normalised volume of the dual polytope P\*.
- `h_k` = #(kP\* ∩ M) = h^0(-kK).
- `e_k` = #(kP ∩ N).
- `ehrhart_a1`, `ehrhart_a2` are the middle entries of the h\*-vector (1, a1, a2, 1) of P. So
  a1 = e1 - 4 = num_points - 3, Vol(P) = a2 + a1 + 2, and e2 = 10 + 4·a1 + a2.

**Data re-check (read-only).** Every statement was recomputed with exact rationals over the SQLite import
(`external/fano3c/toricf3c.db`; throwaway scripts, not in the repository). All 14 have 0 violations and are tight (slack 0)
on at least one row. Per-class ranges quoted below come from this exact recomputation. Caution: the
database stores `degree` as text, so SQL `CAST(degree AS REAL)` truncates fractions such as `997/60`
and must not be used.

**Web queries** (among others):

- "Kasprzyk Canonical toric Fano threefolds maximum degree 72"
- "Prokhorov degree of Fano threefolds with canonical Gorenstein singularities 72"
- "Reid Young person's guide plurigenus formula h^0(-kK)"
- "Kawamata boundedness Q-Fano -K·c2 ≥ 0 r - 1/r 24"
- "minimum volume dual polytope one interior lattice point"
- "canonical Fano polytope maximum number of vertices 16"
- "finite subgroups of GL(3,Z) Tahara"
- "Berman Berndtsson toric Fano barycenter"
- "Liu volume singular Kähler-Einstein Fano (n+1)^n"
- "vol(P) vol(P\*) inequality one interior point"
- "Hibi lower bound theorem Ehrhart"
- "smallest anticanonical degree toric canonical Fano 3-fold"

**Sources consulted.** "verified" means I read the statement in the source text, not only a
search snippet.

| ref | source | what was verified |
|---|---|---|
| [K] | A. M. Kasprzyk, *Canonical toric Fano threefolds*, Canad. J. Math. 62(6) (2010) 1293–1309; arXiv:0806.2604 (v2 text read), https://arxiv.org/abs/0806.2604 | Theorem 3.6 (arXiv v2 numbering): (-K)^3 ≤ 72, equality only for P(1,1,1,3) and P(1,1,4,6). The proof uses minimal polytopes with vol Q^∨ ≤ 12. **No** minimum degree, no vertex-count bound and no automorphism statement found in the text. |
| [Z] | Kasprzyk, dataset README, doi:10.5281/zenodo.5866330, https://zenodo.org/records/5866330 | Column definitions. degree = Vol(P\*). h_i = h^0(-iK). |
| [P] | Yu. G. Prokhorov, *On the degree of Fano threefolds with canonical Gorenstein singularities*, Sb. Math. 196(1) (2005) 81–122; arXiv:math/0405347, https://arxiv.org/abs/math/0405347v4 | -K^3 ≤ 72 for canonical Gorenstein Fano 3-folds, quoted by [K] as Theorem 3.7 with the same equality cases. |
| [JZZ] | C. Jiang, T. Zhang, Y. Zou, *A canonical Fano threefold has degree ≤ 72*, arXiv:2510.06856 (2025), https://arxiv.org/abs/2510.06856 | Abstract: the anticanonical volume of a canonical weak Fano 3-fold is at most 72, and the bound is optimal. This extends 72 beyond the Gorenstein and toric cases. |
| [BKN] | G. Balletti, A. M. Kasprzyk, B. Nill, *On the maximum dual volume of a canonical Fano polytope*, Forum Math. Sigma; arXiv:1611.02455, https://arxiv.org/abs/1611.02455 | Sharp upper bound on vol(P\*) in every dimension (abstract). |
| [CC] | J. A. Chen, M. Chen, *An optimal boundedness on weak Q-Fano threefolds*, arXiv:0712.4356, https://arxiv.org/html/0712.4356 | §2.1 Reid's formula for terminal weak Q-Fano 3-folds: P_{-n} = n(n+1)(2n+1)/12·(-K^3) + (2n+1) - l(n+1), where l(n+1) = Σ_i Σ_{j=1}^{n} \bar{jb_i}(r_i - \bar{jb_i})/(2r_i). §2.8: -K·c2 ≥ 0 by [KMMT], which gives Σ_i (r_i - 1/r_i) ≤ 24 for the basket. |
| [KMMT] | J. Kollár, Y. Miyaoka, S. Mori, H. Takagi, *Boundedness of canonical Q-Fano 3-folds*, Proc. Japan Acad. 76A (2000) 73–77 | Cited by [CC] for -K·c2 ≥ 0. Not read directly. |
| [R] | M. Reid, *Young person's guide to canonical singularities*, Proc. Sympos. Pure Math. 46 (1987) 345–414 | Cited by [CC, 10.3] for the singular Riemann–Roch formula. Not read directly. |
| [BB] | R. Berman, B. Berndtsson, *Real Monge-Ampère equations and Kähler-Ricci solitons on toric log Fano varieties*, Ann. Fac. Sci. Toulouse Math. (2013), http://www.numdam.org/item/AFST_2013_6_22_4_649_0/ | Abstract: for toric log Fano varieties, a Kähler–Einstein metric exists iff "0 is the barycenter of P" (P is the moment polytope, i.e. P\* here). |
| [L] | Y. Liu, *The volume of singular Kähler–Einstein Fano varieties*, Compositio Math. 154 (2018) 1131–1158; arXiv:1605.01034 (text read) | Introduction, citing Fujita [Fuj15, Cor. 1.3]: a Kähler–Einstein Q-Fano n-fold has (-K)^n ≤ (n+1)^n. Theorem 36: Ding-semistable with (-K)^n ≥ (n+1)^n implies X ≅ P^n. |
| [NP] | B. Nill, A. Paffenholz, *On the equality case in Ehrhart's volume conjecture*, arXiv:1205.1270 | Abstract: Ehrhart's volume conjecture (barycentre = unique interior lattice point) holds for reflexive polytopes (Berman–Berndtsson), and the equality case is characterised. |
| [T] | K. Tahara, *On the finite subgroups of GL(3,Z)*, Nagoya Math. J. (1971); Ascher–Grimmer comment, https://www.cambridge.org/core/journals/nagoya-mathematical-journal/article/comment-on-a-paper-by-tahara-on-the-finite-subgroups-of-gl3z/5A433CF2934D779C781BA79C399FED5A | 73 conjugacy classes of finite subgroups (after the Ascher–Grimmer correction). Maximal order 48. Verified from search snippets only, not the full text. |
| [H] | T. Hibi, *A lower bound theorem for Ehrhart polynomials of convex polytopes*, Adv. Math. 105 (1994) 162–165 | Citation verified. Content (h\*_1 ≤ h\*_i when there is an interior point) is from memory and was **not** verified from the source. It is not used below. |

**Facts used below and how each was established:**

- **Maximum degree 72.** From literature ([K] Thm 3.6; [P] Gorenstein; [JZZ] general). Also confirmed on the data.
- **Minimum degree 4.** Data only; literature: not found. Attained by a reflexive tetrahedron, id 547390.
- **Kähler–Einstein bound.** Rows with `is_dual_barycentre_zero` have degree ≤ 64. By [BB] these rows are the
  Kähler–Einstein ones, so ≤ 64 follows from [L]/Fujita. It holds on the data. The max 64 is attained at aut 24,
  consistent with P^3 and [L, Thm 36].
- **Barycentre of P zero.** Rows with `is_barycentre_zero` have Vol(P) ≤ 64 (Ehrhart's conjecture; see [NP]).
  This holds on the data.
- **num_vertices range 4–16.** Data only; no source states 16 as a maximum.
- **automorphism_order.** Takes values {1,2,3,4,6,8,12,16,24,48} on the data. The value ≤ 48 follows from [T].

## Riemann–Roch for statements 10–12 (derivation)

Let X be a canonical toric Fano 3-fold and Y → X a toric crepant Q-factorial terminalisation (subdivide the
fan by the boundary lattice points of P). Then Y is a terminal weak Q-Fano 3-fold with h^0(-kK_Y) = h^0(-kK_X)
and (-K_Y)^3 = (-K_X)^3. With d = degree, [CC §2.1] gives the formulas below, where l_4 = l(4) ≥ 0 and
l_6 = l(6) ≥ 0 are the basket corrections of Y:

    h3 = 7d + 7 - l_4,    h5 = (55/2)d + 11 - l_6

- **Gorenstein rows (l = 0).** h3 = 7d + 7 and h1 = d/2 + 3. This matches all 4,319 reflexive rows (0 mismatches).
- **Bounding the corrections.** Each term \bar{jb}(r - \bar{jb})/(2r) is at most r/8. [CC §2.8] gives Σ(r - 1/r) ≤ 24.
  Since r - 1/r ≥ 3r/4 for r ≥ 2, Σ r ≤ 32. Therefore l_4 ≤ 3·32/8 = 12 and l_6 ≤ 5·32/8 = 20.
- **On the data.** max l_4 = 63/10 (d = 99/10, index 10) and max l_6 = 69/7.

What each statement reduces to:

- **Statement 10** ⇔ l_4 ≤ 2d - 8.
  - Gorenstein: ⇔ d ≥ 4.
  - General: implied by l_4 ≤ 12 once d ≥ 10.
- **Statement 11** ⇔ l_4 ≤ (19d + 56)/37.
  - Gorenstein: always true.
  - General: implied once d ≥ 388/19 ≈ 20.4.
- **Statement 12** ⇔ l_6 ≤ (79d + 694)/146.
  - Gorenstein: always true.
  - General: implied once d ≥ 2226/79 ≈ 28.2.

Below those thresholds these crude bounds do not suffice. The exact constants reflect the extremal baskets in
this finite class; tight rows include degree 32/5 and 28/5 (index 5) and degree 6 (index 3).

## Table

"Nontrivial range" is where the inequality is not already implied by 4 ≤ degree ≤ 72. The bound 72 comes from the
literature; the bound 4 is from the data only.

| # | hypothesis → inequality | status | source or derivation |
|---|---|---|---|
| 1 | TRUE → d ≤ -13/2·nv + 114 | **weak for nv ≤ 6; otherwise not found** | RHS ≥ 75 > 72 for nv ≤ 6, so implied by [K] Thm 3.6. For nv ≥ 7 it is an envelope of the max degree by vertex count (data max: nv=7 → 109/2, nv=15 → 33/2, nv=16 → 10). Tight at nv = 15, 16. No vertex–degree bound found in the literature. |
| 2 | TRUE → d ≥ nv - 6 | **weak for nv ≤ 10 (given min degree 4, data only); not found** | Nontrivial only for nv ≥ 11. Tight at nv = 12 (d = 6), 15 (d = 9), 16 (d = 10). |
| 3 | TRUE → d ≥ 2/3·nf + 2/3 | **weak for nf ≤ 5; not found** | Tight at nf = 8 (d = 6), 9 (20/3), 10 (22/3). nf = number of vertices of P\*. No bound found relating vol(P\*) to its vertex count. |
| 4 | TRUE → d ≥ 5/3·nf - 34/3 | **weak for nf ≤ 9; not found** | Tight at nf = 12 (26/3), 13 (31/3), 14 (12). |
| 5 | TRUE → d ≥ -1/2·aut + 17/3 | **essentially a data fact; weak for aut ≥ 4** | RHS ≤ 11/3 < 4 for aut ≥ 4. Content: min degree 31/6 when aut = 1 and 14/3 when aut = 2, both tight. No literature on degree versus \|Aut(P)\| found. |
| 6 | is_simplicial → d ≥ -11/30·aut + 187/30 | **data fact; weak for aut ≥ 8; not found** | Simplicial minima: aut 1 → 88/15, aut 2 → 11/2 (both tight), aut 3 → 33/4, aut 4 and 6 → 6. |
| 7 | is_barycentre_zero → d ≤ 35/3·aut - 7/3 | **weak for aut ≥ 7 (via [K]); small-sample fact otherwise** | Only 136 rows. aut 1 has a single row (d = 28/3, tight). aut 2 max d = 21 (tight). aut 3, 4, 6 maxima are 64/7, 27, 30. Not found in the literature. |
| 8 | is_barycentre_zero → d ≥ -5/2·aut + 71/6 | **weak for aut ≥ 4; small-sample fact** | Nontrivial only for aut ∈ {1,2,3}: aut 1 → 28/3 (one row), aut 2 → 41/6 (tight). Not found. |
| 9 | is_dual_barycentre_zero → d ≥ -7/6·aut + 34/3 | **weak for aut ≥ 8; small-sample fact** | 122 rows (Kähler–Einstein by [BB]). No row has aut = 1. Minima: aut 2 → 9, aut 4 → 20/3 (both tight), aut 6 → 6. [L]/Fujita give only the upper bound 64, not a lower bound. Not found. |
| 10 | TRUE → d ≤ 1/5·h3 - 3 | **implied by known results for d ≥ 10 and for all Gorenstein rows; finite check otherwise** | See the Riemann–Roch section: Reid's formula [CC §2.1, R] plus Σ(r - 1/r) ≤ 24 [CC §2.8, KMMT]. The exact constant was not found in the literature. |
| 11 | TRUE → d ≤ 37/240·h3 - 203/240 | **implied for d ≥ 20.4 and for all Gorenstein rows; finite check otherwise** | Same derivation. |
| 12 | TRUE → d ≤ 73/1968·h5 - 19/82 | **implied for d ≥ 28.2 and for all Gorenstein rows; finite check otherwise** | Same derivation with k = 5. |
| 13 | TRUE → d ≥ -1/23·e2 + 257/23 | **weak for e2 ≥ 165 (given min 4, data only); not found** | e2 = 10 + 4a1 + a2 on the data range 15–185. Tight at three tetrahedra: d = 4 (e2 = 165), d = 6 (e2 = 119), d = 8 (e2 = 73). This is an inverse relation between #(2P) and Vol(P\*), a lattice Mahler-type bound. No such bound with these constants was found. |
| 14 | TRUE → d ≤ -1633/840·a2 + 13735/168 | **weak for a2 ≤ 5 (RHS ≥ 72.04, via [K]); not found otherwise** | Tight at d = 96/7 (index 7) and d = 343/8 (index 8). a2 = Vol(P) - num_points + 1, so this is again an inverse Vol(P)–Vol(P\*) relation. Not found. |

## Context: the h7–h10 bounds

For example, d ≥ 2/385·h10 - 6/55 ⇔ h10 ≤ 385/2·d + 21. For Gorenstein rows, h10 = 385/2·d + 21 exactly (Reid's
formula with k = 10, l = 0), so these bounds are equalities on reflexive rows. For other rows the correction
l(11) ≥ 0 makes them strict. They are **implied by known results**: [CC §2.1] plus l ≥ 0 and Kawamata–Viehweg
vanishing for weak Fano.

## Open questions

1. **Minimum degree.** Is min (-K)^3 = 4 over canonical toric Fano 3-folds stated or proved anywhere? The
   literature found only upper bounds ([K], [BKN], Zou arXiv:2407.19870). A lower bound would turn statements
   2–6, 8, 9 and 13 from "data only" into "weak".
2. **Statements 10–12 in the small-degree range.** Can a basket-level argument (index ≤ 420 on the data;
   Σ(r - 1/r) ≤ 24) prove l_4 ≤ 2d - 8 for all canonical toric Fano 3-folds? What is the general (non-toric)
   version? The data maximum l_4 = 6.3 is far below the crude bound 12.
3. **Vertex and facet bounds.** Is there a published vertex–degree or facet–degree inequality for canonical
   Fano polytopes (statements 1–4)? Is nv ≤ 16 stated as a theorem anywhere, or only visible in the data?
   Heuristic: vol(P\*) = Σ over vertices v of P of the area of the facet of P\* dual to v, since every such facet
   sits at lattice height 1. This suggests degree grows with nv through facet areas, but I found no bound.
4. **Mahler-type bounds.** Statements 13 and 14 are inverse relations between the size of P and the size of P\*.
   Is there a lattice Mahler-type inequality Vol(P)·Vol(P\*) ≥ c, or one linear in e2 and a2, for polytopes with
   one interior lattice point?
5. **Kähler–Einstein rows and symmetry.** No `is_dual_barycentre_zero` row has trivial automorphism group, and
   exactly one `is_barycentre_zero` row does (id 121738). Is there a structural reason? The samples are tiny (122
   and 136 rows), so statements 7–9 are weak evidence of any general law.
6. **Unverified side facts.** The full list of subgroup orders from [T] and Hibi's inequality [H] were not verified
   from primary text.
