# CONTROLS

One entry per gate: what it reproduced, the result, and any routine written,
failed a control with, and discarded. Failures stay in this file.

| gate | reproduces | status | result | discarded routines |
|---|---|---|---|---|
| G-A0 | `linear_programming` has a working LP solver on macOS | **fail as first recorded**; fixed — `run_txg.py` prepends PuLP 3.3.2's bundled `osx/i64/cbc` (CBC 2.10.3, x86_64 via Rosetta) to PATH when LP is requested and no solver is found; LP then yields 144 candidates on the §1.1 scope (597 → 392 → 216 → 141; the solver path is in the generation row's notes) | PuLP's bundled CBC is available (`listSolvers` → `['PULP_CBC_CMD']`), but txgraffiti 0.4.1 `generators/optimization.get_available_solver` only uses `shutil.which("cbc")` / `shutil.which("glpsol")` and ignores PuLP's binary. With neither on PATH it raises, and `safe_generator` turns the error into zero candidates. §1.1's claim that LP "will run" on a Mac does not hold for the shipped code | the first G-A0 entry ("pass") was checked against PuLP, not the code path txgraffiti uses |
| G-A1 | PROMPT.md §1.1 counts 453 / 384 / 216 / 141 (convex_hull + ratios, 12 features × 6 predicates, `independence_number`) | pass | `python -m ragusa_a.run_txg` → 453 / 384 / 216 / 141; Dalmatian 5.55 s on this Mac vs 20.3 s in §1.1 | first LP-inclusive run wrote rows with 0 LP candidates because `safe_generator` swallowed the solver error; rows moved out of `results/`, script now refuses to write when any generator yields nothing |
| G-A2 | AutoGraphForge `results/unified_report.json` figures from its own `results/`; the 0-vs-154 tension | resolved (see log 2026-09-14) | the 0 is true by construction; the 154 reproduces; the pipeline's own flags give 33, overlap 8 | first comparison script read `d["survivors"]` (an int count) as the row list; fixed to `d["conjectures"]` |
| G-A3 | each leave-one-out kernel reproduces its shipped rule's decisions on the full `graph_data` pool (1716 candidates) | pass | 0 mismatches for Dalmatian (exact arithmetic, first attempt), Sophie and Morgan | |
| G-A4 | kernel decision without object r equals the shipped rule re-run on the table without r | pass | full table: 4 removals per rule (2 with most flips, 2 random), 0 mismatches; small table: every removal of a 30-row table for all three rules (`tests/test_rules.py`) | |
| G-A5 | staged `remove_duplicates > filter_with_dalmatian > filter_with_morgan` equals `txgraffiti2()` | pass | 379 = 379 | first census run crashed at this gate calling `txgraffiti2.txgraffiti2` (the import is already the function); outputs are written at the end, so nothing reached `results/`; fixed and rerun. The background task reported exit 0 because the command chained an `echo` after the script |
| G-B1 | all six TxGraffiti tables + parsimagma corpus load into one table type; typing and values match pandas | pass | `cargo test` in `engine/`: column names, order, skipped columns, every boolean column and NaN count match pandas + `auto_wrap` on all six tables; every numeric column is bit-identical to pandas' correctly rounded parse (`float_precision="round_trip"`); magma corpus 2786 × 4694, per-row popcounts match `laws_satisfied`, E1 holds everywhere, E2 nowhere; content hashes match `fixtures/pins.json` | first comparison used pandas' default parser as reference and failed on 41 columns; the reference was wrong, not the loader (see log) |
| G-B2 | engine generators reproduce txgraffiti 0.4.1's candidates (§1.1 scope and full `graph_data`, target `independence_number`) | pass | `cargo test -p ragusa-gen --test g_b2`: `limit_denominator` identical to Python on 2129 cases; ratios identical in order (144, 598); convex hull identical as per-call multisets (309/309, 1348/1348, none needing the 1e-6 tolerance, including 4 and 10 joggled calls); LP: every bound reaches txgraffiti's total slack within 1e-4, 136/144 and 517/598 identical, the rest are alternative optima | first `Rational` was i128-only and could not hold txgraffiti's 8.36·10⁴⁸ ratio coefficient; extended with an exact huge-integer form |
| G-B3 | a published result reappears from the data | pass | `tests/g_b3.rs`: ratio generator states `(regular) → (independence_number <= 1·matching_number)` (paper's Theorem 2, Caro et al.); without the hypothesis the coefficient exceeds 1 (126 of 340 graphs have α > μ) | |
| G-B4 | engine partitions vs Project A partitions on `graph_data` (agreement reported, not forced) | reported | since Sprint 10 run with the Dalmatian counterpart explicitly: `ragusa floor graph_data --target independence_number --redundancy sharp_coverage --out results/floor/graph_data_independence_number_sharp_coverage` (reproduces the earlier result exactly), then `python -m ragusa_a.compare_b4` → `results/rows/gb4_graph_data_independence_number.json`; same source CSV; over 1635 bounds in both pools: validity agrees on 1480, txgraffiti alone rejects 155; generality and Morgan agree on all 19 bounds that reached it; ragusa keeps 19 survivors, txgraffiti 379, 16 in common (see log) | |
| G-B5 | implication scan on parsimagma's magma corpus is sound against the ETP graph and reproduces parsimagma's own counts | pass | `cargo test -p ragusa-report --test g_b5`: 0 ETP-true pairs refuted by a corpus magma; false pairs 13855357, refuted by the corpus 13574842, hard core 416 of 1062 — all equal to parsimagma's `summary.tsv` | gate as first written ("zero known-false emitted") was wrong: an incomplete corpus must emit false implications, and how many is the floor being measured; the soundness direction is the reverse |
| G-B6 | frontier tables exported to ragusa CSV load exactly as their manifests say | pass | `make frontier-gates` (exporters, then `cargo test --release -p ragusa-table --test g_b6 -- --ignored`): `knotinfo_le12`, `knotinfo_le13` and `fano3c` match file hash, object count, names hash, and per column the point / interval / missing counts and lower / upper value hashes (numeric) or true / false / missing counts and value hash (boolean); `fixtures/pins.json` unchanged for every committed table (38 workspace tests pass) | |
| G-B7 | lazy interval-aware evaluation changes nothing on exact tables | pass | `make reproduce-b` floor commands rerun; `git diff results/floor` shows only the engine git rev in `graph_data_independence_number` and `magma_corpus_etp4694_implications` (json and md); `candidates.csv` byte-identical; workspace tests and clippy clean | |
| G-B8 | attribution derived from each filter's partition equals attribution by per-object re-runs | pass | since Sprint 10 for each redundancy rule, all derived (Sprint 11: `greedy_touch` reruns only objects whose removal can change a pick; mutation check in the log) and for the touch stage with count and √ thresholds on the random tables; `cargo test -p ragusa-report --test g_b8`: all four stages of the full `graph_data` run equal `leave_one_out` exactly; `ragusa-filter` `floor::tests`: equal on 3000 seeded random tables with intervals, unknown hypotheses and inactive objects, each rule's inputs both unfiltered and from the pipeline, with every admissible flip direction occurring; mutation check: three of four sabotaged rules fail the test, the fourth was dead code and was removed (see log); committed floor reports unchanged except the engine rev | |
| G-B9 | Jabłonowski's catalogue, projected onto KnotInfo columns, is not refuted by the KnotInfo tables | pass | `cargo test --release -p ragusa-report --test g_b9 -- --ignored --nocapture` (needs the exports): 132 distinct column bounds from the transitive closure of the 47 relations plus sharper form S4, 0 skipped; `knotinfo_le12` 2977 non-trivial knots and `knotinfo_le13` 12,965, 0 bounds refuted on either; bounds through `fd_clasp_number` and `td_clasp_number` (both "likely" mappings) are mostly undecided because those columns are mostly missing | |
| G-B10 | survivors re-checked on objects the run never saw | reported | `ragusa floor <table> --target T --holdout RULE` writes `heldout.json`/`heldout.md`; unit test on a hand-built split; on KnotInfo, target `unknotting_number`: with a seeded 20% hold-out of `knotinfo_le12`, 14 of 38 survivors are refuted on held-out knots, against 274 of 2751 candidates valid on training (see log: a negative result for sharp coverage) | |
| G-CI | bit-identical partitions across all kernel backends | pending | | |

## Log

<!-- Dated entries below. Numbers here come from scripts, with the command that produced them. -->

### 2026-10-02 — KnotInfo: the maintainers' answer

Charles Livingston, writing for himself and Allison Moore: KnotInfo carries no copyright, and they
ask only that users cite it. They asked that the full database not be posted, because a copy goes
out of date and readers may take it for a mirror. That concern is sharper now that AI systems serve
knot invariants from whatever copies they find.

What changes here: the KnotInfo aggregates already in this file and in `docs/scouting/` can stay,
with the citation (`knotinfo` in `CITATIONS.bib`). What does not change: the export itself
(`external/knotinfo/export/`, 12,965 rows) stays out of the repository for good, not pending an
answer. It was never tracked. Results name the file they were computed on, downloaded 2026-09-14,
sha256 `e054075a0d895ea68d081976388f6b8d8177a2c9e167e42f9b91c7880f1a0dcc`, which is what lets a
reader tell which version a number came from. The download page itself carries no version, date or
checksum.

### 2026-09-14 — G-A2, AutoGraphForge novelty counts

AutoGraphForge at `510cf50`. Produced by the scratch comparison script that
became `calibration/src/ragusa_a/agf_novelty.py`.

1. **`novel_survivors == survivors` (332) holds by definition.** In the legacy
   pipeline (`legacy/run_parallel.py`, `legacy/unified.py`, removed in
   `e2d3616`, read from git history), `gen_worker` drops known conjectures per
   shard, and the final `survivors` list keeps only `inequality is None or
   metadata["novel"]`. `_report` then counts `novel` inside that list. The
   report's zero says nothing about how many the filter killed; that count was
   only logged per shard (`known=%5d`) and not persisted.
2. **The §1.2 figure reproduces.** `classify_statement(stmt, list(_GC2TABLE))`
   over the 6,522 rows of `results/cegis_results.json["conjectures"]`: 154
   known, 5.6 s.
3. **The pipeline's own verdicts differ.** The same file stores
   `metadata.novel`, set in `run_cegis.wrap_survivors` by `classify_statement`
   with the seed frame's column list. 33 known, all `cegis-graffiti3`.
   8 in both, 146 only standalone, 25 only stored.
4. **The column list drives the difference.** The 25 stored-only verdicts are
   class theorems conditioned on graph-class predicates ("perfect graph: χ = ω",
   "bipartite: χ ≤ 2 (definition)"), which the 44-entry numeric map cannot see.
   The 146 standalone-only are dominated by numeric relations (`rad ≤ diam`,
   Fiedler `a(G) ≤ δ`, `diam ≤ 2·rad`). Open question for Pastorek: which list
   the pipeline intends, and whether the numeric relations are skipped on
   purpose. Put to him on 2026-10-04 and not answered so far: items 3 and 4
   are our reading of his code and may be wrong.

Consequence for number 4 of Sprint 1: any novelty rate must name its column
list. Report both, never one.

### 2026-09-14 — G-A1 and the LP generator (negative result)

`python -m ragusa_a.run_txg` with and without `--generators convex_hull,ratios,linear_programming`,
§1.1 scope. Survivor lists in `results/rows/*.survivors.txt`.

- Without LP: 453 → 384 → 216 → 141. This reproduces §1.1 exactly.
- With LP (bundled CBC, see G-A0): 597 → 392 → 216 → 141. LP contributes 144
  candidates, and `remove_duplicates` removes all but 8 of them. Dalmatian then
  drops those 8, since its output count (216) is unchanged.
- **The two 141-survivor sets are identical** (`comm -3` is empty). On this
  scope LP adds no surviving conjecture. PROMPT.md §1.1 expected Mac numbers to
  differ; the stage counts before Dalmatian do, the output does not.
  This holds for one target and `feature_size=1` only; do not generalise it.

### 2026-09-14 — Sprint 7 phase A: intervals, missing values, frontier exports (G-B6)

- **Table model.** A numeric cell is a closed interval; upper values are stored only when some cell
  is a genuine interval, and boolean known-value masks only when some value is unknown. Both enter
  the content hash only when stored, so every committed pin is unchanged. `NumericBlock::column`
  now refuses interval blocks, so no generator or filter can silently read a lower bound as a value
  before phase B makes them interval-aware.
- **Format.** ragusa CSV (`calibration/src/ragusa_a/ragusa_csv.py`, loader
  `ragusa_table::ragusa_csv`): points, `lo..hi` intervals, exact `p/q` fractions, empty for missing.
- **KnotInfo export** (rules fixed by a census of every column): `knotinfo_le12` 2977 knots and
  `knotinfo_le13` 12,965 knots, 37 numeric and 12 boolean columns; 6538 and 36,074 interval cells;
  14,325 and 285,003 missing cells; 36 text cells read as missing (`Not Hyperbolic` in five
  hyperbolic columns, one stray link fragment); no bracket with lower > upper. Excluded on purpose:
  row identifiers, the pair-valued `nu` (1390 brackets with a > b), a polynomial column.
- **Fano export:** `fano3c` 674,688 rows, 30 numeric (with `degree` as exact fraction text and the two
  free Ehrhart δ-vector entries) and 6 boolean columns; 122 MB; 24 s.
- **Knot catalogue:** Jabłonowski's 47 relations, 33 quantities (13 mapped to KnotInfo columns, 4 of
  them only "likely"), sharper forms S1–S4 and open candidates c1–c10 transcribed to
  `engine/data/knots/jablonowski2026.json` from page images of arXiv:2605.22652 v4 by a research
  agent. Not yet used: phase D gates it against the table first.

