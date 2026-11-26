# Handoff — automated-conjecturing filter census and extension study

**Task for the receiving model: DESIGN FIRST.** Produce a research design and a repository plan.
Do not start implementing. Everything below is either measured in this session (marked **[meas]**,
with how it was measured) or read from a source (marked with its URL). Nothing here is recalled
from memory. Where a number is uncertain, it says so.

---

## 0. The verdict this handoff starts from

The direction is right. The balance has changed since the first draft, and the change is the most
important thing in this document.

**There are two pieces of work, and the small one is not the paper.**

- **A — the calibration (one day).** Measure what the existing conjecturing systems throw away, on
  one table, with the shipped code. This is *not* a publication. It exists to tell you which
  filters do real work, to give you a denominator to compare your own engine against, and to
  supply the prior-art citations you owe anyway.
- **B — the engine (the actual project).** A Rust engine that unifies two paradigms nobody has
  noticed are the same one, handles a regime the existing tools cannot, and **reports its own
  floor by construction**.

An earlier draft made A a six-domain, three-filter, permuted-order publication. That was
over-scoped. As a paper it is an audit of someone else's Python, it is contestable on exactly the
grounds this session already hit (wrong stage, truncated feature set, lossy re-parsing), and it is
not where the author's advantage lies.

But A cannot be skipped either, and the reason is precise: **an engine built without a census has
no denominator of its own.** It would be the next system publishing survivors without a kill
rate — the exact failure this whole line of work exists to correct. A day of calibration buys
immunity from that.

### 0.1 The observation B is built on

**TxGraffiti's snapshot table and parsimagma's signature vector are the same object.**

A snapshot table is objects × (numeric invariants + boolean predicates). A magma signature is an
object × 4,694 booleans — the same table with an empty numeric block and an enormous boolean one.
Graffiti-lineage tooling assumes a few dozen columns and a few hundred rows, held in pandas.
Signature tooling assumes bit-vectors and no numeric block at all. **Nothing spans both**, and the
interesting regime is precisely the span: a few dozen numeric columns *and* 10⁴–10⁶ boolean
columns, over 10³–10⁷ objects.

Nobody has run Graffiti-style conjecturing over a 62,576-bit signature space, because with pandas
nobody could. That is not a speed gap; it is a capability gap, and it is the reason to build rather
than to audit.

### 0.2 The product idea

**An engine whose output includes its own floor.** Every filter's kill rate, per-object
attribution, and a chance baseline, emitted as part of the run rather than reconstructed afterwards
by someone else. No conjecturing system does this. It turns A's result from a separate paper into a
feature of B, and it is the one design decision that makes this engine different from the six that
already exist.

Domain is secondary — quasigroup laws, matroids, finite semigroups, the shipped polytope table,
magma signatures. What matters is the regime, not the objects.

---

## 1. What was measured in this session

### 1.1 TxGraffiti package filter kill rates **[meas]**

Environment: `pip install txgraffiti` → version **0.4.1**; Linux container; CBC unavailable (the
bundled PuLP solver is a macOS binary — `pulp/solverdir/cbc/osx/i64/cbc`, exec-format error on
Linux), so the `linear_programming` generator was **skipped**. Generators used: `convex_hull` and
`ratios`. Data: the package's shipped `txgraffiti.example_data.graph_data`. Target invariant:
`independence_number`. Scope: first 12 numeric features × first 6 boolean predicates, `feature_size=1`.

| stage | kept | killed | share |
|---|---:|---:|---:|
| generated | 453 | — | — |
| after `remove_duplicates` | 384 | 69 | 15.2% |
| **after `filter_with_dalmatian`** | **216** | **168** | **43.8%** |
| after `filter_with_morgan` | 141 | 75 | 34.7% |
| **overall** | **141** | **312 / 453** | **68.9%** |

Wall clock: generation 3.3 s, Dalmatian 20.3 s, total ≈ 24 s on one core.

**Caveats that must be carried into any write-up.** One target invariant of ~24 numeric columns.
Two of three generators. A truncated feature and predicate set. On a Mac the bundled CBC will load,
so `linear_programming` will run and the numbers **will differ** — treat the table above as a
feasibility demonstration, not a result.

**Why Dalmatian is slow:** `processing/fajtlowicz.py` compares each candidate against every other
(`others = [c for c in conjectures if c != new_conj]`), i.e. O(M²) with a full re-evaluation per
comparison. The paper's own complexity note says O(MN) with cached sharp sets and hashing. The
released code does not cache. This is both a performance fact and a small contribution.

### 1.2 AutoGraphForge's novelty filter on its own published survivors **[meas]**

Repo cloned from https://github.com/JanPastorek/AutoGraphForge — 309 MB total (`results/` 195 MB,
`.git` 21 MB). `pipeline/novelty.py` imports with only scipy; `KNOWN_THEOREMS` has exactly **559**
entries, `LOWER_BOUNDS` has 5.

