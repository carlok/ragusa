# Load test: KnotInfo

Downloaded 2026-09-14 into the git-ignored `external/knotinfo/`.

| | |
|---|---|
| source | link on https://knotinfo.org/homelinks/database_download.php, served from `https://raw.githubusercontent.com/knotinfo-org/assets/main/knotinfo_data_complete.xls.zip` |
| file | `knotinfo_data_complete.xls.zip`, 16,471,419 bytes, sha256 `e054075a0d895ea68d081976388f6b8d8177a2c9e167e42f9b91c7880f1a0dcc`; unzips to `knotinfo_data_complete.xls` (82,738,176 bytes) |
| reader | `xlrd` 2.0.2 (added to the calibration environment) |
| licence | none stated on the download page; nothing derived is committed until settled |

## Shape

- One sheet (`sample_dat`), 12,968 rows × 248 columns. Row 0 holds column keys, row 1 display
  names; **12,966 knot rows**: the unknot `0_1` plus 12,965 non-trivial prime knots, matching
  arXiv:2605.22652.
- Half the columns are `<name>_anon` twins; 100 columns are empty.

Profile by content (a column's class needs at least 95% of its non-empty cells to fit):

| class | columns | examples |
|---:|---:|---|
| boolean (`Y`/`N`, `Yes`/`No`) | 11 | alternating, fibered, adequate, l_space (all knots); positive, quasipositive, strongly_quasipositive, positive_braid (2,977 knots); quasi_alternating (1,908); almost_alternating (2,096) |
| numeric | 31 | crossing_number, three_genus, bridge_index, signature, arc_index, smooth_four_genus, determinant, rasmussen_invariant, ozsvath_szabo_tau_invariant, volume, arf_invariant (all knots); braid_index, topological_four_genus, turaev_genus, tunnel_number, ropelength (about 2,800–2,978 knots) |
| numeric or interval | 12 | unknotting_number, smooth_4d_crosscap_number, topological_4d_crosscap_number, double_slice_genus, nu (all knots); crosscap_number, nakanishi_index, smooth_concordance_genus (2,977–2,978 knots) |
| other | 94 | polynomials, notations, symmetry type (`chiral`, `reversible`, …), text values |

Most common short non-numeric cells: empty (2,231,386), `N` (46,829), `Y` (22,706), `hyperbolic`
(12,955), `No` (12,954), `D.N.E.` (12,448), `chiral` (8,238), `infty` (7,729), and intervals such as
`[9,12]` (7,006), `[2, 4]` (4,290), `[2,3]` (4,183).

## What this means for ragusa

1. **Interval values are the norm for hard invariants.** A cell such as `[9,12]` records that the
   true value lies in that range. The table type needs a lower and an upper value per numeric cell;
   a bound is violated only if it fails for every value in the interval, and tight only when the
   interval is a single point. This is new engine work.
2. **Missing values have several spellings** (empty, `D.N.E.`, `infty`, `Not Hyperbolic`); each needs
   an explicit meaning (missing, infinite, not applicable).
3. **Two natural tables.** Many invariants and the positivity booleans cover only about 2,977 knots,
   consistent with the prime knots up to 12 crossings (count not re-checked here). A dense
   ≤ 12-crossing table and a sparse ≤ 13-crossing table are both worth running.
4. The count conflict noted in the shortlist (12,965 knots against the PyPI snapshot's 2,979) is
   explained by this coverage split.