### 2026-09-15 — Sprint 11 phases B and C: harvests

`python -m ragusa_a.harvest` (engine `053c731`; vetted pipeline `--min-tight sqrt` with the default
`greedy_touch`; 20% held out, seeds 0–4; no chance draws). A bound is shortlisted when it survives
in at least 3 seeds, is refuted on no held-out object and confirmed on at least one, is not implied
by the knot catalogue, has a feature term, and names no column with at most 3 distinct values
(`ragusa column-stats`). The last two filters were added after reading the first list, whose top was
table facts such as constant bounds on the crossing-number range; the counts below come from
`harvest.py --from-json`, which reapplies all filters to the saved harvest.

- **KnotInfo ≤ 13** (`--oracle knots`, every numeric target; output in `external/results/`, not
  committed): 185 runs, 706 distinct survivors (by seeds survived: 1: 157, 2: 57, 3: 54, 4: 66,
  5: 372); 484 in ≥ 3 seeds, never refuted, confirmed somewhere; excluded among those: 46 constant
  only, 46 implied by the catalogue, 134 naming a low-cardinality column (overlapping); **282
  shortlisted**: 58 `new_pair`, 32 `same_pair_not_implied`, 192 `outside_catalogue`. Reading the
  top of the list: many are recognisable known theorems restricted to a property (for example
  relations between the Rasmussen and Ozsváth–Szabó invariants and the signature for alternating
  and quasi-alternating knots), and some look like artefacts of stopping at 13 crossings. A
  literature check is running (Phase D); statements are not reproduced here until the KnotInfo
  licence is settled.
- **Fano, targets `degree` and `volume`** (`--hypotheses TRUE,all`; 10 runs, wall time not recorded;
  `results/rows/harvest_fano3c.json` and `.shortlist.csv`): 5 distinct survivors, **4 shortlisted**,
  each in all 5 seeds and confirmed on every one of the 674,690 held-out checks (134,938 × 5) or,
  under `is_reflexive`, on 4285:
  `(TRUE) → (degree >= 2·h1 + -6)`, `(TRUE) → (volume >= 2·num_points + -4)`,
  `(is_reflexive) → (degree <= 2·h1 + -6)`, `(is_reflexive) → (volume <= 2·num_points + -4)`.
  So equality on reflexive polytopes and an inequality on all.