Ran `pipeline.cegis_novelty.classify_statement` with that module's own 44-entry `_GC2TABLE` column
map over all **6,522** statements in `results/cegis_results.json`:

- **154 flagged as already-known (2.36%)**, 0 errors, **6.2 s** — i.e. **0.956 ms per conjecture**.
- Most frequent reasons: `rad ≤ diam` (8), `a(G) ≤ δ` Fiedler (8),
  `power_domination_number ≤ ½·total_zero_forcing_number` (8),
  `independence_number ≤ annihilation_number` (7), `diam ≤ 2·rad` (7), `Δ ≤ n−1` (4).

**Three caveats.** (i) The shipped survivors have `inequality: null` — only statement strings
survive serialisation, so everything is re-parsed, which is lossy in both directions. (ii) The
pipeline may invoke the filter at a different stage or with a different vocabulary; check
`graphconj_cli.py` before claiming anything. (iii) This is a *different filter* from TxGraffiti's
— see §2.

### 1.3 Denominators that are already in the AutoGraphForge repo but not in its paper **[meas]**

- `results/unified_report.json` and `results/parallel_report.json` (identical figures):
  `generated_total: 494`, `generated_by_method: {txgraffiti: 209, conjecturing-expr: 277,
  conjecturing-prop: 8}`, `refuted: 162`, `survivors: 332`, **`novel_survivors: 332`**,
  `elapsed_s: 1982.8`, `rounds: 3`, `final_db_graphs: 348530`.
  **The novelty filter killed zero in that run.** That does not sit comfortably with §1.2's 154,
  and resolving the tension is a genuine research question, not a gotcha.
- `results/lifted.json`: `total: 2695`, `subsumed: 141`, `decorative: 367`, `refuted_by_corpus: 0`,
  `lifted: 2328`, `low_support: 977`, `kept: 2268`.
- `results/conjecture_report.json`: `total_generated: 12`, `falsified: 3`, `survived: 9`,
  `formalized: 9`, `proven: 0`, `proof_failed: 9`, `elapsed_s: 66.5`.

So the repo is more instrumented than the paper. Read all of `results/` before asserting any gap.

### 1.4 What the txgraffiti package actually contains **[meas]**

Six shipped domain tables — **TxGraffiti is not graph-specific**:

| table | rows | cols | bool preds | numeric |
|---|---:|---:|---:|---:|
| `graph_data` | 340 | 37 | 13 | 24 |
| `integer_data` | 149 | 16 | 7 | 9 |
| `polytope_data` | 496 | 18 | 5 | 13 |
| `qubits_data` | 216 | 30 | 10 | 20 |
| `calabi_yau_data` | 175 | 65 | 19 | 46 |
| `nba_game_data` | 65,698 | 55 | 0 | 55 |

Module inventory of note:

- `heuristics/` — `fajtlowicz` (Dalmatian), `delavina` (coverage), `davila` (strict-dominance).
  Three *named, separable* acceptance rules. Each has its own kill rate. Nobody has published any
  of them.
- `processing/` — `fajtlowicz` (`filter_with_dalmatian`), `davila` (`filter_with_morgan`),
  `preprocessors`, `postprocessors`, `registry`.
- `generators/` — `optimization` (`linear_programming`), `convex_hull`, `ratios`, `registry`.
- `systems/` — `txgraffiti2` (the documented pipeline), `christine` (undocumented in the paper).
- `graffiti3/` — a whole second engine with ~11 runner families (`lp`, `poly`, `ratio`,
  `ratio_lift`, `root_log`, `exp_exponent`, `nonlinear_multi`, `mixed`, `heuristic`, `constant`)
  plus a `sophie` module (the sufficient-condition heuristic from Graffiti.pc).
- `export_utils/lean4.py` — **a Lean 4 exporter** (`conjecture_to_lean4`, `conjectures_to_lean`,
  `necessary_conjecture_to_lean`, plus fraction/type-casting machinery). This is the bridge to
  Carlo's existing Lean and Mathlib work. AutoGraphForge also ships a `lean_project/`.
- `export_utils/sql.py`, `logic/` (`conjectures`, `inequalities`, `predicates`, `properties`,
  `tables`), `playground/`, `utils/safe_generator.py`.

### 1.5 The live web application **[meas, via browser]**

https://txgraffiti.streamlit.app/ — sidebar pages: **Welcome**, **About**, **Generate
Conjectures**, **Generate MIP Conjectures**, **Generate Graph Product Conjectures**, **Graph
Calculator**. Example deep link the user supplied:
https://txgraffiti.streamlit.app/Generate_Graph_Product_Conjectures

The MIP and graph-product generators are **not** obviously present in the PyPI package under those
names — verify before assuming either way.

