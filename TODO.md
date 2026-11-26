# Open items

What is still open in this repository. Generated files are rendered from `results/` by `make docs`;
edit the `.tmpl` next to them, not the `.md`.

## Outreach

- [x] **Pastorek** (AutoGraphForge) and **Davila** (TxGraffiti) were written to on 2026-10-04 about the points
      that concern their code. Where they have not answered, those points are stated as our reading.
- [ ] If either answers, record it in `CONTROLS.md` and update the note if a number changes.
- [x] **KnotInfo maintainers** answered on 2026-10-02: no copyright; cite KnotInfo; do not post the full
      database. Recorded in `CONTROLS.md`; the citation is `knotinfo` in `CITATIONS.bib`. Derived aggregates are
      published; the export itself stays out of the repository.

## Review

- [ ] `calibration/note/calibration.md`: re-read the "alias" definition (ours, not txgraffiti's)
      and the Sophie paragraph.
- [ ] `CITATIONS.bib`: every entry marked VERIFY needs its title and authors checked against
      the source; Fajtlowicz, DeLaViña and Lenat still have no entry.
- [ ] Write-up (ROADMAP, "later"): a short note, or a related-work table in B's paper.
- [ ] `engine/data/knots/jablonowski2026.json` was transcribed by an agent from page images of
      arXiv:2605.22652 v4. Spot-check the 47 relations against Figure 1, and the four "likely" column
      mappings (clasp numbers, doubly slice genus). G-B9 found no bound refuted by KnotInfo, which
      catches wrong directions but not missing relations.

## Decisions

- [x] 2026-09-14: B's target is a frontier domain that is not ETP, chosen in Sprint 6.
      Magmas (4694 laws) are a calibration against a complete oracle; graphs are the control.
      See `ROADMAP.md`.
- [x] 2026-09-14: parsimagma is used as data (signature corpus, ETP matrix), not as a code dependency.
- [x] Sprint 6: KnotInfo (primary, with Jabłonowski's catalogue), toric canonical Fano 3-folds
      (scale test), matroids ≤ 9 (reserve).
- [x] 2026-09-15: sharp coverage overfits on held-out knots (CONTROLS.md, phase E log); replaced by
      `greedy_touch` behind `--min-tight sqrt` (ROADMAP, Sprints 9–10).