- **Fano verification (Phase D, done):** `docs/literature/fano3c_harvest.md`, by a research agent
  (sources and what was read in full listed there; not yet reviewed line by line by a person).
  None of the four is new. `degree >= 2·h1 − 6`, with equality exactly on reflexive rows, follows
  from Reid's plurigenus formula at n = 1 as stated in Chen–Chen (arXiv:0712.4356) applied to a toric
  crepant terminalisation; the terminalisation step and the lower bound 1/2 on the gap are the
  agent's derivation, not found in print. `volume >= 2·num_points − 4` is Hibi's lower bound theorem
  (h*₁ ≤ h*₂, Adv. Math. 1994) and equality on reflexive rows is Hibi's criterion (reflexive ⇔
  symmetric h*, Combinatorica 1992); both Hibi papers and Reid's were read only through later papers
  that state them. Data check with exact fractions: equality on 4,319 of 4,319 reflexive rows and 0
  of 670,369 non-reflexive rows for both; smallest non-reflexive gap in the degree bound exactly 1/2.
  The Fano harvest recovered known theorems only, which on a complete classification studied since
  2010 is a calibration of the pipeline, not a discovery.

### 2026-09-15 — Sprint 12: artefact checks and the candidate dossier

Engine: `ragusa check <table> --bounds FILE [--edge COLUMN] [--complete-knots]` (bound text parses
back to the same bound, `tests/bound_text.rs`); `ragusa_report::checks::complete_with_catalogue` fills
missing cells of catalogue columns with the integer intervals the catalogue implies from the same
knot (unit test: a bound undecided because of a missing cell becomes refuted); `Evaluated::tolerance`
ignores infinite ends of the right-hand side (no change on any committed report). Data:
`knotinfo_le13_mirror` (signature, s, τ, ε negated; Chern–Simons and Im vol missing;
chirality-dependent properties unknown; trefoil row checked by hand), G-B6 passes with it.

`python -m ragusa_a.candidate_checks` on the 282-bound Sprint 11 shortlist (engine `330371a`),
output in `external/results/`: catalogue completion filled 35,585 cells with 0 contradictions.
Failed checks (overlapping): hypothesis not recorded at the largest crossing number 169, refuted on
mirror images 10, refuted after catalogue completion 9, tight only at the largest crossing number 2;
none refuted on the table itself. **99 bounds survive, in 42 distinct relations.**

`python -m ragusa_a.dossier` wrote `external/candidates/knotinfo_le13/` (README for an external
language model with task, scope, conventions, definitions, checks already done and answer format;
one Markdown file per relation; JSON; LaTeX and PDF). Honest reading of the 42:

- The range-edge check is too weak as specified (tight **only** at 13 crossings). Sprint 11's
  literature pass found most range artefacts tight at 12 and 13, and several such relations survive,
  including fitted bounds with constant terms that torus knots just past the table refute. A
  criterion over the top crossing numbers, or over bounds with a constant term, is still to do.
- Several known theorems and trivial relations survive, because the catalogue does not contain them
  (for example the arc index of alternating knots, and smooth against topological invariants).
- So the dossier is a triage input: the external model's KNOWN / IMPLIED / REFUTED verdicts are
  expected to remove most of it, and nothing in it is claimed new.

`python -m ragusa_a.split_dossier` re-emits the same dossier as one folder per candidate
(`external/candidates/knotinfo_le13_reviewed/Kxx/prompt.md`, about 5 kB each, plus an `answer.md`
skeleton): the whole dossier is too large for one request to an external model, and the candidates
are independent (each is one relation between KnotInfo columns; none refers to another), so each
prompt repeats the task, scope, checks, conventions and only the definitions that candidate uses.
Every number in it comes from `candidates.json`.

### 2026-09-15 — Sprint 11 phase D: literature check of the KnotInfo shortlist

Report `external/results/literature_knotinfo_harvest.md` (git-ignored with everything derived from
KnotInfo until its licence is settled), by a research agent; most sources were read as abstracts or
search snippets, one paper in full, so every classification below is provisional. Only aggregates
and engine lessons are recorded here.

- **Groups.** The 282 shortlisted bounds collapse to 131 distinct relations (identities counted once,
  a relation under several hypotheses counted once): 24 `new_pair`, 21 `same_pair_not_implied`,
  86 `outside_catalogue`.
- **Classification.** 13 known theorems (several absent from Jabłonowski's catalogue), 4 implied by
  known results with a short derivation, 8 mixed, 12 table artefacts, 29 range artefacts with an
  explicit counterexample beyond 13 crossings, 24 suspected range or fit artefacts, 41 not found.
  One relation holds on all 12,965 knots regardless of hypothesis, so its restriction to a property
  says nothing. One relation looks false beyond the table by the agent's own computation (not
  sourced; to verify).
- **Candidates.** The agent names 11 groups as real candidates (not found, not artefacts, not
  refuted by a known family). Their statements stay in the external report.
- **Engine lessons (committable, generic):**
  1. *Chirality.* KnotInfo stores one chirality per knot, so bounds that depend on the sign of
     signature-type invariants are fitted to that choice and refuted by mirror images the table does
     not contain. A mirror-aware check (evaluate sign-dependent bounds also on negated rows) would
     catch several artefacts automatically.
  2. *Range limits.* Many bounds with a constant term are refuted by torus knots T(2,n) just past the
     table, some of whose cells are empty in the table itself. A bound whose tight objects all sit at
     the largest crossing number of its column is a candidate range artefact.
  3. *Empty cells.* A valid lower bound on an empty cell can refute a shortlisted bound; probing
     empty cells with known bounds (e.g. from the catalogue) would expose such cases.
  4. *Column coverage.* Some boolean columns stop at 12 crossings, so a ≤ 13 run silently treats
     13-crossing knots as not applicable under those hypotheses.

### 2026-09-15 — Sprint 11 phase A: exact greedy_touch attribution at scale

Removing an object `r` is the same as starting the greedy cover with `r` already covered. Before the
pick that first covers `r` nothing changes (no earlier pick contains `r`, other gains only drop); at
that pick, with gain `G`, the winner can change only if `G = 1`, or a remaining candidate not tight
at `r` has gain `G`, or gain `G − 1` at a lower position; otherwise both covers agree from then on.
Only objects meeting that condition are rerun, each exactly. A work estimate
(`GreedyTouch::WORK_BUDGET`, 2·10¹¹ word operations) replaces the old `objects² × input` cut-off.

- `floor::tests`: equal to reruns on 3000 random tables, both flip directions now asserted for
  `greedy_touch`; G-B8 on `graph_data` passes; the committed `graph_data` report is byte-identical
  apart from the engine rev.
- Mutation check (three sabotaged versions, one at a time): making lower-position ties count at
  higher positions fails the random-table test; dropping the `G = 1` case fails it; marking every
  added object divergent when a pick has no contender does not, because it only adds reruns, which
  are exact. That mutation is harmless for correctness and costs time, so no test can see it.
- KnotInfo, target `unknotting_number`, default pipeline, no chance draws: `knotinfo_le12` 1273
  candidates into `greedy_touch`, 58 kept, attribution computed in 0.01 s (42 fragile decisions,
  38 objects with a flip); `knotinfo_le13` 1295 in, 69 kept, attribution 0.04 s (48 fragile, 42
  objects). Under Sprint 10's budget both were "not computed".

### 2026-09-15 — Sprint 10: greedy_touch becomes the default redundancy stage

Decision (user, 2026-09-15): replace sharp coverage; no touch stage by default. `greedy_touch` is the
replacement because it is better than sharp coverage in all 8 held-out comparisons (both tables,
both seed ranges, with and without the touch stage; phases B and C below). It is **not** better than
the pool it filters: no redundancy rule tried is.