From the About page: the database began as "about 100 or so handwritten edge lists made from
examining images of graphs available on the Wolfram Mathworld website", with a deliberate bias
toward cubic graphs, and "the actual database used by TxGraffiti does not consist of the edge lists
of these graphs" (it stores precomputed invariants). The paper describes the research table as
"currently on the order of a few hundred" curated graphs — consistent with the shipped 340 rows,
but **not proven to be the same table**.

---

## 2. The distinction that everything depends on

There are **two different kinds of filter**, and conflating them produces false claims:

| | asks | implemented in | reports its kill rate? |
|---|---|---|---|
| **Novelty / known-results** | "is this already a published theorem?" | AutoGraphForge: 559 curated relations, closed under transitive composition and linear substitution, LP feasibility test | no |
| **Redundancy / significance** | "does this add a new equality witness the bank doesn't have?" | TxGraffiti: static-Dalmatian, DeLaViña coverage, Davila dominance; and the whole Graffiti lineage | no |

The Graffiti lineage has *only* the second kind. AutoGraphForge is, as far as this session could
find, the first system with a real oracle of the first kind. **So there are at least two unreported
kill rates, over different populations, and the design must name which one each number refers to
every single time.**

---

## 3. The three-artifact fork

"TxGraffiti" names three different things whose numbers may differ:

1. **The paper's design** — an abstract pipeline. Specifies a three-way partition
   (representative / alias / discarded) and reports no counts.
2. **The PyPI package** `txgraffiti` 0.4.1 — versioned, pinnable, installable, reproducible by a
   reviewer with no server. What §1.1 measured.
3. **The website** — more generators, a table that may or may not be the shipped one, a moving
   target, and someone else's compute.

The paper itself says the website "is one system-level instantiation of this design."

**Design decision, pre-committed: measure the package, and say so in the title.** Use the website
only to (a) enumerate what capabilities exist, and (b) hand-check a handful of outputs. Do not run
a sweep against the hosted app.

---

## 4. People, etiquette, and citation obligations

This work touches other people's live research. The etiquette is part of the design.

### Randy Davila — TxGraffiti
- Author of arXiv:2409.19379 (sole author on the arXiv listing), affiliations given as
  First Principles, Toronto, and Department of Computational Applied Mathematics & Operations
  Research, Rice University.
- Contact addresses are listed on the site and in the paper; they are not repeated here.
- **The site's Terms of Usage grants what is needed, explicitly:** "I do not ask to be on any
  papers that result from using TxGraffiti. Still, I would appreciate a mention in the
  acknowledgments and a citation given to the TxGraffiti website. I am always happy to hear about
  the results generated by using TxGraffiti or to work collaboratively with you."
- **Obligations that follow, and they are not optional:** acknowledge him, cite the website
  *and* the paper, and email him. The email is a courtesy that also gets a question answered that
  only he can answer: *is the website's snapshot table the same as the package's shipped
  `graph_data`, or a larger curated one?*
- The right tone with him is not audit. He defined a three-way partition and did not publish the
  counts; the ask is "here is the table your paper implies — does it match what you see?"

### Ján Pastorek — AutoGraphForge
- arXiv:2609.03478, 3 Sep 2026; code at https://github.com/JanPastorek/AutoGraphForge.
- Ten days old at the time of writing and building in the same direction. Treat as a peer working
  the same seam, not as a subject. Contact before publishing anything about his numbers.
- The 154 from §1.2 is a gift to send him, as a CSV of statement plus matched relation — not a repo
  link. Lead with the list, admit the filter may have been pointed at the wrong stage, and ask.
- A draft email to him already exists in this conversation's history; reuse its shape.

### The lineage, to cite and to not re-derive
- **Siemion Fajtlowicz** — Graffiti, and the Dalmatian heuristic.
- **Ermelinda DeLaViña** — Graffiti.pc, the `dalmatians` component, the Sophie heuristic.
- **Craig Larson and Nico Van Cleemput** — *Automated conjecturing I: Fajtlowicz's Dalmatian
  heuristic revisited*, Artificial Intelligence 231 (2016),
  https://www.sciencedirect.com/science/article/pii/S0004370215001575 — the modern formal treatment,
  and the `Conjecturing` Sage program. Series runs I–VII.
- **Simon Colton** — HR / automated theory formation, over finite algebras as well as graphs;
  PhD Edinburgh 2001, https://era.ed.ac.uk/handle/1842/35263.
- **Douglas Lenat** — AM / EURISKO, and **Graeme Ritchie & Frank Hanna**, *Why AM and EURISKO
  Appear to Work*, Artificial Intelligence 23(3), 1984 — the critique that predicts exactly this
  class of self-deception. Cite it as the reason the guards exist.
- Graffiti's conjectures have a large downstream literature; do not attempt to survey it.

---

## 5. References with URLs

- TxGraffiti paper: https://arxiv.org/abs/2409.19379 — v1 28 Sep 2024, **v2 11 May 2026**
  (Annals of Mathematics and Artificial Intelligence, DOI 10.1007/s10472-026-10005-5). Cite v2.
