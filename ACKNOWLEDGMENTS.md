# Acknowledgments

Drafted before any result, so these obligations are structural.

**Randy Davila.** TxGraffiti: the paper (arXiv:2409.19379 v2, Annals of
Mathematics and Artificial Intelligence, DOI 10.1007/s10472-026-10005-5), the
website (https://txgraffiti.streamlit.app/) and the `txgraffiti` package, whose
shipped tables, generators and acceptance rules Project A measures directly.
His terms of usage ask for an acknowledgment and a citation of the website.
Both are given here, and he was written to with the results on 2026-10-04.

**Ján Pastorek.** AutoGraphForge (arXiv:2609.03478,
https://github.com/JanPastorek/AutoGraphForge), whose known-results filter is,
as far as we found, the first novelty oracle in this line of work. Nothing
about its numbers was published before he was written to, on 2026-10-04.

**The lineage.** Siemion Fajtlowicz (Graffiti and the Dalmatian heuristic);
Ermelinda DeLaViña (Graffiti.pc, the Sophie heuristic); Craig Larson and Nico
Van Cleemput (the formal treatment of the Dalmatian heuristic and the
`Conjecturing` program); Simon Colton (HR); Douglas Lenat (AM, EURISKO); and
Graeme Ritchie and Frank Hanna, whose 1984 critique is the reason the guards in
this project exist. Cited in `CITATIONS.bib` as `fajtlowicz1988graffiti`,
`delavina2005graffitipc`, `larson2016dalmatian`, `colton2001hr`, `lenat1976am` and
`ritchie1984am`.

**Charles Livingston and Allison Moore.** KnotInfo (https://knotinfo.org), the table of knot
invariants behind the knot runs and the candidate dossier. KnotInfo carries no copyright; they ask
that it be cited (`knotinfo` in `CITATIONS.bib`) and that the database itself not be reposted, since
copies go stale. It is read from their site and never bundled here.

**Data.** The Equational Theories Project
(https://github.com/teorth/equational_theories, Apache-2.0), whose implication
data serves as the magma novelty oracle. The files here come from upstream
commit `e5a88a1479011ece4aad8e3c2e7e5c0ebc0a5b2a` by way of parsimagma
(https://github.com/carlok/parsimagma, the author's own project, Apache-2.0), which
repacked them; the solver run behind them is Janota, arXiv:2508.15856. The
formats, hashes and copy provenance are in
`engine/data/parsimagma/PROVENANCE.txt`, the attribution required by Apache-2.0
section 4 is in `NOTICE`, and the citations in `CITATIONS.bib` are `etp2025` (the form in the project's
`CITATION.cff`), `bolan2025etp` (its paper, arXiv:2512.07087) and `janota2025vampire`.

**Michał Jabłonowski.** *Integer Knot Invariants: Inequalities, Computations, and Open
Problems* (arXiv:2605.22652), whose catalogue of relations between knot invariants is the
knot oracle (`engine/data/knots/jablonowski2026.json`, `jablonowski2026knots`). The file was
transcribed from the figures of version 4 by an AI agent and has not yet been checked by hand
against the paper (`TODO.md`); errors in it are ours.

**Alexander Kasprzyk.** The classification of toric canonical Fano 3-folds (Zenodo
5866330, CC0) and *Canonical Toric Fano Threefolds*, Canad. J. Math. 62 (2010), behind the
Fano runs; cited as the dataset asks, `kasprzyk2010fanodata` and `kasprzyk2010fano`. The
database is downloaded from Zenodo and not bundled.
