# ragusa implication floor report: `magma_corpus_etp4694`

- table content sha256 `dd1ae154e08ec093f2678ddf736f7c9a0c604b972d4b499f795dc92c515bb240`, 2786 objects, 4694 laws (906 distinct columns on these objects)
- engine 0.1.0 (git a450552a44c48274fcba3fd5c7a087ce5cd7ede0); oracle: ETP implication graph

## Stages

| stage | kind | input | kept | removed |
|---|---|---:|---:|---:|
| `refutation_by_table` | validity | 22028942 | 8454100 emitted | 13574842 refuted |
| `etp_oracle` | novelty | 8454100 | 280515 known false | 8173585 known true |

Soundness: 0 pairs refuted by the table are true in the oracle (pass).

## Chance baseline

Known-true rate: 0.3710 over all 22028942 ordered pairs, 0.9668 over the 8454100 emitted.

Hard core: 416 of 1062 pairs refuted by the table.

## Objects that decide the most

2841746 pairs are refuted by exactly one object; removing that object would emit them.

| object | pairs it alone refutes |
|---|---:|
| linear/Z_m m=2 a=0 b=0 finite:2 | 1731358 |
| linear/Z_m m=2 a=0 b=1 finite:2 | 262672 |
| linear/Z_m m=2 a=1 b=0 finite:2 | 262672 |
| linear/Z_m m=3 a=0 b=2 finite:3 | 74968 |
| linear/Z_m m=3 a=2 b=0 finite:3 | 74968 |
| linear/Z_m m=7 a=0 b=2 finite:7 | 65904 |
| linear/Z_m m=7 a=2 b=0 finite:7 | 65904 |
| linear/Z_m m=4 a=0 b=2 finite:4 | 53406 |
| linear/Z_m m=4 a=2 b=0 finite:4 | 53406 |
| linear/Z_m m=2 a=1 b=1 finite:2 | 48372 |