- TxGraffiti site: https://txgraffiti.streamlit.app/
- TxGraffiti package: PyPI `txgraffiti` (0.4.1 at time of writing);
  repo https://github.com/RandyRDavila/TxGraffiti
- Earlier TxGraffiti conjectures paper: https://arxiv.org/pdf/2104.01092 (also AJC v84 p258)
- AutoGraphForge: https://arxiv.org/abs/2609.03478 ; https://github.com/JanPastorek/AutoGraphForge
- Larson & Van Cleemput 2016: https://www.sciencedirect.com/science/article/pii/S0004370215001575
- Ritchie & Hanna 1984:
  https://citeseerx.ist.psu.edu/document?repid=rep1&type=pdf&doi=37ab31f586cdc1efc3bf0dcc9ba52f644077e466
- Colton HR: https://era.ed.ac.uk/handle/1842/35263
- Field survey, Aug 2025: https://arxiv.org/abs/2508.20825 (does not raise the kill-rate question)
- LeanConjecturer, for the one existing formal-setting novelty floor:
  https://arxiv.org/html/2506.22005v1 — 12,289 generated → 10,950 valid → 4,130 novel via `exact?`
  → 3,776 surviving `aesop`. **Do not re-derive this; cite it as precedent for the method.**
- Venue for the census half: NeurIPS Datasets & Benchmarks / Evaluations track, whose CfP solicits
  saturation studies and states submissions need not introduce a new model. AITP is the small,
  exactly-right audience.

---

## 6. The design task

Produce a design document, not code.

### 6.1 A — the calibration (one day, not a paper)

Scope it down hard. One table: `graph_data`. One filter ordering. The three acceptance rules
(`heuristics/fajtlowicz` Dalmatian, `heuristics/delavina` coverage, `heuristics/davila` dominance)
measured **separately**, because they are separable and nobody has published any of them.

Deliver four numbers and stop:

1. the three-way partition — representative / alias / discarded — per rule, with exact denominators;
2. the **object-database floor**: how many accept/reject decisions are determined by a handful of
   extremal objects. Larson & Van Cleemput note output size is bounded by database size; at 340
   rows this is seconds of compute and has never been instrumented;
3. a **chance baseline** — what a random acceptance rule at the same acceptance rate kills. Without
   it "Dalmatian kills 44%" means nothing;
4. the **AutoGraphForge novelty rate** over the same corpus, labelled as the other kind of filter
   (§2), with Pastorek credited.

This is the calibration that tells you what B should measure about itself. Publish it as a short
note or fold it into B's paper as a related-work table — decide later, not now. **Do not expand it
to six domains, permuted orderings, or a caching reimplementation of someone else's filter.** Those
were in the earlier draft and are cut.

### 6.2 B — the engine (the project)

A Rust engine over the unified regime of §0.1. The design must specify:

1. **The data model.** One table type carrying a numeric block (dense `f64`, objects × invariants)
   and a boolean block (bit-packed, objects × predicates, where predicates may number 10⁴–10⁶).
   Both blocks optional. A magma signature corpus and a TxGraffiti snapshot must both load into it
   without special-casing.
2. **Generators.** Start by reproducing the documented shapes — affine upper/lower bounds fitted by
   LP, convex-hull facets, ratios. These are the ones with a published description to check
   against. Do not invent new generator families before the reproductions pass §6.3.
3. **Filters, as first-class measured objects.** Every acceptance rule is a trait implementation
   that returns a *partition*, not a filtered list. The kill rate is a return value, not something
   recovered later by diffing lengths.
4. **The floor report.** Emitted by every run: per-filter partitions, per-object attribution,
   chance baseline, and the pinning block (data checksum, generator set, seeds, version).
5. **A domain to point it at.** *(Decided 2026-09-14, see `ROADMAP.md`: magma signatures and
   graphs are calibration and control; the target is a non-ETP frontier domain chosen in a
   scouting sprint.)* Pick one and justify it. Quasigroup laws tie to the Le Floch thread;
   magma signatures reuse parsimagma's data; matroids and finite semigroups are unexplored by any
   Graffiti-lineage system. Graphs are the crowded choice and should be the *control*, not the
   target.
6. **Formalization, optional and last.** `txgraffiti.export_utils.lean4` already emits Lean 4, and
   AutoGraphForge ships a `lean_project/`. If this axis is taken, Appendix A becomes mandatory.
   AutoGraphForge's own honest prior: 9 formalized, 0 proven, 9 proof_failed.

### 6.3 Verification discipline — non-negotiable

- **Reproduce a published number with the same code before trusting it on an unpublished one.**
  Candidates: AutoGraphForge's `unified_report.json` figures; the α ≤ μ result for r-regular graphs
  that the TxGraffiti paper cites from Caro et al.; any conjecture the paper states as an example.
