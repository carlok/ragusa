# Literature check: the four held-out harvest bounds on canonical toric Fano 3-folds

Scope: the four bounds kept by the held-out harvest on Kasprzyk's classification (674,688 rows,
Zenodo 5866330). Checked 2026-09-15. This follows up [fano3c_degree_survivors.md](fano3c_degree_survivors.md),
which derived h1 = d/2 + 3 on Gorenstein rows and listed Hibi's inequality as unverified.

| # | rows | bound |
|---|---|---|
| 1 | all | degree ≥ 2·h1 − 6 |
| 2 | all | volume ≥ 2·num_points − 4 |
| 3 | is_reflexive | degree ≤ 2·h1 − 6 |
| 4 | is_reflexive | volume ≤ 2·num_points − 4 |

## Result

| statement | status | source / derivation | data check (exact rationals) |
|---|---|---|---|
| 1. degree ≥ 2h1 − 6 (all) | **implied by known results** | Reid's plurigenus formula at n = 1 [CC §2.1, citing R], applied to a toric crepant terminalisation. Slack = 2·l(2) ≥ 0. Strict unless the basket is empty, i.e. unless X is Gorenstein. | 0 violations. Equality on 4,319 of 4,319 reflexive rows and on **0 of 670,369** non-reflexive rows. Minimum non-reflexive slack is exactly **1/2** (id 3, degree 33/2, h1 = 11, index 2), matching the derived lower bound of 1/2. |
| 3. degree ≤ 2h1 − 6 (reflexive) | **implied by known results** (well known in the Gorenstein case) | Same formula with an empty basket: h^0(−K) = d/2 + 3. | Equality on 4,319 of 4,319 rows. |
| 2. volume ≥ 2·num_points − 4 (all) | **known**: Hibi's lower bound theorem, restated | volume − (2·num_points − 4) = h\*_2 − h\*_1. Hibi [H94]: if P has an interior lattice point, then h\*_1 ≤ h\*_i for 1 ≤ i ≤ d − 1. Here d = 3, i = 2. | 0 violations. Identities checked on all 674,688 rows: volume = 2 + a1 + a2; a1 = num_points − 3 = e1 − 4; slack = a2 − a1. Equality on 4,319 reflexive rows and **0** non-reflexive rows. |
| 4. volume ≤ 2·num_points − 4 (reflexive) | **known**: Hibi's reflexivity criterion, restated | Hibi [H92]: for a lattice polytope with the origin in its interior, P is reflexive ⇔ h\* is symmetric. For d = 3, symmetry means h\*_1 = h\*_2. | Equality on 4,319 of 4,319 rows. Equality on 0 non-reflexive rows, consistent with the "⇐" direction. |

So on this dataset, "equality in 1" ⇔ reflexive ⇔ "equality in 2". The first equivalence is derived below. The
second is Hibi's criterion, since every row has exactly one interior lattice point, and a3 = 1.

## Method

**Local files read in full:** `docs/scouting/loadtest-fano.md`, `calibration/src/ragusa_a/export_fano.py`,
`docs/literature/fano3c_degree_survivors.md`.

**Data check.** Read-only (`mode=ro`) SQLite over `external/fano3c/toricf3c.db`, in Python with `fractions.Fraction`.
`degree` was parsed from text and never CAST. `ehrhart_delta` was JSON-decoded into (1, a1, a2, 1). The script is
in a throwaway script (`harvest_check.py`, about 9 s), not in the repository. The table above reports its output. Additional counts:
- `gorenstein_index = 1 ⇔ is_reflexive` holds on all rows.
- h1 = d/2 + 3 on all 4,319 reflexive rows.
- On every non-reflexive row, slack(1) ≥ (index − 1)/index, and the denominator of slack(1) divides the Gorenstein index.

**Web queries:**
- "Hibi lower bound theorem for Ehrhart polynomials h\*_1 ≤ h\*_i interior point"
- "Hibi dual polytopes of rational convex polytopes Combinatorica 1992 reflexive h\* symmetric"
- "Hibi A lower bound theorem for Ehrhart polynomials of convex polytopes Advances in Mathematics 105 1994"

I also fetched Chen–Chen (arXiv 0712.4356) directly, because the previous report cites it for Reid's formula.

**Sources**

