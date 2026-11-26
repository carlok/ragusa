# Load test: matroids on at most 9 elements (reserve)

Downloaded 2026-09-14 into the git-ignored `external/matroids9/`.

| | |
|---|---|
| source | Zenodo record 6825419 (Royle, Mayhew), via the Zenodo records API |
| file | `matroids09_rankLine`, 197,057,794 bytes, md5 `66777481824f5426a1469bafde130c82` — matches the record's checksum |
| licence | CC BY 4.0 (checked in [combinatorics.md](combinatorics.md)) |
| script | `loadtest_matroids.py` (standard library only), a throwaway script not in the repository |

## Format, as the readme describes it

One matroid per line. Line 0 is `NULL` (the empty matroid). Every other line is the rank function of
a matroid on `E = {0, …, k−1}` as a string of length 2ᵏ; position `Σ_{x∈S} 2ˣ` holds `rk(S)`.

## Results

- 385,370 lines, read in 0.2 s; **0 malformed or inconsistent lines** (every line has length 2ᵏ with
  1 ≤ k ≤ 9, digits only, rank 0 at the empty set, maximal rank at E).
- By number of elements: 1, 2, 4, 8, 17, 38, 98, 306, 1724, 383,172 — total 385,370, matching the
  record and OEIS A055545.
- 9-element matroids by rank: 1, 9, 87, 1275, 190,214, 190,214, 1275, 87, 9, 1 for ranks 0…9, symmetric
  as matroid duality requires. The table is essentially the 380,428 matroids of rank 4 or 5 on
  9 elements.
- Numbers of bases on 2000 random 9-element matroids: 0.14 s in plain Python, so about 27 s for all of
  them; range 15–122 in the sample.
- Cross-check from the readme: M(K₄) is in the catalogue and has 16 bases, as it should.

## What this means for ragusa

1. Loading is trivial and the format is self-checking; no external software is needed for rank-based
   invariants.
2. The table is lopsided (99.4% nine-element, and among those almost all rank 4 or 5), so hypotheses
   should include size and rank classes, and chance baselines matter.
3. The expensive invariants (Tutte polynomial evaluations, representability) need real code or
   SageMath; the cheap ones (bases, circuits, flats, girth) are seconds to minutes.