- **Every number in any write-up is generated into the document by a script.** None typed by hand,
  including in prose.
- **No hand-rolled numerics or optimisers.** PuLP / OR-Tools / HiGHS / scipy / ndarray. A previous
  project in this family reported a hand-written single-linkage clustering as a positive result;
  re-run with a real library it scored worse than random.
- **A chance baseline for every headline.**
- **Report negatives as loudly as positives.**
- **Pin everything:** version, table checksum, generator set, feature and predicate subset, target
  column, seeds. §1.1's table shows how much a result moves with these.

### 6.4 Deliverables the design must specify

1. Repository layout, A and B separate, separate reproduce targets.
2. A results schema — one row per (system, table, target, generator, filter, stage) — from which
   every published table is recomputed.
3. The calibration note (A) and the engine's floor-report format (B).
4. `CONTROLS.md` — each gate, what it reproduced, its result, and any routine written, failed a
   control with, and discarded.
5. Two emails: Davila (question plus acknowledgment plan) and Pastorek (the 154 as CSV).
6. An acknowledgments and citation section drafted *before* the results, so §4's obligations are
   structural rather than an afterthought.

---

## 7. Explicit non-goals

- Do not run sweeps against https://txgraffiti.streamlit.app/ . Enumerate its capabilities,
  hand-check a few outputs, and otherwise leave the man's server alone.
- Do not publish anything about Pastorek's numbers before contacting him.
- Do not re-derive LeanConjecturer's formal-setting novelty floor; cite it.
- Do not build a concept inventor. Ritchie & Hanna 1984 and a previous project in this family both
  record how that ends.
- Do not use an LLM inside the measurement pipeline for A. B may, and must say where.
- Do not reimplement someone else's filters in Rust as part of A — that would measure the
  reimplementation. A calls their code. B is your own engine and owes them only citation.
- Do not frame any of this as "these systems are bad". They published their filters in full and did
  not publish the counts, which is the field norm, not anyone's error. The contribution is the
  counts, and then the engine that emits them by default.

---

## 8. Python or Rust? — answered with measured numbers, not taste

**Short answer: Python for Project A, by necessity. Fix the algorithm before touching the
language. Rust behind a PyO3 boundary for the two largest cells and for Project B.**

### 8.1 Project A cannot be Rust, and this is not a performance argument

The census measures what *their* filters keep and discard. The population under measurement is
produced by their Python generators and judged by their Python acceptance rules. Reimplementing
either in Rust changes the thing being measured — the result would describe a reimplementation,
not TxGraffiti. **Project A calls the shipped package. That is settled.**

The freedom is in the *analysis kernel*: the code that takes candidates and computes
partitions, marginal contributions, covers and per-object attribution.

### 8.2 Where the time actually goes — measured **[meas]**

From this session, on `graph_data` (N = 340 rows):

- generation: **22.9 ms per generator call**, 3.15 candidates per call (144 calls → 453 candidates
  in 3.3 s). Generators already bottom out in scipy / LP solvers, i.e. C;
- `filter_with_dalmatian`: 384 candidates in 20.3 s. Naive pairwise is 384 × 383 = 147,072
  comparisons, so **≈138 µs per comparison at N = 340**.

Projecting the full six-domain, all-targets, three-generator sweep from those two rates
(an estimate, with the arithmetic shown, not a measurement):

| table | rows | gen calls | generation | candidates | Dalmatian |
|---|---:|---:|---:|---:|---:|
| `graph_data` | 340 | 21,528 | 8.2 min | 67,813 | **7.3 h** |
| `integer_data` | 149 | 1,512 | 0.6 min | 4,763 | 0.0 h |
| `polytope_data` | 496 | 2,340 | 0.9 min | 7,371 | 0.2 h |
| `qubits_data` | 216 | 11,400 | 4.4 min | 35,910 | 1.6 h |
| `calabi_yau_data` | 175 | 117,990 | 45.0 min | 371,668 | **59.3 h** |
| `nba_game_data` | 65,698 | 8,910 | 3.4 min | 28,066 | **106.1 h** |
| **total** | | | ~1 h | ~516,000 | **≈176 CPU-hours (7.3 days)** |

Generation is about an hour. **Filtering is 175 of the 176 hours.** So the whole question is the
filter kernel, and the design should say so in one line rather than discussing language in the
abstract.

### 8.3 The fix is algorithmic and it dwarfs any language choice

`processing/fajtlowicz.py` compares each candidate against every other and **re-evaluates the
conjecture against the pandas DataFrame inside every comparison**. The cost is therefore
O(M² · N) with a pandas constant factor. The paper's own complexity note says O(M · N) with cached
sharp sets and hashing; the released code does not cache.

The fix, in order:

1. **Evaluate once.** Build a dense M × N matrix of each candidate's left- and right-hand values,
   once. For the worst cell — Calabi–Yau, M ≈ 8,080 per target, N = 175 — that is about 11 MB of
   float64. Every comparison afterwards is arithmetic on a resident array instead of a DataFrame
   round-trip.
