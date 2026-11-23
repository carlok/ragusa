# ragusa floor report: `graph_data`, target `independence_number`

- table content sha256 `04f7b04fc0b76392167e5eb9c65213ccdbf88e92163c748a5f61aed828b8c5b1`, 340 objects, 24 numeric and 13 boolean columns
- engine 0.1.0 (git a450552a44c48274fcba3fd5c7a087ce5cd7ede0), seed 0, 1000 chance draws per stage, slack tolerance 1e-9
- 2544 candidates (`convex_hull` 1348, `linear_programming` 598, `ratios` 598); 10 convex-hull calls needed joggling

## Stages

| stage | kind | input | representative | alias | discarded | fragile decisions | objects with a flip |
|---|---|---:|---:|---:|---:|---:|---:|
| `validity` | validity | 2544 | 2510 | 0 | 34 | 15 | 2 |
| `table_duplicates` | redundancy | 2510 | 1526 | 984 | 0 | 11 | 2 |
| `sharp_coverage` | redundancy | 1526 | 19 | 0 | 1507 | 11 | 9 |
| `generality` | redundancy | 19 | 19 | 0 | 0 | 0 | 0 |

Survivors: 19

## Chance baseline

Random acceptance of the same number of candidates from the same stage input.

| stage | metric | rule | random mean [2.5%, 97.5%] | P(random ≥ rule) |
|---|---|---:|---|---:|
| `validity` | tight_objects | 256 | 256.0 [256, 256] | 0.999 |
| `validity` | false_kept | 0 | 33.5 [32, 34] | 1.000 |
| `validity` | hypotheses_kept | 13 | 13.0 [13, 13] | 1.000 |
| `validity` | distinct_table_classes | 1526 | 1546.7 [1541, 1552] | 1.000 |
| `table_duplicates` | tight_objects | 256 | 254.6 [250, 256] | 0.424 |
| `table_duplicates` | false_kept | 0 | 0.0 [0, 0] | 1.000 |
| `table_duplicates` | hypotheses_kept | 13 | 13.0 [13, 13] | 1.000 |
| `table_duplicates` | distinct_table_classes | 1526 | 1078.4 [1057, 1099] | 0.000 |
| `sharp_coverage` | tight_objects | 81 | 60.4 [23, 159] | 0.198 |
| `sharp_coverage` | false_kept | 0 | 0.0 [0, 0] | 1.000 |
| `sharp_coverage` | hypotheses_kept | 9 | 10.0 [8, 12] | 0.906 |
| `sharp_coverage` | distinct_table_classes | 19 | 19.0 [19, 19] | 1.000 |
| `generality` | tight_objects | 81 | 81.0 [81, 81] | 1.000 |
| `generality` | false_kept | 0 | 0.0 [0, 0] | 1.000 |
| `generality` | hypotheses_kept | 9 | 9.0 [9, 9] | 1.000 |
| `generality` | distinct_table_classes | 19 | 19.0 [19, 19] | 1.000 |

## Objects that decide the most

Decisions that flip when this one object is removed, per stage.

| object | total | validity | table_duplicates | sharp_coverage | generality |
|---|---:|---:|---:|---:|---:|
| 286 | 14 | 14 | 0 | 0 | 0 |
| 4 | 8 | 0 | 8 | 0 | 0 |
| 44 | 3 | 0 | 3 | 0 | 0 |
| 135 | 2 | 0 | 0 | 2 | 0 |
| 180 | 2 | 0 | 0 | 2 | 0 |
| 5 | 1 | 0 | 0 | 1 | 0 |
| 77 | 1 | 1 | 0 | 0 | 0 |
| 98 | 1 | 0 | 0 | 1 | 0 |
| 161 | 1 | 0 | 0 | 1 | 0 |
| 187 | 1 | 0 | 0 | 1 | 0 |