- **Committed `graph_data` report regenerated** (`ragusa floor graph_data --target
  independence_number`): validity 2510 and duplicates 1526 as before; `greedy_touch` keeps 33 (sharp
  coverage kept 19), generality keeps all 33; objects with a flip at the redundancy stage 33
  (sharp coverage: 9). The report now states `run.redundancy` explicitly.
- **Cost: attribution.** `greedy_touch` has no closed-form attribution; the report computes it by
  one rerun per object while `objects² × input ≤ 2·10⁹` (`graph_data`: 340² × 1526 ≈ 1.8·10⁸,
  computed) and reports `"computed": false` above. KnotInfo ≤ 12 and Fano runs will not have
  attribution at that stage. `maximal_touch` keeps exact attribution but ranked behind greedy.
- **G-B4 kept intact.** Dalmatian is compared with sharp coverage, so `make reproduce-b` now also
  writes `results/floor/graph_data_independence_number_sharp_coverage/` with `--redundancy
  sharp_coverage`. That report equals the previously committed default report apart from the new
  `run.redundancy` field and the engine rev, and `gb4_graph_data_independence_number.json` is
  unchanged apart from recording the rule.
- `rule_study.py` now asks for `--redundancy sharp_coverage` explicitly, since its rules use
  sharp-coverage witnesses. The √ touch stage stays opt-in (`--min-tight sqrt`): with it, survivors
  are refuted less often than the validity stage in three of four held-out cells.

### 2026-09-15 — Sprint 10 phase C: the √ touch stage in front of each redundancy rule

`heldout_sweep.py --engine-args "--min-tight sqrt --redundancy RULE"` (engine `779c9c4`, where
`--min-tight sqrt` keeps candidates tight on at least `⌈√a⌉` of the `a` training objects their
hypothesis holds on; exact attribution, both flip directions checked against reruns). Refuted share
of tested survivors after generality (survivors kept in brackets); `graph_data` JSONs committed as
`results/rows/heldout_sweep_graph_data_sqrt_*`, KnotInfo in `external/`:

| table, seeds | validity | touch √ pool | → sharp_coverage | → greedy_touch | → maximal_touch |
|---|---:|---:|---:|---:|---:|
| graph 0–4 | 0.241 | 0.078 | 0.174 (1007) | 0.142 (1395) | 0.160 (2573) |
| graph 5–9 | 0.229 | 0.160 | 0.322 (1071) | 0.245 (1462) | 0.277 (2674) |
| knots 0–4 | 0.166 | 0.049 | 0.104 (1477) | 0.083 (2367) | 0.082 (4778) |
| knots 5–9 | 0.154 | 0.058 | 0.080 (1507) | 0.067 (2397) | 0.073 (4873) |

- **Selection (seeds 0–4):** `greedy_touch` lowest on graphs; on knots `maximal_touch` and
  `greedy_touch` tie (0.082, 0.083) and greedy keeps half as many bounds. Chosen: `greedy_touch`.
- **Confirmation (seeds 5–9):** `greedy_touch` lowest on both tables.
- Together with phase B (no touch stage: greedy 0.302 / 0.291 / 0.329 / 0.283 against sharp 0.341 /
  0.399 / 0.389 / 0.310), `greedy_touch` is better than sharp coverage in all 8 comparisons, both
  tables, both seed ranges, with and without the touch stage.
- **Still true:** every redundancy rule raises the refuted rate over the pool it is applied to (touch
  √ pool 0.078 → greedy 0.142, and so on). With the √ touch stage, survivors are safer than the
  validity stage in three of four cells; on `graph_data` seeds 5–9 they are slightly worse (0.245
  against 0.229; 59 runs worse, 61 better).

### 2026-09-15 — Sprint 10 phases A and B: a scaled threshold, and two failed replacements

Engine `1d57b4d` (that commit broke the `ragusa-filter` test build through a removed import; the
library and the release binary used here were unaffected; fixed in the next commit).

**A. Scaled touch threshold.** `rule_study.py` with the new column `applicable_training` (objects of
the training table where the hypothesis holds). Refuted share of tested bounds, pooled (runs better
than the duplicates pool in brackets):

| rule | graph 0–4 | graph 5–9 | knots 0–4 | knots 5–9 |
|---|---:|---:|---:|---:|
| duplicates pool | 0.286 | 0.235 | 0.222 | 0.208 |
| tight ≥ 10 (Sprint 9) | 0.083 (114) | 0.164 (98) | 0.067 (143) | 0.071 (135) |
| tight ≥ 5% of applicable | 0.147 (120) | 0.226 (71) | 0.112 (164) | 0.110 (157) |
| tight ≥ 10% of applicable | 0.102 (117) | 0.189 (91) | 0.112 (155) | 0.113 (148) |
| tight ≥ √applicable | **0.078** (114) | **0.160** (96) | **0.049** (152) | **0.058** (146) |
| tight ≥ √applicable / 2 | 0.149 (114) | 0.250 (55) | 0.103 (154) | 0.105 (150) |

Chosen on seeds 0–4, where tight ≥ √applicable had the lowest rate on both tables; confirmed on seeds
5–9, where it is again lowest on both. It keeps 17,140 and 18,967 bounds over 120 graph runs and
about 32,500 over 185 knot runs; 23 and 25 knot runs keep no tested bound. Fractions of applicable
objects do worse than √: on graphs 5–9 the 5% rule is barely better than the pool.

**B. Replacing sharp coverage.** `heldout_sweep.py --engine-args "--redundancy RULE"`. Refuted share of
tested survivors (after generality):

| table, seeds | validity | duplicates pool | sharp_coverage | maximal_touch | greedy_touch |
|---|---:|---:|---:|---:|---:|
| graph 0–4 | 0.241 | 0.286 | 0.341 | 0.343 (26,599 kept) | 0.302 (3824) |
| graph 5–9 | 0.229 | 0.235 | 0.399 | 0.270 (26,740) | 0.291 (3851) |
| knots 0–4 | 0.166 | 0.222 | 0.389 | 0.347 (66,315) | 0.329 (12,482) |
| knots 5–9 | 0.154 | 0.208 | 0.310 | 0.309 (65,890) | 0.283 (12,433) |

- **Neither replacement is confirmed.** Both are worse than the duplicates pool they start from in
  every cell, and worse than the validity stage. `greedy_touch` is better than sharp coverage in all
  four cells, by 0.03 to 0.11; `maximal_touch` keeps ten times more bounds for no gain.
- **What this says.** Every redundancy rule tried so far (strictly tightest somewhere, maximal tight
  set, greedy cover of tight objects) raises the held-out refutation rate over the pool. Removing
  redundancy by any notion of "covers objects others do not" keeps bounds fitted to particular
  objects. The robust selection found is the touch threshold, which does not remove redundancy.
- Per the plan the default stays `sharp_coverage` until a replacement is confirmed. Next measured:
  the √ touch stage before each redundancy rule.

### 2026-09-15 — Sprint 9 phase C: touch stage in the engine

`Touch { min }` (`ragusa floor ... --min-tight K`): after table duplicates, keep candidates tight on
at least K active objects. Attribution is exact (a representative with exactly K tight objects flips
at each of them; nothing else flips), checked against reruns on the 3000 random tables with `min = 2`
(`floor::tests`, both flip directions asserted). Off by default: committed floor reports unchanged
(engine `f2621ff`).

`python -m ragusa_a.heldout_sweep <table> --seed-from 0|5 --engine-args "--min-tight 10"`
(`graph_data` JSON in `results/rows/heldout_sweep_graph_data_touch10_s*.json`; KnotInfo in
`external/`). Refuted share of tested bounds, pooled:

| table, seeds | validity | touch 10 | survivors with touch 10 | survivors without (rule study, phase B) |
|---|---:|---:|---:|---:|
| graph_data 0–4 | 0.241 | 0.083 | 0.207 (869 kept) | 0.341 (2308 kept) |
| graph_data 5–9 | 0.229 | 0.164 | 0.325 (986) | 0.399 (2321) |
| knotinfo_le12 0–4 | 0.166 | 0.067 | 0.139 (1688) | 0.389 (6779) |
| knotinfo_le12 5–9 | 0.154 | 0.071 | 0.104 (1725) | 0.310 (6541) |

