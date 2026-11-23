# ragusa

A conjecturing engine that reports its own floor. Every run emits each filter's
partition (representative / alias / discarded), per-object attribution, a chance
baseline and a pinning block.

## What ragusa is

1. **A table:** a finite list of objects, each with numeric invariants and true/false
   properties, in one table type that holds a few dozen numeric columns or thousands of
   boolean ones.
2. **A generator:** every statement of a fixed, small language over that table (bounds
   `target <= c·feature + k` under a hypothesis; implications `P ⇒ Q`), exhaustively.
3. **Filters and the floor report:** what survives, what was discarded and why, which single
   objects decide each verdict, how the rules compare with random acceptance, and what a
   known-results oracle already knows.

A surviving statement is a conjecture (true on every listed object), not a theorem; ragusa
proves nothing. Its output is conjectures together with the evidence floor behind them.

**The core is `engine/`.** Everything else is its validation suite: `calibration/` (Project A,
a census of what existing tools discard), the TxGraffiti graph tables (the control) and
parsimagma's magma corpus against the ETP implication graph (a complete answer key). The gates in
`CONTROLS.md` load them, so they stay with the core.

- `calibration/` — Project A. Measures what the shipped TxGraffiti 0.4.1 filters
  and AutoGraphForge's novelty filter throw away on `graph_data`, calling their
  code as-is.
- `engine/` — Project B. A Rust engine over one table type: a dense numeric block
  and a bit-packed boolean block. It runs on TxGraffiti snapshot tables and
  magma signature corpora without special-casing either.

The design lives in `PROMPT.md`, the current plan in `ROADMAP.md`. Gates and their outcomes
are in `CONTROLS.md`; open items for the author in `TODO.md`.

## Running

```bash
make test          # schema, template lint, unit tests (incl. exhaustive kernel control)
make reproduce-a   # Project A from scratch: gates, census, rendered note and email drafts
make docs          # re-render documents from results/rows only
```

Project A needs `uv` and a clone of AutoGraphForge at `../_ext/AutoGraphForge`, pinned
in `calibration/vendor.lock`. The calibration note is `calibration/note/calibration.md`.

## Licence

Apache-2.0, in `LICENSE`. The magma oracle data under `engine/data/parsimagma/`
comes from the Equational Theories Project and is Apache-2.0 too; the attribution
it requires is in `NOTICE`, the provenance and hashes in
`engine/data/parsimagma/PROVENANCE.txt`. Other people's work that ragusa *measures*
rather than ships (TxGraffiti, AutoGraphForge) is cloned separately and credited in
`ACKNOWLEDGMENTS.md`.

KnotInfo is read from https://knotinfo.org and never bundled. Its maintainers asked that the
database not be reposted, because copies go out of date; download it from the source. Results
record the sha256 of the file they were computed on, and `CITATIONS.bib` has the citation the
maintainers ask for.

## The name

The Dalmatian heuristic is named after the dog, and the dog after Dalmatia. The
great city-state of Dalmatia was the Republic of Ragusa (today's Dubrovnik). Its
motto, LIBERTAS, is carved over the Lovrijenac fortress gate.

The Dalmatian locks conjectures up; Ragusa sets them free.