| ref | source | how read |
|---|---|---|
| [CC] | J. A. Chen, M. Chen, *An optimal boundedness on weak Q-Fano threefolds*, arXiv:0712.4356v2, https://arxiv.org/abs/0712.4356 | PDF downloaded and converted to text. §2 (pp. 2–7) and the bibliography read in full. |
| [R] | M. Reid, *Young person's guide to canonical singularities*, Proc. Sympos. Pure Math. 46 (1987) 345–414 | **Not read.** Cited by [CC §2.1] as the source of the formula. |
| [HT] | M. Henk, M. Tagami, *Lower bounds on the coefficients of Ehrhart polynomials*, arXiv:0710.2665v2 (Europ. J. Combin.), https://arxiv.org/abs/0710.2665 | PDF converted to text. Abstract, §1 introduction and bibliography read. |
| [Ts1] | A. Tsuchiya, *The δ-vectors of reflexive polytopes and of the dual polytopes*, arXiv:1411.2122v3, https://arxiv.org/abs/1411.2122 | PDF converted to text. Pages 1–3 (introduction, start of §1) and bibliography read. |
| [Ts2] | A. Tsuchiya, *Best possible lower bounds on the coefficients of Ehrhart polynomials*, arXiv:1501.02138v3, https://arxiv.org/abs/1501.02138 | Introduction and bibliography read. It only confirms the [H94] citation and does not state the theorem. |
| [H94] | T. Hibi, *A lower bound theorem for Ehrhart polynomials of convex polytopes*, Adv. Math. 105 (1994) 162–165, doi:10.1006/aima.1994.1042 | **Primary not read.** ScienceDirect returned 403. Statement taken from [HT, eq. (1.6)] and [Ts1, p. 1], which agree. |
| [H92] | T. Hibi, *Dual polytopes of rational convex polytopes*, Combinatorica 12 (1992) 237–240, doi:10.1007/BF01204726 | **Primary not read.** Springer requires login, and a search snippet gave only the abstract's gist. Statement taken from [Ts1, p. 2]. |
| [St] | R. P. Stanley, *Decompositions of rational convex polytopes*, Ann. Discrete Math. 6 (1980) 333–342 | Not read. Cited by [HT] and [Ts1] for h\*_i ≥ 0. |
| [K] | A. M. Kasprzyk, *Canonical toric Fano threefolds*, arXiv:0806.2604v2 | §1, p. 3–4 read: definition of terminal and canonical fans, and of Fano polytopes. |

**What each source states, as read** (paraphrased; quotations kept short):
- [CC §2.1] Let X be a terminal weak Q-Fano 3-fold. It has a basket B_X = {(b_i, r_i)} with 0 < b_i ≤ r_i/2 and b_i
  coprime to r_i. For all integers n > 0:

  P_{−n}(X) = n(n+1)(2n+1)/12 · (−K_X^3) + (2n+1) − l(n+1), with l(n+1) = Σ_i Σ_{j=1}^{n} \bar{jb_i}(r_i − \bar{jb_i})/(2r_i).

  The paper also gives the rewrite P_{−1} = ½(−K^3 + Σ b_i²/r_i) − ½Σ b_i + 3. Attribution: "According to Reid [8]".
- [CC §2.8] By Kollár–Miyaoka–Mori–Takagi, −K_X·c_2(X) ≥ 0, which gives Σ 1/r_i − Σ r_i + 24 ≥ 0.
- [HT §1] and [Ts1, p. 1] state the standard facts: h\*_0 = 1, h\*_1 = #(P ∩ Z^d) − (d+1), h\*_d = #interior lattice
  points, h\*_i ≥ 0 [St], and Σ h\*_i = normalised volume [Ts1 uses this]. They state Hibi's bound: if there is an interior
  lattice point, h\*_1 ≤ h\*_i for 1 ≤ i ≤ d − 1, citing [H94]. [HT] shows the hypothesis cannot be dropped.
- [Ts1, p. 2] For an integral d-polytope with the origin in its interior, reflexive ⇔ δ-vector symmetric
  (δ_i = δ_{d−i}), citing [H92].
- [K, p. 3] A toric variety is terminal iff every cone has its rays on an affine hyperplane u = 1, with no other
  lattice points of the cone on or below that hyperplane.

## Derivation for statements 1 and 3 (my own; not found stated in this form)

Setup: X is a canonical toric Fano 3-fold with Fano polytope P (fan = cones over the faces of P), and d = (−K_X)^3 = Vol(P\*).

1. **Terminalisation.** Choose a regular (coherent) triangulation of ∂P that uses every boundary lattice point. The cones over
   its triangles form a projective simplicial fan, giving a toric morphism f: Y → X.
   - Each new ray lies on some facet F of P, where the linear function u_F (u_F = 1 on F) is the support function of −K_X.
     So K_Y = f\*K_X, and f is crepant.
   - In a cone over a triangle T ⊂ F, lattice points with u_F ≤ 1 lie in conv(0, T) ⊂ P. The only interior point of P is 0,
     and the only lattice points on F ∩ T are the vertices of T, because the triangulation uses all points.
     By the criterion in [K], Y is terminal.
   - Hence Y is a terminal, Q-factorial, projective weak Q-Fano 3-fold, and −K_Y = f\*(−K_X) is nef and big.
   - Crepancy gives f_\*O_Y(−K_Y) = O_X(−K_X), so h^0(−K_Y) = h^0(−K_X) = h1. It also gives (−K_Y)^3 = d.
   - *Not sourced:* the existence of regular triangulations that use all points is standard (placing/pulling triangulations).
