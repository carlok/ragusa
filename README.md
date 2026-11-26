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
[parsimagma](https://github.com/carlok/parsimagma)'s magma corpus against the ETP implication graph (a complete answer key). The gates in
`CONTROLS.md` load them, so they stay with the core.

- `calibration/` — Project A. Measures what the shipped TxGraffiti 0.4.1 filters
  and AutoGraphForge's novelty filter throw away on `graph_data`, calling their
  code as-is.
- `engine/` — Project B. A Rust engine over one table type: a dense numeric block
  and a bit-packed boolean block. It runs on TxGraffiti snapshot tables and
  magma signature corpora without special-casing either.

The design lives in `PROMPT.md`, the current plan in `ROADMAP.md`. Gates and their outcomes
are in `CONTROLS.md`; open items in `TODO.md`.

## Requirements

- [rustup](https://rustup.rs/); `engine/rust-toolchain.toml` pins Rust 1.96.0 and rustup installs it.
- [uv](https://docs.astral.sh/uv/); it installs Python 3.13 and the locked environment of `calibration/`.
- cmake, a C/C++ compiler and libclang: the HiGHS and qhull bindings build their C/C++ sources.
- An LP solver for TxGraffiti's `linear_programming` generator: `cbc` or `glpsol` on PATH, or else
  the CBC binary bundled with PuLP, which `calibration/` finds by itself.

## Running

```bash
make test            # schema, template lint, pytest, the engine's tests (no external inputs needed)
make docs            # re-render the documents from results/rows only
make reproduce-a     # Project A from scratch: gates, census, rendered note (needs AutoGraphForge)
make reproduce-b     # engine fixtures, every engine gate, the committed floor reports
make frontier-gates  # gates G-B6 and G-B9 on the KnotInfo and Fano tables (needs both downloads)
```

The Rust tests read TxGraffiti's shipped tables from `calibration/.venv`, which `make test` creates;
`RAGUSA_TXGRAFFITI_DATA` points them elsewhere.

## External inputs

None of these are in the repository; each goes where the table says.

| input | where | used by |
|---|---|---|
| [AutoGraphForge](https://github.com/JanPastorek/AutoGraphForge) at commit `510cf50` (`calibration/vendor.lock`) | `../_ext/AutoGraphForge`, next to this repository | `make reproduce-a` |
| KnotInfo spreadsheet, `knotinfo_data_complete.xls` unzipped from https://knotinfo.org | `external/knotinfo/` | `make frontier-gates`, the knot runs |
| toric canonical Fano 3-folds, Zenodo record [5866330](https://doi.org/10.5281/zenodo.5866330), imported with `sqlite3 toricf3c.db < toricf3c.sql` | `external/fano3c/toricf3c.db` | `make frontier-gates`, the Fano harvest |

KnotInfo's download carries no version. The results record the sha256 of the file they were
computed on (`docs/scouting/loadtest-knotinfo.md`); a later download may differ.

## Where the results are

- `calibration/note/calibration.md`: Project A, what the shipped TxGraffiti filters and
  AutoGraphForge's novelty filter discard on `graph_data`, every number rendered from `results/rows/`.
- `results/floor/`: the engine's floor reports, one directory per run (`floor.md` to read).
- `CONTROLS.md`: every gate, pass or fail, and the dated log of what was measured, including the
  negative results.
- `results/README.md`: what each result file is and which command produces it.

## Licence

Apache-2.0, in `LICENSE`. The magma oracle data under `engine/data/parsimagma/`
comes from the Equational Theories Project and is Apache-2.0 too; the attribution
it requires is in `NOTICE`, the provenance and hashes in
`engine/data/parsimagma/PROVENANCE.txt`. Other people's work that ragusa *measures*
rather than ships is installed from PyPI (TxGraffiti) or cloned next to the repository
(AutoGraphForge), and credited in `ACKNOWLEDGMENTS.md`. To cite ragusa, see `CITATION.cff`.

KnotInfo is read from https://knotinfo.org and never bundled. Its maintainers asked that the
database not be reposted, because copies go out of date; download it from the source. Results
record the sha256 of the file they were computed on, and `CITATIONS.bib` has the citation the
maintainers ask for.

## The name

The Dalmatian heuristic is named after the dog, and the dog after Dalmatia. The
great city-state of Dalmatia was the Republic of Ragusa (today's Dubrovnik). Its
motto, LIBERTAS, is carved over the Lovrijenac fortress gate.

The Dalmatian locks conjectures up; Ragusa sets them free.
