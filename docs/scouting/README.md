# Sprint 6: frontier scouting

Goal (ROADMAP.md): pick the domain where ragusa can produce conjectures mathematicians care
about that are not already settled. Graphs (the control) and ETP magmas (a complete answer key)
are excluded as targets.

## Criteria

A domain qualifies when it has all four:

1. a large, already-enumerated object database, downloadable and licensed for reuse;
2. many numeric invariants and many boolean properties per object, stored or cheap to compute;
3. a live literature of inequalities or implications between those invariants;
4. no complete oracle (so conjectures can be new), ideally a partial one to measure novelty against.

## Method

Three research agents surveyed one area each (finite algebra; number theory, topology and
geometry; combinatorics), reading web pages and papers only, with no data downloaded and a URL
required for every claim. Their reports are the raw material, not the result: before a
candidate enters the shortlist, its load-bearing facts (object count, completeness, licence) are
re-checked against the source page here.

Every page marks each fact as one of:

- **checked**: re-read on the source page during scouting, with the URL;
- **reported**: from the agent's report with its URL, not yet re-read;
- **unverified**: nobody could confirm it.

Literature citations are all **reported** until someone reads the paper.

## Status

| area | page | status |
|---|---|---|
| finite algebra | [algebra.md](algebra.md) | report received, key facts checked |
| number theory, topology, geometry | [numbertheory-geometry.md](numbertheory-geometry.md) | report received, key facts checked |
| combinatorics | [combinatorics.md](combinatorics.md) | report received, key facts checked |
| all areas | [shortlist.md](shortlist.md) | finalists, engine needs, recommendation, decision |

## Load tests

| dataset | page | result |
|---|---|---|
| KnotInfo | [loadtest-knotinfo.md](loadtest-knotinfo.md) | loads; interval-valued invariants; dense subset up to 12 crossings |
| toric canonical Fano 3-folds | [loadtest-fano.md](loadtest-fano.md) | loads; 674,688 rows, no NULLs; rational degree; one README off-by-one |
| matroids ≤ 9 elements | [loadtest-matroids.md](loadtest-matroids.md) | loads; format self-checks; counts match OEIS |

Downloads live in the git-ignored `external/`.
