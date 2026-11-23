# Sprint 6 shortlist

Built from [algebra.md](algebra.md), [numbertheory-geometry.md](numbertheory-geometry.md) and
[combinatorics.md](combinatorics.md). Only facts marked **checked** there are used to rank.

## Finalists against the four criteria

| candidate | objects (checked) | licence (checked) | invariants | partial oracle | prior Graffiti-style work | main risk |
|---|---:|---|---|---|---|---|
| **KnotInfo** | 12,965 prime knots ≤ 13 crossings | no copyright; cite, do not repost (2026-10-02) | 31 numeric, 12 numeric-or-interval, 11 boolean in the spreadsheet (load test) | **47 known inequalities between 33 invariants, same knots** (arXiv:2605.22652) | Graffiti3, narrowly: 2,978 knots from pyknotid, about ten columns (checked) | intervals and missing values; licence |
| **Toric canonical Fano 3-folds** | 674,688 | CC0 | ≈ 28 numeric, 6 boolean, stored | Ehrhart/h* literature only | none found | rediscovering h* theorems; few booleans |
| **Matroids ≤ 9 elements** | 385,370 | CC BY 4.0 | none stored; 30–60 cheap to compute (reported) | small, hand-curated only | none found | 9-element evidence misleads; must compute everything |
| **LMFDB abstract groups** | 544,831 (complete to order 2000 with stated exceptions) | CC BY-SA 4.0 | 18 numeric, 18 boolean, stored | Kourovka Notebook, Groupprops (unstructured) | Graffiti3 on GAP SmallGroups with non-linear templates; five of its conjectures refuted (checked) | heavily mined; theorems are often non-linear |
| **Posets ≤ 10 points** | 2,567,284 at n = 10 | CC BY 4.0 | none stored | none | none found | the headline conjecture is a constant bound |

Not pursued: SnapPy censuses (floating-point invariants, no oracle), LMFDB arithmetic collections
(logarithmic relations), Kreuzer–Skarke (no booleans), CICY (licence), SRGs and designs (dependent
parameters), graphs (ragusa's control; crowded by AutoGraphForge).

## What each finalist would need from the engine

| candidate | loader | missing values | invariant computation | scale |
|---|---|---|---|---|
| KnotInfo | spreadsheet → table | **required**: generators and evaluation currently reject NaN | none | small |
| Toric Fano 3-folds | SQLite or text → table | not expected | none | **674,688 rows: first real scale test** |
| Matroids | rankLine → rank functions | no | **required** (SageMath or own code) | 385,370 rows |
| LMFDB groups | PostgreSQL mirror → table | some fields | none for stored columns | 544,831 rows |
| Posets | Hasse diagrams | no | **required** | 2.6 M rows |

## Decision (2026-09-14)

KnotInfo is the primary target and the toric canonical Fano 3-folds the scale test, as
recommended below; matroids stay in reserve. All three load-test downloads were approved. They go
to the git-ignored `external/` folder; nothing derived is committed until the KnotInfo licence is
settled. *Settled 2026-10-02: no copyright, cite KnotInfo, do not repost the database; see
`CONTROLS.md`.*

## Recommendation

**Primary target: KnotInfo.** It is the only candidate where ragusa's novelty floor can be
measured against a structured catalogue of known results written in ragusa's own statement shape
(Jabłonowski's 47 inequalities over the same 12,965 knots), the way the ETP graph worked for magmas
but partial, so new survivors are possible. Rich booleans give hypotheses; the table is small
enough for exhaustive runs with full attribution. It forces one engine feature that any real
domain needs anyway: missing and interval-valued invariants.

**Scale target: toric canonical Fano 3-folds.** CC0, ready columns, 674,688 rows. It exercises the
regime ragusa was designed for without any invariant computation, and it has no prior
Graffiti-style work.

**Reserve: matroids on at most 9 elements.** The most interesting literature fit (class-conditioned
Merino–Welsh), but everything must be computed and small-n evidence is known to mislead.

## Open before committing

1. ~~KnotInfo's data licence.~~ Resolved 2026-10-02: no copyright; the maintainers ask for a
   citation and that the database itself not be reposted. Derived aggregates are fine.
2. ~~The Graffiti3-on-knots claim.~~ Resolved: Graffiti3 ran on knots (2,978 knots, about ten
   columns), finite groups and Calabi–Yau hypersurfaces; see the area pages. Graffiti3's statement
   templates (square roots, logarithms, exponentials) are already richer than ragusa's linear
   bounds, so ragusa's distinct contribution in these domains is the floor report (attribution,
   chance baselines, a measured novelty rate against a catalogue), not the language. The refuted
   Graffiti3 group conjectures are the case for it.
3. ~~The KnotInfo count conflict.~~ Resolved by the load test: many invariants are filled only for
   the 2,977 knots up to 12 crossings ([loadtest-knotinfo.md](loadtest-knotinfo.md)).

## Load tests (each needs your approval to download)

| candidate | files | size |
|---|---|---|
| KnotInfo | invariants spreadsheet (zipped `.xls`) | ~16 MB |
| Toric Fano 3-folds | `toricf3c.sql` (or `toricf3c.txt`, 817.1 MB) | 563.7 MB |
| Matroids | `matroids09_rankLine` | 197.1 MB |

Reading KnotInfo's `.xls` from Python would also need an Excel reader package in the calibration
environment (another download).
