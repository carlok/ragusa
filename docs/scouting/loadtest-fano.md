# Load test: toric canonical Fano 3-folds (scale target)

Downloaded 2026-09-14 into the git-ignored `external/fano3c/`.

| | |
|---|---|
| source | Zenodo record 5866330 (Kasprzyk), via the Zenodo records API; please cite Kasprzyk, *Canonical toric Fano threefolds*, Canad. J. Math. 62(6), 2010, and doi:10.5281/zenodo.5866330 (dataset README) |
| file | `toricf3c.sql`, 563,679,312 bytes, md5 `d38e5d87917a3444a8b5a877e1868f00` — matches the record's checksum |
| licence | CC0 1.0 (checked in [numbertheory-geometry.md](numbertheory-geometry.md)) |
| import | `sqlite3 toricf3c.db < toricf3c.sql`, 33.0 s |
| script | `loadtest_fano.py`, a throwaway script not in the repository |

## Shape

- One table `toricf3c`: **674,688 rows, 41 columns, no NULL in any column.**
- Stored types: integers for counts, volume, Gorenstein index, h1–h10, e1–e10, Picard rank,
  automorphism order; 0/1 for the six booleans; text for the vertex, point and dual lists, the two
  δ-vectors, the normal form, and `degree`.

| boolean | true | of 674,688 |
|---|---:|---:|
| is_terminal | 634 | 0.09% |
| is_simplicial | 12,190 | 1.8% |
| is_regular | 18 | 0.003% |
| is_reflexive | 4,319 | 0.64% |
| is_barycentre_zero | 136 | 0.02% |
| is_dual_barycentre_zero | 122 | 0.02% |

The 4,319 reflexive rows match the number of reflexive 3-polytopes reported by Kreuzer–Skarke, an
independent consistency check.

| numeric | min | max | distinct |
|---|---:|---:|---:|
| num_vertices, num_faces | 4 | 16 | 13 |
| num_points | 4 | 38 | 35 |
| volume | 4 | 72 | 69 |
| degree (parsed as an exact fraction) | 4 | 72 | 3,167 |
| gorenstein_index | 1 | 420 | 57 |
| picard_rank | 1 | 7 | 7 |
| automorphism_order | 1 | 48 | 10 |

`degree` is an integer on 64,813 rows, including all 4,319 reflexive ones, and a fraction such as
`997/60` elsewhere. Compared as text it sorts wrongly; it must be loaded as an exact rational.

## Identities from the dataset README

| identity | violations |
|---|---:|
| volume = sum of ehrhart_delta | 0 |
| is_regular ⇒ is_simplicial ∧ is_terminal | 0 |
| gorenstein_index = 1 ⇔ is_reflexive | 0 |
| is_simplicial ⇒ picard_rank = num_vertices − 3 | 0 |
| hilbert_delta is symmetric | 0 |
| e1 = num_points (as the README states) | **674,688** |
| e1 = num_points + 1 | 0 |

The README's "e1 is equal to num_points" is off by one in this release: `point_list` never contains
the origin (0 of 674,688 rows), so `num_points` counts the lattice points of P other than the
interior origin, while `e1 = #(P ∩ N)` includes it. The data is consistent; the sentence is not.

## What this means for ragusa

1. **Scale is real but easy to load:** 674,688 × about 28 numeric columns fits the dense numeric
   block in a few hundred megabytes.
2. **Exact rationals are needed in the table**, not only in generated coefficients (`degree`).
3. **The booleans are rare**, so single-property hypotheses cover tiny subsets; hypotheses need
   combinations or derived classes (for example ranges of Gorenstein index or Picard rank), and
   chance baselines will matter.
4. **The README identities are exactly what ragusa should rediscover** (equalities, implications
   among booleans). They form a free known-results check before any novelty claim.
