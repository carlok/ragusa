# Roadmap

Decided 2026-09-14, after Sprint 1. Supersedes the domain choice in PROMPT.md §6.2 item 5
(magma signatures as the target). The rest of PROMPT.md stands.

## The shape

| stage | what | answers | discovery possible? |
|---|---|---|---|
| **A** (done) | census of TxGraffiti / AutoGraphForge filters, their code | what existing filters discard; the baseline B must beat | no, it is a measurement |
| **B-control** | ragusa on TxGraffiti's graph tables | does ragusa reproduce the known generators and report a correct floor? | no |
| **B-ETP** | ragusa on parsimagma's magma signatures (4694 laws) | is the floor exact against a complete oracle? | no: ETP settles every implication |
| **B-frontier** | ragusa on a domain ETP-style proving does not cover | new conjectures with a published floor | **yes, the real project** |

Why not a SAT/prover tool: provers decide statements someone already wrote down. Where the
statements can be enumerated (law pairs), a prover pipeline is the right tool and ETP shows it.
ragusa is for domains where the statements must first be guessed from data: fitted bounds,
invariants mixed with properties. Its survivors are inputs for provers and people, not proofs.

parsimagma is used as **data**, not as a code dependency: B reads its signature corpus and the
ETP implication matrix from files. Magma order 5 or 6 is not a frontier for ragusa; pure
implications there remain provers' territory.

## Sprints

Status (2026-11): sprints 0–12 are done and none is in progress. The repository went public on
2026-11-23.