Runs where survivors are refuted more often than the validity stage, with touch 10: graph 45/120
(seeds 0–4) and 68/120 (5–9); knots 66/150 and 53/150 (35 runs per range keep no tested survivor).
Without touch, the same comparison exists only for seeds 0–4 (Sprint 8 sweep): graph 89/120, knots
145/185.

- **The touch stage helps survivors in every cell**, by a factor of 1.2 (graph 5–9) to 3 (knots
  5–9), and on KnotInfo survivors become safer than the average valid candidate.
- **Sharp coverage still undoes part of it.** In all four cells the survivors are refuted more often
  than the touch pool they were chosen from (0.083 → 0.207, 0.164 → 0.325, 0.067 → 0.139,
  0.071 → 0.104), and on `graph_data` seeds 5–9 more often than the validity stage. Choosing the bound
  tightest on some object is itself the mechanism that selects fragile bounds; the touch stage only
  gives it less to choose from.
- **Not decided here:** whether the default pipeline gains the touch stage (it would change the
  committed `graph_data` report and cut survivors by about 60%), whether K should scale with table
  size, and whether sharp coverage should be replaced by a redundancy rule that does not select for
  extremes. Survivors from runs without the touch stage should be read with the Sprint 8 rates in mind.

### 2026-09-15 — Sprint 9 phase B: rule study

`python -m ragusa_a.rule_study <table> --seeds 0-4|5-9` (engine `76d73f4`): every numeric target,
20% held out, all generators and boolean hypotheses; each rule applied to the representatives of
the table-duplicates stage and scored by the share of tested bounds refuted on held-out objects.
`graph_data` 120 runs per seed range (`results/rows/rule_study_graph_data_s0-4.json`, `_s5-9.json`);
`knotinfo_le12` 185 runs per range (`external/results/`, not committed). Rules were to be chosen on
seeds 0–4 and confirmed on 5–9.

Refuted rate of tested bounds (pooled), and runs better than the pool:

| rule | graph 0–4 | graph 5–9 | knots 0–4 | knots 5–9 |
|---|---:|---:|---:|---:|
| duplicates pool (random-subset baseline) | 0.286 | 0.235 | 0.222 | 0.208 |
| sharp ≥ 1 (current) | 0.341 (43/120) | 0.399 (18/120) | 0.389 (42/185) | 0.310 (58/185) |
| sharp ≥ 5 | 0.277 (76/120) | 0.404 (53/120) | 0.335 (85/185) | 0.263 (101/185) |
| sharp ≥ 10 | 0.283 (60/120) | 0.342 (49/120) | 0.329 (95/185) | 0.250 (115/185) |
| tight ≥ 5 | 0.160 (106/120) | 0.254 (53/120) | 0.083 (150/185) | 0.083 (144/185) |
| tight ≥ 10 | **0.083** (114/120) | **0.164** (98/120) | **0.067** (143/185) | **0.071** (135/185) |
| denominator ≤ 10 | 0.248 (107/120) | 0.206 (99/120) | 0.116 (156/185) | 0.107 (159/185) |
| sharp ≥ 2 and denominator ≤ 100 | 0.223 (80/120) | 0.359 (42/120) | 0.202 (88/185) | 0.136 (110/185) |

(Full tables with kept counts in the JSON files and the script output; 15 rules in all.)

- **Requiring more sharp witnesses does not fix sharp coverage.** On `graph_data` it looks better on
  seeds 0–4 and is worse than the pool on seeds 5–9. The seed ranges disagree this much, so any rule
  chosen on one range alone would have been fooling us.
- **Tight ≥ 10 is the only rule best on both tables in both seed ranges**, including the ranges not
  used to choose it: 8–16% refuted on graphs and 7% on knots, against pools of 21–29%.
- **Caveats.** It keeps many bounds (18,633 and 20,207 over 120 graph runs; about 41,000 over 185
  knot runs), so it trades redundancy for robustness. A bound attained on ten or more objects may be
  robust because it is dull (a structural equality) rather than deep; not measured. The threshold is
  an absolute count on tables of 272 and 2382 training objects. 30 of 185 knot runs keep no tested
  bound under it.

### 2026-09-15 — Sprint 8 phase B: profiling the Fano run

`target/release/ragusa floor fano3c --target degree --hypotheses TRUE,all --timings FILE` (11-core
machine; timings files kept in the scratchpad, not committed).

- **Before** (engine `c56402f`): 1324 s wall, 1.56 GB peak. `generate.linear_programming` 1294.5 s;
  every filter, attribution and chance baseline together under 30 s; convex hull 1.1 s.
- **Rejected fix: a smaller equivalent LP.** Eliminating the slack columns (the objective is
  `a·Σx + n·b − Σy`) and merging duplicate points gives the same feasible set and optimum, but HiGHS
  then returns different vertices among tied optima: G-B2 still passes (identical bounds 498 of 598
  instead of 517 on full `graph_data`; 128 of 144 instead of 136 at §1.1 scope), and the committed
  `graph_data` floor report changed. Reverted, since committed reports must reproduce.
- **Kept fix: parallel LP calls** (`b50d78d`). The unchanged single-threaded models run in parallel
  and results are consumed in the original order; committed floor reports reproduce byte for byte.
  Fano: 398 s wall (3.3× faster), identical stage counts (2307 → 2288 → 1548 → 47 → 47); LP call
  seconds summed 1957 s (contention: 11 threads give 3.3×). **Peak memory rose to 7.9 GB**, since up to
  11 HiGHS models with 674,688 slack columns each are alive at once. If memory matters more than time,
  cap the LP pool; the compact model would fix both but changes which optimum is reported.
- Remaining time after LP: chance baselines 19 s, partitions 8 s, attribution under 1 s.

### 2026-09-15 — Sprint 8 phase C: literature check of the Fano survivors

`docs/literature/fano3c_degree_survivors.md`, written by a research agent from web sources (URLs and
what was read in full versus from snippets are listed there) plus an exact-rational recheck of the
database. Not yet reviewed line by line by a person.

- None of the 14 survivors that are not h7–h10 restatements has a published source with these
  constants. Each has 0 violations and is tight on at least one row.
- The h7–h10 survivors are implied by Reid's plurigenus formula (as stated in Chen–Chen,
  arXiv:0712.4356): equalities on all 4,319 Gorenstein rows, strict elsewhere. The earlier note
  that `degree >= 2/385·h10 - 6/55` is Riemann–Roch is confirmed.
- Statements 10–12 (degree against h3, h5) are implied by Reid's formula and Σ(r − 1/r) ≤ 24 above
  a degree threshold (10, 20.4, 28.2); below it they are a finite check on the data.
- Most of the rest (vertex, facet, automorphism-order, e2, Ehrhart bounds) are weak where the known
  maximum degree 72 (Kasprzyk; Prokhorov; Jiang–Zhang–Zou for all canonical Fano 3-folds) or the
  data minimum 4 already implies them, and record per-value extremes elsewhere. The minimum degree 4
  has no source found; it is a data fact.
- Automorphism-order statements under the barycentre hypotheses rest on 136 and 122 rows.
- Open questions listed in the document: a source for min degree 4, vertex/facet–degree bounds,
  Mahler-type Vol(P)–Vol(P*) relations (statements 13, 14).
- Data pitfall found: `CAST(degree AS REAL)` in SQLite truncates fractional degrees; the ragusa
  exporter parses them as exact fractions, so engine results are unaffected.

### 2026-09-15 — Sprint 8 phase A: held-out sweep

`python -m ragusa_a.heldout_sweep graph_data --out results/rows/heldout_sweep_graph_data.json`
(engine `cd502ee`): every numeric column of `graph_data` as target (24), seeds 0–4, 20% of graphs
held out, all generators, every boolean hypothesis; 120 runs, none skipped, 38 s.