2. **Sharp sets as bitmasks.** `Sharp(c)` is an N-bit mask. Coverage becomes `sharp & ~covered`,
   alias detection becomes a hash-table lookup on the mask. That is the paper's own O(M · N).
3. **Only then** consider a language port.

Steps 1–2 are worth roughly three to five orders of magnitude. A Python→Rust port is worth perhaps
one to one and a half. **Porting first would be optimising the wrong layer, and would also make the
equivalence check harder to trust.**

Note one subtlety the design must get right: `dalmatian_accept` as shipped is *not* purely a
new-sharp-object test — its docstring says it retains conjectures "strictly tighter than all other
matching conjectures on at least one row", a pairwise comparison over values. Bitmask coverage
captures the alias and representative logic; the strict-tightness part remains genuinely pairwise
and is what stays O(M²). After step 1 that residual is dense-matrix arithmetic, which is exactly
where numpy is already near-C — and exactly where Rust with `rayon` starts to pay.

### 8.4 Where Rust is genuinely right

- **The two heavy cells.** After the fix, Calabi–Yau (46 targets, M ≈ 8,080 each) and NBA
  (N = 65,698, so each comparison touches 193× more rows than the measured baseline) are the only
  places where a numpy kernel plausibly still takes tens of minutes. A Rust kernel with packed
  bitsets, blocked value matrices and `rayon` is the right tool there.
- **Permuted-order marginal contributions.** §6.1 requires the cascade measured in several stage
  orders, because a per-stage kill rate depends on order. That multiplies the filter work by the
  number of orderings and is embarrassingly parallel.
- **Greedy plus ILP cover** over half a million candidates — use OR-Tools or HiGHS, which are C++
  already; do not write this in either language by hand.
- **Project B on large tables.** If the extension points TxGraffiti at magma signatures
  (4,694- or 62,576-bit vectors, per `parsimagma`), Rust is obviously right and the Rust tooling
  already exists in that repo.

### 8.5 The recommended shape, and the gate

Python + numpy analysis kernel first. Measure. Port only the hot kernel to Rust behind **PyO3**,
keeping the Python implementation in the repository as the reference.

**Gate, non-negotiable:** the Rust kernel must produce **bit-identical accept/alias/discard sets**
to the Python reference on every one of the six tables before any timing claim is published. A
faster kernel that changes the partition is not an optimisation, it is a different experiment. Keep
the equivalence test in CI.

---

## 9. Scaling: cores, GPUs, nodes — and the cliff that is not where you expect

The instinct is to reach for a distributed framework. Resist it, and here is the evidence rather
than the opinion.

### 9.1 The datum that settles multi-node

`parsimagma`'s own README: `pm bruteforce 4` sweeps **4,294,967,296 tables, 178,981,952 after
isomorphism filtering, in about 83 minutes on 11 cores at 75 MB resident**. Billions of items,
under an hour and a half, on one laptop, in a rounding error of RAM.

A signature corpus of 10⁶ objects × 62,576 boolean columns is 7.8 GB bit-packed; 10⁷ objects is
62 GB. Both fit one machine. **The working set is not the constraint and multi-node is not the
answer to a question anyone here is asking.** If a cell ever does exceed one machine, the workload
is embarrassingly parallel over (domain, target) cells, so the right tool is a job array or a work
queue — not Spark, Ray or Dask, whose per-task overhead would dominate millisecond-scale tasks and
whose data model assumes the data does not fit, which here it does.

### 9.2 The ladder, in the order the wins actually arrive

1. **Algorithm.** Evaluate once into a matrix; sharp sets as bitmasks; hash for alias detection;
   sort by touch count and prune with the coverage mask. §8.3: three to five orders of magnitude.
   Everything below is worth one to one and a half.
2. **Memory layout.** Bit-packed columns, cache-blocked pairwise loops, column-major where the
   access is columnar. For this workload, layout is probably worth more than every remaining item
   on this list combined.
3. **SIMD.** `u64` word loops with `count_ones()` compile to POPCNT; AVX-512 VPOPCNTDQ where
   available. This is the big constant-factor win for the boolean block, and it is bigger here than
   a GPU would be.
4. **Multi-core, one box.** `rayon` over candidates or over (target, domain) cells. Mandatory,
   nearly free, 8–16×. Do this and stop.
5. **GPU — only for one phase, and only after profiling.** See §9.3.
6. **Multi-node — almost certainly never.** See §9.1.

### 9.3 Why GPU is the wrong tool for the boolean block and the right one for the numeric block

Bitset AND / ANDNOT / popcount is **memory-bandwidth-bound**, not compute-bound: a few operations
per word loaded. GPUs win on arithmetic intensity, and here PCIe transfer often costs more than the
CPU would have spent doing the work. Modern CPU POPCNT is already excellent.

