# TODO for Carlo

Things only you can do. Generated files are rendered from `results/` by `make docs`;
edit the `.tmpl` next to them, not the `.md`.

## Outreach

- [x] **Pastorek** (AutoGraphForge) and **Davila** (TxGraffiti) were written to on 2026-10-04 about the points
      that concern their code. Where they have not answered, those points are stated as our reading.
- [ ] If either answers, record it in `CONTROLS.md` and update the note if a number changes.
- [x] **KnotInfo maintainers** answered on 2026-10-02: no copyright; cite KnotInfo; do not post the full
      database. Recorded in `CONTROLS.md`; the citation is `knotinfo` in `CITATIONS.bib`. Derived aggregates are
      published; the export itself stays out of the repository.

## Review

- [ ] Read `calibration/note/calibration.md`. Check the "alias" definition (ours, not
      txgraffiti's) and the Sophie paragraph read the way you want.
- [ ] `CITATIONS.bib`: every entry marked VERIFY needs its title and authors checked against
      the source; Fajtlowicz, DeLaViña and Lenat still have no entry.
- [ ] Decide later (Sprint 7, not now): short note, or related-work table in B's paper.
- [ ] `engine/data/knots/jablonowski2026.json` was transcribed by an agent from page images of
      arXiv:2605.22652 v4. Spot-check the 47 relations against Figure 1, and the four "likely" column
      mappings (clasp numbers, doubly slice genus). G-B9 found no bound refuted by KnotInfo, which
      catches wrong directions but not missing relations.

## Decisions pending

- [ ] **Sharp coverage overfits (CONTROLS.md, phase E log).** On KnotInfo ≤ 12 crossings with a
      seeded 20% hold-out, 14 of 38 survivors fail on held-out knots, against 274 of 2751 valid
      candidates. Options: make a held-out check part of every run's report; add a rule that keeps
      bounds with several witnesses rather than one; or accept it and treat survivors strictly as
      "to be refuted". One table, one target, one seed so far.

## Decisions

- [x] 2026-09-14: B's target is a frontier domain that is not ETP, chosen in Sprint 6.
      Magmas (4694 laws) are a calibration against a complete oracle; graphs are the control.
      See `ROADMAP.md`.
- [x] 2026-09-14: parsimagma is used as data (signature corpus, ETP matrix), not as a code dependency.
- [x] Sprint 6: KnotInfo (primary, with Jabłonowski's catalogue), toric canonical Fano 3-folds
      (scale test), matroids ≤ 9 (reserve).