| stage | kept (pooled) | refuted on held-out | refuted / tested |
|---|---:|---:|---:|
| validity | 309,160 | 74,372 | 0.241 |
| table_duplicates | 183,172 | 52,398 | 0.286 |
| sharp_coverage | 2308 | 787 | 0.341 |
| generality | 2308 | 787 | 0.341 |

Per run, the survivors' refuted rate exceeds the validity stage's in 89 of 120 runs and is lower in
31. The Sprint 7 negative result on KnotInfo is not a one-table accident: on the control table too,
sharp coverage keeps bounds that fail on unseen graphs more often than the average valid candidate.
Pooled rates weight targets with many candidates more; the per-run count does not.

Same script on `knotinfo_le12` (all 37 numeric targets, seeds 0–4; 185 runs, none skipped, 3 min;
output in `external/results/heldout_sweep_knotinfo_le12.json`, not committed):

| stage | kept (pooled) | refuted on held-out | refuted / tested |
|---|---:|---:|---:|
| validity | 536,942 | 89,008 | 0.166 |
| table_duplicates | 276,106 | 61,267 | 0.222 |
| sharp_coverage | 6779 | 2636 | 0.389 |
| generality | 6779 | 2636 | 0.389 |

Survivors worse than validity in 145 of 185 runs, better in 40. Two tables, 305 runs: the effect
is systematic. On both, the duplicates stage already raises the rate, and sharp coverage raises it
most; generality changes nothing here because it discarded no survivor in any run.

### 2026-09-14 — Sprint 7 phase F: frontier runs

- **Hypothesis `TRUE` in generators** (`b6497d6`): the Fano booleans are rare (634 terminal, 18
  regular of 674,688), so single-column hypotheses cover tiny subsets. `--hypotheses TRUE,all` adds
  unconditional candidates; `tests/always.rs` checks every generator gives the same bounds under
  `TRUE` as under an all-true column.
- **Fano, trained on 1%** (`ragusa floor fano3c --target degree --hypotheses TRUE,all --holdout
  fraction=0.99,seed=0`): 6747 rows for training, 667,941 held out; 5.7 s wall, 1.16 GB peak
  resident memory. 1627 candidates; 41 survivors. Refuted on held-out rows: 1481 of 1620 valid on
  training, 971 of 1031 after duplicates, 40 of 41 survivors. A 1% sample misses the extreme
  polytopes, so almost nothing fitted on it extrapolates; survivors fare worst. On a complete
  classification this is a statement about sampling, not about conjectures: a bound that holds on
  all 674,688 rows is a fact about the class (given correct data), which is what the full run
  measures.
- **Fano, full table** (`target/release/ragusa floor fano3c --target degree --hypotheses TRUE,all`,
  engine `b6497d6`, 1000 chance draws): 674,688 rows; 1290 s wall (single run, no per-phase
  timing yet), 1.64 GB peak resident memory; `floor.json` 33.5 MB, mostly the per-object flip
  arrays. 2307 candidates → validity 2288 (19 discarded, all with zero fragile decisions) →
  duplicates 1548 (740 aliases) → sharp coverage 47 → generality 47. Attribution computed at every
  stage. 31 of the 47 survivors are upper bounds on `degree` in terms of `h7`–`h10`, many differing
  only in the constant. They restate Riemann–Roch: for Gorenstein Fano 3-folds
  h⁰(−kK) = k(k+1)(2k+1)/12 · degree + 2k + 1, which at k = 10 is degree = (h10 − 21)/192.5, i.e.
  `degree = 2/385·h10 − 6/55`; the survivor `(TRUE) → (degree >= 2/385·h10 + -6/55)` is exactly this
  line, rediscovered as a bound over all canonical ones (checked by hand arithmetic, not by a
  script). A known-results sanity check, not a new statement. Because the table is a complete
  classification, every survivor is true for all canonical toric Fano 3-folds as far as the data is
  correct; none has been checked against the literature. Where the 21 minutes go is not measured;
  profile before any larger run.

### 2026-09-14 — Sprint 7 phase E: held-out objects (G-B10), and a negative result

- **Mechanism.** `--holdout column>=value` or `--holdout fraction=F,seed=S` (seeded ChaCha8). The
  table is split with `Table::subset`; generators, filters, attribution and the chance baseline run
  on the training subset only, which gets its own pin. Every candidate valid on training is then
  evaluated on the held-out subset with `Evaluated::verdict` (refuted / confirmed / undecided / not
  applicable; the catalogue gate now uses the same rule, G-B9 unchanged). Per stage the report
  counts kept candidates refuted, supported (confirmed somewhere, refuted nowhere) and untested
  (hypothesis holds on no held-out object). Outputs go to `external/results/floor/*_holdout/`.
- **Seeded 20% hold-out** (`knotinfo_le12`, target `unknotting_number`, `fraction=0.2,seed=0`,
  all generators): 2382 knots for training, 595 held out.

  | stage | kept | refuted on held-out | supported | untested |
  |---|---:|---:|---:|---:|
  | validity | 2751 | 274 | 2457 | 0 |
  | table_duplicates | 1261 | 215 | 1036 | 0 |
  | sharp_coverage | 38 | 14 | 24 | 0 |
  | generality | 38 | 14 | 24 | 0 |

  **Negative result.** Sharp coverage raises the held-out refutation rate from about 10% (validity)
  and 17% (duplicates) to 37% of survivors. Keeping the bound that is tightest on some training
  object selects bounds fitted to training extremes; most refuted survivors fail on one or two
  held-out knots. This is the overfitting the floor report is meant to expose. It is one table,
  one target and one seed; whether it holds for the rule in general needs the same check on other
  targets, seeds and `graph_data`. Until then the survivor list must not be read as conjectures
  without a held-out check.
- **Hold-out by crossing number** (`knotinfo_le13`, `crossing_number>=13`): 2977 knots for
  training, 9988 held out. Refuted / supported / untested per stage: validity 216 / 353 / 1779 of
  2777; survivors 1 / 1 / 17 of 36. The test is data-starved rather than informative: on the
  13-crossing knots, 7 of the 12 boolean columns and 20 of the 37 numeric columns are unknown on
  every knot, and `unknotting_number` is an interval on 4368 of 9988 (ad hoc count over the export,
  not a committed script; regenerate before citing). A crossing-number hold-out on KnotInfo needs
  the target and hypotheses restricted to columns the larger knots have.

### 2026-09-14 — Sprint 7 phase D: the knot catalogue as an oracle (G-B9)

- **Projection** (`ragusa_oracle::knots`). The 47 relations are closed transitively (shortest chain
  kept as the derivation); every implied `u ≥ v` between two mapped vertices becomes linear bounds on
  columns, one per column as target, with an absolute value on the smaller side split into both
  signs. A relation whose larger side is an absolute value would not be linear; none occurs. S4 is
  added as given and solved for its other column.
- **Gate** (`ragusa_report::knots::check_catalogue`). Each bound is evaluated on knots with crossing
  number ≥ 1, with the engine's interval rules: refuted only when it fails for every value in the
  intervals. To state "for every knot" the engine gained a reserved hypothesis `TRUE`
  (`ragusa_gen::ALWAYS`); a table with a boolean column of that name is refused. Committed floor
  reports unchanged except the engine rev.
- **Novelty classes** (`NoveltyOracle::classify`): `implied` when one catalogue bound on the same
  target, operator and feature is at least as tight for every feature value (equal slope; or, for a
  feature the catalogue does not take in absolute value, slope and constant both no looser). This is
  sufficient, not necessary: facts such as arc index ≥ 2 are not used. Otherwise
  `same_pair_not_implied` when the catalogue relates the two columns, `new_pair` when it does not,
  `outside_catalogue` for any other column or a bound with more than one feature. `new_pair` says
  nothing about the literature beyond this catalogue.