2. **Reid's formula at n = 1** [CC §2.1] on Y:
   h1 = d/2 + 3 − l(2), with l(2) = Σ_i \bar{b_i}(r_i − \bar{b_i})/(2r_i).
   So **d − (2h1 − 6) = 2·l(2)**.
3. **Sign.** Each basket pair has r_i ≥ 2 and 1 ≤ b_i ≤ r_i/2 (so \bar{b_i} = b_i), which makes each term positive.
   Each term is at least (r_i − 1)/(2r_i) ≥ 1/4. Therefore l(2) ≥ 0, with equality iff the basket is empty.
   This proves statement 1.
4. **Equality ⇔ Gorenstein.**
   - Empty basket ⇔ Y has index 1. The basket records the non-Gorenstein points; terminal cyclic quotients 1/r(1,−1,b)
     are their own basket. This step relies on [R] as summarised by [CC] and was not read in [R] directly.
   - Y Gorenstein ⇔ X Gorenstein. Since K_Y = f\*K_X, the support function of K_Y restricted to a subcone of a cone σ
     of X is the linear function of σ. It has integral slope on the subcone iff it has integral slope on σ.
   - X Gorenstein ⇔ P reflexive (standard; the dataset itself confirms gorenstein_index = 1 ⇔ is_reflexive).
   - Hence equality in 1 holds exactly on reflexive rows, which proves statement 3 as well.
   - **Equality cannot hold for a non-Gorenstein row:** the slack is at least 1/2.
5. **Sharper derived prediction:** slack = 2·l(2) ≥ Σ_i (r_i − 1)/r_i.
   - On the data: min non-reflexive slack = 1/2 exactly, attained at index 2 (basket 1/2(1,1,1)).
   - slack ≥ (index − 1)/index holds on every non-reflexive row.

Caveat: step 1 uses Kawamata–Viehweg vanishing implicitly, because [CC] states the formula for P_{−n} = h^0 on weak
Q-Fano 3-folds. For toric Y, Demazure vanishing suffices. [R] itself was not read.

## Derivation for statements 2 and 4

Conventions, checked on all rows:
- ehrhart_delta = (1, a1, a2, 1) is the h\*-vector of P.
- num_points = #(P ∩ N) − 1, because `point_list` excludes the origin (loadtest note).
- Hence a1 = #(P∩N) − 4 = num_points − 3 [HT, Ts1], and Vol(P) = 1 + a1 + a2 + 1 = 2 + a1 + a2.
- Also a3 = #interior points = 1.

It follows that volume − (2·num_points − 4) = (2 + a1 + a2) − (2a1 + 2) = **a2 − a1**. Then:
- **Statement 2 ⇔ h\*_2 ≥ h\*_1.** This is [H94] with d = 3, i = 2; the interior point is the origin.
- **Statement 4 ⇔ h\*_1 = h\*_2 on reflexive P.** Together with h\*_0 = h\*_3 = 1, this is the symmetry in [H92].
- **Equality in 2 on a non-reflexive row** would make (1, a1, a1, 1) symmetric, and so P reflexive by [H92].
  So equality holds exactly on reflexive rows. The data agree: 0 non-reflexive equalities.

Note that statement 2 needs Hibi's theorem; Stanley non-negativity alone does not give it. Without an interior
point it can fail in general [HT].

## Open questions

1. **Primary texts not read.** [H94], [H92] and [R] were accessed only through secondary statements ([HT], [Ts1], [CC]).
   Before any publication-grade claim, read Hibi 1994 (Thm on h\*_1 ≤ h\*_i) and Hibi 1992 (reflexive ⇔ symmetric),
   plus Reid YPG §10 for the basket definition, including the claim that an empty basket ⇔ Gorenstein.
2. **Direct toric proof of statement 1.** A proof avoiding Riemann–Roch probably exists: count #(P\* ∩ M) against Vol(P\*)
   for the rational polytope P\* via its Ehrhart quasi-polynomial and reciprocity. I did not find one stated.
3. **Is 2h1 − 6 ≤ d with equality iff Gorenstein stated for canonical Fano 3-folds in general?** The Gorenstein identity
   h^0(−K) = d/2 + 3 ("genus" convention) is classical, but I found no explicit statement of the inequality for non-Gorenstein
   canonical Fano 3-folds. It follows from [R] as above, since terminalisations exist in general.
4. **Refined bound.** slack(1) ≥ Σ(r_i − 1)/r_i ≥ (index − 1)/index holds on the data. The second inequality uses
   lcm(r_i) = index, derived, not sourced. A harvest on non-reflexive rows could target d − 2h1 + 6 against gorenstein_index.
5. **Harvest interpretation.** All four kept bounds are known theorems or direct consequences of them, so they are a
   known-results calibration and not candidates for novelty.