| sprint | stage | content | gate |
|---|---|---|---|
| 0–1 | A | done (see CONTROLS.md, calibration note) | G-A0..G-A5 |
| 2 | B | done: table model (numeric + bit-packed boolean), loaders for TxGraffiti CSV and parsimagma files, pinning | G-B1 pass |
| 3 | B-control | done: ratio, LP (HiGHS), convex hull (qhull) generators reproducing txgraffiti's candidates | G-B2 pass, G-B3 pass |
| 4 | B-control | done: filters returning partitions, floor report emitted by every run | G-B4 reported |
| 5 | B-ETP | done: implication generator over the boolean block, ETP oracle | G-B5 pass (soundness, exact match with parsimagma's counts) |
| 6 | frontier scouting | done: pages in `docs/scouting/`; **chosen 2026-09-14: KnotInfo (primary), toric canonical Fano 3-folds (scale test); matroids ≤ 9 elements in reserve** | load tests done (`docs/scouting/loadtest-*.md`) |
| 7 | B-frontier | done: the engine at frontier scale (intervals, evaluation on demand, attribution without reruns, knot oracle, held-out objects) and first runs | G-B6..G-B10 |
| 8 | B-frontier | done: held-out sweep on two tables, Fano profiling, literature check | reported (CONTROLS.md) |
| 9 | B-frontier | done (partial fix): an acceptance rule that does not select overfitted bounds | CONTROLS.md 2026-09-15 phase C |
| 10 | B-frontier | done: sharp coverage replaced; `greedy_touch` is the default | CONTROLS.md 2026-09-15 |
| 11 | B-frontier | done: first real harvest on the frontier tables | CONTROLS.md 2026-09-15 |
| 12 | B-frontier | done: artefact checks and a candidate dossier for an external model | range-edge check still weak (CONTROLS.md) |
| later | write-up | rendered from results; negatives as loud as positives | doc-number lint |
| later | optional | Lean export | out of scope for now |

## Sprint 12: artefact checks and a candidate dossier for an external model (done; range-edge check still weak, see CONTROLS.md)

Sprint 11's literature check found most KnotInfo "candidates" to be artefacts of the table: mirror
images it does not store, knots just past 13 crossings, empty cells, columns recorded only up to 12
crossings. The surviving candidates go first to an external language model, then to a person.

| phase | content | output |
|---|---|---|
| A. artefact checks | `ragusa check <table> --bounds FILE` evaluates any bounds on any table (verdicts, tight objects, tight objects at the maximum of an edge column); a mirrored KnotInfo export (sign-dependent invariants negated, chirality-dependent properties unknown); `--complete-knots` fills empty cells with intervals implied by the catalogue; coverage of each hypothesis by crossing number | engine command, exporter option, tests |
| B. re-check | the Sprint 11 shortlist through the four checks; survivors grouped into distinct relations (identities, one relation under several hypotheses, the same inequality solved for either column) | `external/results/` |
| C. dossier | `external/candidates/knotinfo_le13/`: README for the model (context, conventions, answer format), one Markdown file per candidate, a combined LaTeX file and its PDF | git-ignored: regenerated from the KnotInfo export, which stays out of the repository |

## Sprint 11: first real harvest (done; see CONTROLS.md 2026-09-15)

At the start of this sprint the vetted pipeline (`--min-tight sqrt`, `greedy_touch`) had not yet run on the
frontier tables.

| phase | content | output |
|---|---|---|
| A. attribution at scale | exact `greedy_touch` attribution without full reruns: removing an object changes the greedy result only through the pick that first covered it, so only objects where that pick can change (gain 1, or a competitor not tight there with gain G, or G − 1 at an earlier position) are rerun | derived attribution equal to reruns on random tables and `graph_data`; no budget cut-off |
| B. frontier runs | vetted pipeline on KnotInfo ≤ 13 (every target, knot oracle, held-out check) and on Fano (several targets, `TRUE` hypothesis) | reports in `external/results/` (KnotInfo) and `results/` (Fano, CC0) |
| C. shortlist | survivors refuted on no held-out object, not implied by the catalogue, ranked by evidence | shortlist files |
| D. literature | deep search on the top candidates, reviewed by a person before any claim of novelty | `docs/literature/` |

## Sprint 10: replace sharp coverage (done; greedy_touch is the default, see CONTROLS.md 2026-09-15)

Decided 2026-09-15 after Sprint 9: the default pipeline does not gain the touch stage; the touch
threshold should scale with table size; sharp coverage should be replaced by a redundancy rule that
does not select for extremes.

| phase | content | output |
|---|---|---|
| A. scaled threshold | `applicable_training` in `candidates_heldout.csv`; rule study adds thresholds relative to the objects where the hypothesis holds (fractions, √n); chosen on seeds 0–4, confirmed on 5–9 | `results/rows/rule_study_graph_data_*`; CONTROLS.md |
| B. redundancy rules | `maximal_touch` (discard a bound whose tight set is strictly inside another same-target, same-direction bound's; exact attribution from set differences) and `greedy_touch` (greedy cover of tight objects; attribution by reruns within a budget); `--redundancy sharp\|maximal_touch\|greedy_touch`; held-out sweep on both tables, chosen on seeds 0–4, confirmed on 5–9 | sweep JSONs; CONTROLS.md |
| C. default | the confirmed rule becomes the default redundancy stage; committed `graph_data` report regenerated and the change logged; sharp coverage stays available | new committed report, gates |

## Sprint 9: an acceptance rule that does not overfit (done; partial fix, see CONTROLS.md 2026-09-15 phase C)

Sprint 8: sharp coverage keeps bounds refuted on held-out objects at 34% (`graph_data`) and 39%
(KnotInfo ≤ 12) against 24% and 17% after validity.

| phase | content | output |
|---|---|---|
| A. per-candidate detail | with `--holdout`, `candidates_heldout.csv`: for every candidate valid on training, its duplicates class, sharp-coverage witness count, tight objects on training, largest denominator, and held-out verdict counts | the data rules are compared on, without rerunning the pipeline per rule |
| B. rule study | `rule_study.py` evaluates candidate rules on the duplicates representatives: at least k witnesses, at least k tight objects, bounded denominators, combinations; pooled refuted rate of tested bounds, bounds kept, per-run comparison with the duplicates pool (the rate of a random subset of the same size). Rules are chosen on seeds 0–4 and confirmed on seeds 5–9, never the reverse | `results/rows/rule_study_graph_data_s0-4.json` and `_s5-9.json`; KnotInfo in `external/`; CONTROLS.md |
| C. engine rule | the chosen rule as an engine filter, with exact attribution (equal to reruns on random tables) and the floor report; committed `graph_data` report regenerated only if the default pipeline changes, with the change logged | new filter, gates |

## Sprint 8: does the floor report hold up? (done; results in CONTROLS.md, 2026-09-15 entries)

Sprint 7 ended with a negative result on one table, one target and one seed, a 21-minute Fano run
nobody has profiled, and 47 Fano survivors nobody has compared with the literature.

| phase | content | output |
|---|---|---|
| A. held-out sweep | `heldout_sweep.py`: every numeric target of `graph_data` and the catalogue columns of `knotinfo_le12`, five seeds each, 20% held out; refutation rate per stage, pooled and per run | `results/rows/heldout_sweep_graph_data.json` (committed); KnotInfo aggregate in `external/` and CONTROLS.md |
| B. profiling | per-phase wall-clock timings (`--timings FILE`, never in `floor.json`); Fano rerun; fix the dominant phase if it is not inherent | timings in CONTROLS.md; floor reports unchanged |
| C. literature | Fano survivors not restating Riemann–Roch, checked against Kasprzyk and related work by web search | `docs/literature/fano3c_degree_survivors.md` |

## Sprint 7: the engine for the frontier (phases, one commit each) (done)

The load tests and the scale of the Fano table (674,688 rows) change what the engine must do. Dense
evaluation (candidates × objects stored) and attribution by re-running each filter once per object
do not survive that scale, so both are redesigned before the first frontier run.

| phase | content | gate |
|---|---|---|
| A. table (done) | numeric cells as closed intervals (point = lo = hi), missing values, exact rationals on load; a neutral `ragusa CSV` format; Python exporters for KnotInfo and the Fano table with manifests | G-B6: exported tables load bit-exactly against their manifests; `fixtures/pins.json` unchanged for every existing table |
| B. evaluation (done) | slack computed on demand from the table (no stored candidates × objects arrays); interval semantics (violated only if certain, tight only on exact cells); generators fit on exact cells only | the committed `graph_data` floor report reproduces byte for byte (except the engine rev) |
| C. attribution at scale (done) | per-filter attribution computed from witnesses instead of reruns, where the rule allows it exactly; explicit "not computed" otherwise | analytic attribution equals rerun attribution on `graph_data` and on hand-built tables |
| D. knot oracle (done) | Jabłonowski's 47 inequalities (arXiv:2605.22652) encoded against KnotInfo columns; a novelty stage in the report | encoded catalogue checked on the knots themselves (no catalogue inequality refuted by the table) |
| E. held-out (done) | split objects (by crossing number, or seeded fraction); survivors re-checked on the held-out part | reported, not forced |
| F. runs (done; see CONTROLS.md phase F log) | KnotInfo up to 12 crossings; the full Fano table | floor reports; KnotInfo outputs stay git-ignored, since the maintainers ask that the database not be reposted (2026-10-02) |

## Sprint 6: frontier scouting (done)

A domain qualifies when it has all four:

1. a large, already-enumerated object database, downloadable and licensed for reuse;
2. many numeric invariants and many boolean properties per object;
3. a live human literature of inequalities between those invariants;
4. no complete oracle (so conjectures can be new), ideally a partial one (for the novelty floor).

Method: a deep literature and database search run at the start of the sprint, with every
candidate, count and license taken from a cited source. The list below is only a seed from
memory and carries no weight in the choice: finite groups (GAP SmallGroups), LMFDB (number
fields, elliptic curves), knots (KnotInfo), matroids (enumerated small-matroid databases),
graphs at scale (all small graphs, House of Graphs). The search may add or drop any of them.

Per candidate, record: object count, invariant and property counts, access and license, a
literature pointer to published inequalities, what partial oracle exists, and a 30-minute load test.
Pick one; keep the rest as reserve.