The **dense numeric pairwise phase** is the opposite case. After the §8.3 fix, Dalmatian's residual
strict-tightness test is arithmetic over an M × N value matrix compared pairwise — a GEMM-shaped
access pattern with real arithmetic intensity. *That* is where a GPU could pay, and only there.

So the honest guidance: profile the two blocks separately, and do not let "we have a GPU" decide
which algorithm gets written.

### 9.4 Crates and libraries worth evaluating

**Verify current status and versions before relying on any of these — this list was assembled
without checking release state in this session.**

- `rayon` — data parallelism, single node. The default answer to item 4.
- `roaring` (roaring-rs) — compressed bitmaps with fast AND / OR / ANDNOT / cardinality.
  Strongly relevant: signature columns here are often extremely skewed (of 62,576 order-5 laws,
  19,392 are trivial-only, and the constant magma satisfies 22,604). Compare against plain
  bit-packing on real data before committing; Roaring wins on sparse, loses on dense.
- `bitvec` / `fixedbitset` — plain bit-packed vectors, when the columns are dense.
- `ndarray` — the numeric block.
- `polars` / `arrow` / `datafusion` — if the table layer wants to be a real columnar engine rather
  than a hand-rolled struct. Worth considering for the numeric block and the I/O, not for the
  boolean block.
- GPU (`cudarc`, `wgpu`, and successors) — evaluate only if §9.3's numeric phase is shown to
  dominate. Treat as a late optimisation, never a foundation.
- **Not** Spark, Ray, or Dask. Task granularity too small, data fits in RAM, and the operational
  cost buys nothing.

### 9.5 The design rule that makes all of this safe to postpone

**Make the kernel a trait with a plain scalar reference implementation, and keep that reference in
the repository forever.** Every backend — SIMD, rayon, Roaring, GPU — must produce
**bit-identical partitions** to the reference on every table, checked in CI. A faster kernel that
changes the partition is not an optimisation, it is a different experiment.

With that gate in place, none of the questions in this section have to be answered now. Write the
scalar version, measure, and let the profile choose.

### 9.6 The trigger for revisiting

Name it in the design so the decision is not made by enthusiasm:

- revisit **GPU** when the numeric pairwise phase exceeds ~50% of wall clock after items 1–4;
- revisit **multi-node** when the working set exceeds single-machine RAM, or when one (domain,
  target) cell takes more than a day on all cores after items 1–4.

Until one of those fires, one box with `rayon` is the architecture.

---

## 10. The name

**`ragusa`.** Decided, not a suggestion.

**Where it comes from, for the README.** The Dalmatian heuristic is named after the dog. The dog is
named after Dalmatia. The great city-state of Dalmatia was the **Republic of Ragusa** — *Ragusa*
being the Italian and Latin name for what is now Dubrovnik — and its motto was a single word,
**LIBERTAS**, carved over the Lovrijenac fortress gate, from *"Non bene pro toto libertate venditur
auro"*: liberty is not well sold for all the gold.

So the line the project turns on: **the Dalmatian locks conjectures up; Ragusa sets them free.**
A republic, not a cage. That is the whole joke, it is historically real rather than bolted on, and
the Italian name is the one to use.

**Not `something-lean`.** That suffix in this author's repositories marks projects whose *content*
is a Lean formalization. This is Rust and Python with Lean as an optional export at the far
end, and it does not belong in the `lean4/` folder. If the formalization axis (§6.2 item 6) later
grows into a real library, spin *that* out as its own `-lean` repo, which preserves the convention
instead of diluting it.

**Repository conventions.** Same name locally and on GitHub, directory matching the repo. A search
at naming time found a GitHub user `ragusaa` and the Sicilian city, but no software project holding
the name.

---

# Appendix A — Lean 4 and Mathlib on this machine (shared-environment rules)

**When this applies.** Only to the formalization axis, §6.2 item 6 — the moment anything in this
project invokes `lake`, Lean, or Mathlib, whether through `txgraffiti.export_utils.lean4` or
AutoGraphForge's `lean_project/`. The census in Project A touches none of it. Do not front-load
these rules into a project that is mostly Python; do not skip them for one minute once Lean enters.

**Canonical source of truth:** `PROMPT_SHARED_LEAN_ENV.md`, in the Lean workspace root on Carlo's Mac. The text below was read from that file in this session and is reproduced verbatim.
If the two ever differ, **the file wins** — re-read it before acting.

---

# Lean 4 and Mathlib on this machine: reuse, never re-download

You are working on Carlo's Mac, where several Lean projects already share built copies of
Mathlib. One built copy is about 7 GB. Downloading or building another is wasted disk space
and hours of time, and one careless command can break every project that shares a copy.
Read this before any `lake` command.

## 1. The toolchain

Lean is installed through elan at `~/.elan`. Many toolchains are already there:

