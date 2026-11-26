# results

Every number in the rendered documents (`make docs`) is recomputed from the files in `rows/`; none is
typed by hand there. `CONTROLS.md`, `ROADMAP.md` and `docs/` are hand-written logs and quote numbers as
they stood on the dated entry.

- `*.jsonl` — one row per (system, table, target, generator, filter, stage), validated against
  `schema.json` by `calibration/src/ragusa_a/pin.py` (partition parts must sum to `n_in`).
- `census_*.metrics.json` — structured measurements that are not partition counts: object floor,
  chance baselines, gate outcomes, pinning.
- `census_*.decisions.csv` — one line per candidate: its class under each rule, alias target,
  pipeline survival, AutoGraphForge verdicts.
- `txg_*.survivors.txt` — sorted survivor names of a staged run, for set comparison.
- `gb4_*.json` — gate G-B4: ragusa's filter partitions against Project A's, joined by bound text.
- `heldout_sweep_*.json`, `rule_study*.json`, `harvest_fano3c.*` — Sprints 8–11. No make target
  produces them; each file records its seeds and engine arguments, and the commands are, from
  `calibration/`:

      uv run --locked python -m ragusa_a.heldout_sweep graph_data --out ../results/rows/heldout_sweep_graph_data.json
      uv run --locked python -m ragusa_a.rule_study graph_data --seeds 0-4 --out ../results/rows/rule_study_graph_data_s0-4.json
      uv run --locked python -m ragusa_a.harvest fano3c --targets degree,volume \
          --engine-args "--min-tight sqrt --hypotheses TRUE,all" --out ../results/rows/harvest_fano3c.json

  The variants differ only in the recorded `seeds` and `engine_args`. `harvest_fano3c` needs
  `external/fano3c/toricf3c.db` (README.md, External inputs).

`floor/<table>_<target>/` holds one ragusa run, written by `ragusa floor` (Project B), which cannot
run without emitting all three:

- `floor.json` — the floor report: pinning block, run spec, per-stage partitions, per-object flips
  under single-object removal, and seeded chance baselines;
- `floor.md` — the same report rendered for reading;
- `candidates.csv` — one line per candidate with its class at every stage it reached.

`floor/<table>_implications/` holds an implication run on a boolean-only table, written by
`ragusa implications`: `floor.json` and `floor.md` with the refutation stage, per-object
attribution, the ETP novelty split, the soundness check and hard-core coverage. There is no
per-candidate CSV: the candidates are all 22 million ordered law pairs.

Rendered documents: `make docs`. Project A's rows and the census: `make reproduce-a`. The engine
fixtures, `gb4_*` and `floor/`: `make reproduce-b`.

**Revisions.** The `engine_git_rev` and `git_rev` values recorded before 2026-11-23 name commits of the
pre-release history, which is not public; that history was published as the single commit `13c95ef`.
The results were not regenerated.