- **Report.** `ragusa floor <table> --target T --oracle knots` writes `novelty.json` and `novelty.md`
  next to the floor report, with classes for all candidates as the baseline and for survivors, and
  fails when the gate fails. Reports on tables under `external/` now default to
  `external/results/floor/`, so KnotInfo-derived output cannot land in the repository by default.

### 2026-09-14 — Sprint 7 phase C: attribution without re-runs (G-B8)

Leave-one-out attribution re-ran every filter once per object: 674,688 re-runs per stage on the Fano
table. Each rule now states which single removals flip its decisions, from its own partition:

- **validity**: a discarded candidate flips exactly when its counterexample is unique;
- **sharp coverage**: winning on an object depends on that object alone, so a representative flips
  exactly when its witness is unique;
- **table duplicates**: no alias can flip; a representative flips at `r` when an earlier
  representative's key differs from its own on `r` only. Pairs are found through 64 block hashes
  and every hit is verified exactly. The search is quadratic in representatives per (target,
  operator), so above 10⁹ pairs the stage reports `"floor": {"computed": false}` instead of a
  number;
- **generality**: from the set differences of hypothesis coverage within each conclusion group
  (rule in the `Generality` doc comment).

The re-run stays as the reference (`floor::leave_one_out`) and is what the tests compare against.

Mutation check: four rules were sabotaged one at a time. Three made the random-table test fail. The
fourth, "a discarded candidate is not rescued at `r` if another candidate would newly dominate it at
`r`", survived because it can never fire: a rescue at `r` needs `r ∉ A`, a new dominance at `r`
needs `r ∈ A`. The clause was removed.

Scale fixes found on the way, none changing output: table classes are hashed then compared exactly,
instead of storing candidates × 2 × objects key words; generality stores coverage bits only for
candidates sharing a conclusion; the chance baseline keeps tight objects as lists, not n-bit masks.

### 2026-09-14 — Sprint 7 phase B: evaluation on demand, interval semantics (G-B7)

- **No stored candidates × objects arrays.** `Evaluated<'t>` borrows the table and compiles each bound
  to column slices; right-hand side and slack are computed when asked. On the Fano table the dense
  form would have needed about 33 GB.
- **Interval rules.** The right-hand side is an interval (each term takes the end its coefficient's
  sign selects). Violated: hypothesis holds and the slack's upper end is below −tolerance. Tight:
  hypothesis holds, every involved cell is a point, |slack| ≤ tolerance. Missing values are neither.
  An unknown hypothesis value does not hold.
- **Filters.** Table duplicates key on both ends of the right-hand side. Sharp coverage: an upper bound
  wins on an object only when its largest right-hand side is below every competitor's smallest by
  more than the tolerance (mirror for lower bounds). Both reduce to the old rules on points, which is
  what G-B7 checks.
- **Generators** fit only on exact cells: a missing or interval cell leaves the object out of the hull
  and LP fits (previously `GenError::MissingValue`, now removed) and gives NaN in ratios, as before.
- **Implication scan** takes the known-value mask: an object refutes `P_i ⇒ P_j` only if `P_j` is
  known false there.

### 2026-09-14 — Sprint 5: the magma corpus against the ETP implication graph (G-B5)

Data vendored as data (not code) from parsimagma: `implications.bits.gz` (ETP, Apache-2.0,
from Janota's Vampire run, arXiv:2508.15856), `hard_core.txt`, `summary.tsv`; checksums in
`engine/data/parsimagma/PROVENANCE.txt`. Scan: `ragusa_gen::implications::scan`, 22,028,942
ordered law pairs over 2786 magmas in 0.30 s (test build, opt-level 3).

- **Gate corrected before running.** The plan said G-B5 is "zero known-false implications
  emitted". That is backwards: the corpus has 2786 magmas, so false implications without a
  countermodel in it must be emitted, and that count is the corpus's floor. The check that can
  fail is soundness: a corpus magma refuting a pair the ETP graph proves true. It is 0.
- **Independent cross-check, exact:** false pairs 13,855,357; refuted by the corpus 13,574,842;
  hard core refuted 416 of 1062. All three equal parsimagma's `summary.tsv`, computed by different
  code (parsimagma's coverage matrix, ragusa's packed column scan).
- **Floor:** 8,454,100 pairs emitted. 8,173,585 are known true (all true off-diagonal pairs) and
  280,515 known false. The novelty oracle kills every true one, so nothing new survives: by
  construction on a complete oracle, as ROADMAP.md says. Known-true rate is 0.371 over all
  ordered pairs and 0.967 over emitted ones.
- **Attribution:** 2,841,746 pairs are refuted by exactly one magma, so a single removal would
  emit them. One magma dominates: the constant-zero magma on ℤ₂ (`linear/Z_m m=2 a=0 b=0`,
  x ◇ y = 0) is the sole refuter of 1,731,358 of them (61%); the two projections on ℤ₂
  (x ◇ y = y and x ◇ y = x) follow with 262,672 each. Mirror-image magmas (a and b swapped) get
  identical counts throughout the top ten, as the left/right duality of the law set predicts.
  Full list in `results/floor/magma_corpus_etp4694_implications/floor.md`.
- **Redundancy on this corpus:** only 906 of the 4694 law columns are distinct, so most laws
  cannot be told apart by these 2786 magmas at all.

### 2026-09-14 — Sprint 4: filters, floor report, G-B4

`ragusa floor graph_data --target independence_number` (all 23 features, 13 hypotheses, three
generators in txgraffiti2's order, seed 0, 1000 draws); report in
`results/floor/graph_data_independence_number/`. Filters are ragusa's own (order-independent,
same-direction comparisons, validity within 1e-9·max(1, |rhs|)); unit tests check them by hand on a
four-object table.

- **Pipeline:** 2544 candidates → validity 2510 (34 discarded) → table_duplicates 1526 (984 aliases)
  → sharp_coverage 19 → generality 19. Survivors: 12 lower bounds, 7 upper.
- **Negative, chance baseline:** sharp_coverage is not better than random acceptance of 19
  candidates from its 1526-candidate input. Objects with a tight survivor: 81 against a random mean
  of 60.4 in [23, 159] (P(random ≥ rule) = 0.198); hypotheses kept 9 against 10.0 in [8, 12]. A
  rule keeping only unique-best bounds is strict, and on this table strictness does not buy coverage.
  Same finding in kind as Project A's Dalmatian (census log above).
- **Per-object attribution:** object 286 (n = 2, m = 1, i.e. K₂) decides 14 validity decisions;
  object 4 decides 8 table_duplicates decisions; K₄ (row 77) 1. sharp_coverage's 11 fragile
  decisions spread over 9 objects.
- **G-B4** (`results/rows/gb4_graph_data_independence_number.json`, same source CSV on both sides).
  Pools share 1635 distinct bounds; 81 appear only in txgraffiti's (mostly the LP alternative
  optima of Sprint 3), 3 only in ragusa's.
  - Validity: agree on 1480; txgraffiti alone calls 155 false (95 convex_hull, 31
    linear_programming, 29 ratios), ragusa alone never. These are violations within ragusa's
    tolerance; their sizes were not measured here.
  - sharp_coverage vs Dalmatian, over bounds that reached sharp_coverage: 16 both, 3 ragusa only,
    713 Dalmatian only (641 of them lower bounds, which Dalmatian does not filter), 793 neither.
  - generality vs Morgan: all 19 agree.
  - Survivors vs txgraffiti's pipeline: 16 both, 3 ragusa only, 357 txgraffiti only.
- **Pinning caveat, fixed in the commit sequence:** the first floor report recorded engine rev
  `0f6a5ab`, which predates the filter code. The report was regenerated after committing the code.

### 2026-09-14 — Sprint 3: generators (G-B2, G-B3)

Fixtures: `calibration/src/ragusa_a/export_generator_fixtures.py`. txgraffiti keeps no expression
tree, so its candidates' coefficients are parsed back from `Property._binop` names into exact
fractions; every parse is checked against txgraffiti's own `rhs(df)`. The gates feed ragusa the
95 `graph_data` cells as pandas' default parser reads them, so parser effects are separated from
generator effects.