```bash
elan toolchain list
```

A project's `lean-toolchain` file picks the version. Use one that is already installed.
**Do not** run `elan toolchain install`, `elan update` or `elan self update`, and do not change
a `lean-toolchain` to a version that is not on the list. If you really need a new toolchain,
stop and ask Carlo.

## 2. The shared Mathlib copies

Each canonical copy is a real `.lake/packages` directory holding Mathlib, its dependencies and
the compiled `.olean` files. Other projects point at it with a symlink.

| Mathlib rev | Toolchain | Note | Projects already sharing it |
|---|---|---|---|
| `0df444a360eaa60ab8c11dca51a86af692955474` | `leanprover/lean4:v4.33.1` | newest; the default choice | 4 |
| `81a5d257c8e410db227a6665ed08f64fea08e997` (tag `v4.32.0`) | `leanprover/lean4:v4.32.0` | | 1 |
| `c5ea00351c28e24afc9f0f84379aa41082b1188f` | `leanprover/lean4:v4.30.0` | | none |
| `777aaa61dcd2a1258d2b4962dbe983ede4d23b2e` | `leanprover/lean4:v4.29.0-rc3` | | none |

The canonical `.lake/packages` paths are local to Carlo's machine and are in
`PROMPT_SHARED_LEAN_ENV.md`, not here.

Check this table against the disk before you rely on it. The `rev` for `mathlib` in the
canonical project's `lake-manifest.json` is the truth.

## 3. Setting up a new or existing project

Pick a row. **The Mathlib rev and the toolchain must both match exactly.** If no row matches
what the project needs, stop and ask; do not fetch a new Mathlib.

1. **Toolchain.** Copy `lean-toolchain` from the canonical project.
2. **Requirement.** In `lakefile.lean` or `lakefile.toml`, require Mathlib at the same rev as
   the canonical project. A full commit hash is safest; the `v4.32.0` tag also resolves to the
   right commit for that row.
3. **Manifest.** Copy `lake-manifest.json` from the canonical project, then change only the
   top-level `"name"` field to your package's name. Every entry under `packages` must stay
   byte-identical: same `rev`, `inputRev` and `packagesDir`. Confirm with:
   ```bash
   python3 -c "import json,sys; a,b=[{p['name']:(p['rev'],p.get('inputRev')) for p in json.load(open(f))['packages']} for f in sys.argv[1:]]; print('identical' if a==b else 'DIFFERENT')" <canonical>/lake-manifest.json ./lake-manifest.json
   ```
4. **Symlink.** Point your `.lake/packages` at the canonical one:
   ```bash
   mkdir -p .lake
   ln -s <canonical project>/.lake/packages .lake/packages
   ```
   If `.lake/packages` already exists as a real directory, do not delete it. Tell Carlo how
   big it is (`du -sh .lake/packages`) and let him decide.
5. **Build only your own code:**
   ```bash
   lake build <YourLib>
   ```
   Mathlib's oleans come from the shared copy, so this should compile only your files. If the
   output shows thousands of Mathlib jobs *compiling* rather than being replayed, stop at once.
   The manifest or the toolchain does not match.

Your project's own `.lake/build` stays a real, per-project directory. Only `packages` is shared.

## 4. Commands that are forbidden in a project whose packages are shared

These commands write into the shared directory, so they damage every project linked to it:

- `lake update`, which rewrites the manifest and re-checks-out dependencies;
- `lake exe cache get` or `lake exe cache get!`, which overwrite the shared oleans;
- `lake clean`, and any `rm -rf .lake` that follows the symlink;
- `git checkout`, `git pull` or `git stash` inside `.lake/packages/*`. Some copies already
  report "local changes"; leave them alone;
- editing any file under `.lake/packages`;
- deleting or moving a canonical project, or its `.lake`.

When `lake` offers to update a dependency or reports a manifest mismatch, **do not accept**.
Fix your own manifest (step 3) instead.

Reading is always fine: `grep -rn "theorem foo" .lake/packages/mathlib/Mathlib/`, `#check`,
and `lake env lean File.lean`.

## 5. Concurrency

Several agents may build at the same time. Two `lake build` runs on the *same* project fight
over its lock; wait for one to finish rather than killing it. Builds in different projects
that share packages are fine, because they only read the shared oleans.

## 6. Where to put things

- Lean projects live under a single Lean root on Carlo's machine, with the shared workspaces
  in a subdirectory of it; `PROMPT_SHARED_LEAN_ENV.md` gives the paths.
- **Nothing bulky goes in `$HOME`.** No new workspaces, clones or caches there.
- Scratch files go in your session's scratch directory or in `/tmp`, never inside a canonical
  project.

## 7. Before you finish

Report three things:

- which canonical copy you linked to;
- the output of `ls -la .lake`, showing the symlink;
- the disk use of your project's own `.lake/build` (`du -sh .lake/build`).

