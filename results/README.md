# results

Every published number is recomputed from the files in `rows/`; nothing is typed by hand.

- `*.jsonl` — one row per (system, table, target, generator, filter, stage), validated against
  `schema.json` by `calibration/src/ragusa_a/pin.py` (partition parts must sum to `n_in`).
- `census_*.metrics.json` — structured measurements that are not partition counts: object floor,
  chance baselines, gate outcomes, pinning.
- `census_*.decisions.csv` — one line per candidate: its class under each rule, alias target,
  pipeline survival, AutoGraphForge verdicts.
- `txg_*.survivors.txt` — sorted survivor names of a staged run, for set comparison.
- `gb4_*.json` — gate G-B4: ragusa's filter partitions against Project A's, joined by bound text.

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

Rendered documents: `make docs`. Full regeneration: `make reproduce-a`.