- **Solvers.** LP: HiGHS via `highs` 2.4.0 / `highs-sys` 1.15.0, built with cmake 4.4.3
  (Homebrew, approved), one thread. Hull: qhull via `qhull` / `qhull-sys` 0.4.1 with option
  `Qt`, as scipy 1.18.1 passes it. Both pinned exactly in `engine/Cargo.toml`.
- **Convex hull reproduces exactly**, not just within tolerance, although scipy and qhull-sys
  bundle separate qhull builds; that holds on the joggled (`QJ`) calls too.
- **LP differs by vertex, not by optimum.** CBC (txgraffiti) and HiGHS pick different optimal
  lines in 8 of 144 (§1.1) and 81 of 598 (full) bounds; every one has the same total slack
  within 1e-4. Which bound a solver returns is therefore not part of what txgraffiti "found".
- **txgraffiti emits a coefficient of about 8.36·10⁴⁸.** `second_largest_adjacency_eigenvalue`
  holds floating-point noise near zero on some graphs, so the ratio generator states bounds such
  as `(connected) → (independence_number <= 8360502766108716818096799791169600360057550143488 ·
  second_largest_adjacency_eigenvalue)`, 7 times in the full scope. On the correctly rounded table
  the same 2 of those change value (the noise itself is misparsed by pandas).
- **K₄ blocks two published results.** The paper (fetched from arXiv:2409.19379v2 through an
  automated page summary; quotes to be checked against the PDF before any write-up) lists
  `Z(G) ≤ α(G) + 1` for connected graphs with Δ ≤ 3, G ≠ K₄ (its Conjecture 4) and `Z(G) ≤ 2γ(G)`
  for connected cubic graphs, G ≠ K₄ (its Theorem 5). In `graph_data`, K₄ is row 77
  (Z = 3, α = 1, γ = 1) and is the only subcubic or cubic graph violating either. A
  single-predicate hypothesis cannot exclude it, so txgraffiti emits weaker bounds instead
  (e.g. `Z <= 5/6·α + 13/6` under `subcubic`). Row 77 is also the object behind all 5 fragile
  Morgan decisions in the Sprint 1 census. This is the per-object attribution B's floor report
  is meant to surface.

### 2026-09-14 — G-B1 and pandas' default float parser

`make reproduce-b` (exporter, then `cargo test`). Reference: `engine/fixtures/txgraffiti_manifest.json`.

- **pandas' default CSV parser is not correctly rounded on these files; txgraffiti computes on it.**
  Against `float_precision="round_trip"` (which equals Python's `float()`), 2115 of 2550365
  non-missing numeric values differ, in 41 columns of five tables (`nba_game_data` has none).
  Most are 1–3 ulps. The largest are truncation: `calabi_yau_data.sec_cy` text
  `0.0001976490020751953` parses as `0.0001976490020751`, 3516 ulps away; `qubits_data.mz_*`
  up to 190 ulps. The Rust loader is correctly rounded and matches round_trip bit for bit.
  Consequence for Sprint 3 (G-B2): TxGraffiti's candidates were fitted on the default-parsed
  values, so engine vs TxGraffiti comparisons need a stated tolerance or the default-parsed
  values as input; this is a known cause of mismatch, not a bug in either.
- **PROMPT.md §1.4's column counts do not match pandas + `auto_wrap`.** Numeric columns:
  `qubits_data` 19 (§1.4: 20), `calabi_yau_data` 44 (46), `nba_game_data` 45 (55). The gaps are
  string columns pandas cannot type as numeric (`input_type`; `lattice`,
  `triangulation_backend`; ten team/date/matchup columns). G-B1 checks against pandas, not §1.4.
- Engine builds offline from the local cargo cache (`engine/.cargo/config.toml`); no new crates
  were downloaded. `csv` is not cached, so the loader splits on `,` and rejects any `"` or CRLF;
  none of the six files contains either.

### 2026-09-14 — Sprint 1 census (numbers 1–4)

`python -m ragusa_a.census`; rendered in `calibration/note/calibration.md`. Scope: full
`graph_data`, target `independence_number`, generators in `txgraffiti2`'s default order.

- Pool: 2544 generated → 1716 after `remove_duplicates`; 244 of them false on the table.
- Partition (rep / alias / discarded): Dalmatian 779 / 14 / 923; Morgan 830 / 17 / 869;
  Sophie 1 / 0 / 1715. Sophie is degenerate here: the first pool candidate has hypothesis
  `connected`, which holds on all 340 objects.
- **Measured: Dalmatian does not filter lower bounds beyond validity.** All 691 true `>=`
  candidates are accepted; the 83 rejected ones are all false. Upper bounds: 88 of 781 true
  kept. This confirms the source-reading item below about `op in ("<=", "≤")`.
- **Negative, chance baseline:** apart from keeping no false conjecture (random: 95–125
  kept), Dalmatian is not better than random acceptance at the same count. Objects with a
  tight survivor: 248, inside the random range [232, 252]. Distinct table classes: 746,
  *below* the random range [754, 768]. Morgan: tight objects 253, above the range
  (P = 0.007); false kept 132, at the top of random.
- Floor: Dalmatian has 50 fragile decisions (2.9%) across 24 objects; 6 objects carry half
  of all flips; one of the top five is K₂ (row 286, n = 2, m = 1). Morgan: 5 fragile, all
  from row 77. Sophie: none.
- AutoGraphForge on the TxGraffiti pool: 132 known (arrow adapter, table columns),
  115 (`_GC2TABLE` columns), **0 without the adapter**. Statement syntax alone decides
  whether the filter sees anything.

### 2026-09-14 — shipped generators emit conjectures false on their own table

Full `graph_data` scope (target `independence_number`, all 23 features, all 13
predicates), each generator run alone by a throwaway script (scratchpad
`false_by_gen.py`, nothing written to `results/`). The census pool counts false
candidates after deduplication (244); these counts are before it.

| generator | raw | false on the table | example counterexample slack | apparent cause |
|---|---:|---:|---|---|
| `linear_programming` | 598 | 73 | about −6·10⁻⁸ | constants rounded to fractions such as `6882346/999999` |
| `convex_hull` | 1348 | 125 | about −2·10⁻¹⁵ | floating-point facet coefficients |
| `ratios` | 598 | 66 | exactly −1 | ratio skips rows where the feature is 0, but the bound still covers them (e.g. `α ≤ 13/6 · triameter` on a triameter-0 graph) |

Consequence: the partition's `discarded` class mixes false candidates with
redundant true ones, so every table reports the false count separately.
`dalmatian_accept` is the only one of the three rules that checks validity.

### 2026-09-14 — txgraffiti 0.4.1 source reading (unmeasured; to test in Sprint 1)

Three of these points (the whole-pool comparison, `>=` candidates compared with upper bounds, the
solver lookup) were put to Randy Davila on 2026-10-04 as questions about intent, not answered so
far. They describe version 0.4.1 as we read and measured it; they are not claims that the code is
wrong.

- `heuristics/delavina.py` is `sophie_accept` (new hypothesis-cover rows) and
  `heuristics/davila.py` is `morgan_accept` (no strictly more general
  hypothesis with the same conclusion). PROMPT.md's "coverage" and "dominance"
  labels map onto these; use the function names in every table.
- `processing/fajtlowicz.filter_with_dalmatian` calls `dalmatian_accept`
  against the whole candidate pool (`c != new_conj`), not the accepted set, and
  `!=` on `Conjecture` compares names.
- `dalmatian_accept` collects matching old bounds with `op in ("<=", "≤")`
  whatever the new conjecture's operator, so a `>=` candidate is compared
  against upper bounds. Measure how many lower bounds pass only through this
  path before describing it.
- `remove_duplicates` keys on `(hypothesis.name, conclusion.name)`, the
  rendered strings, so equal bounds written differently are not merged.
- `filter_with_morgan` depends on input order: it removes earlier accepted
  items when a later one is more general.
