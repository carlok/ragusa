"""Number 4: AutoGraphForge's known-results filter over TxGraffiti candidates (novelty kind).

TxGraffiti renders a conjecture as "(hyp) → (body)"; AutoGraphForge's parser splits
the hypothesis off at "⇒". The only adapter is that arrow substitution, and every
row declares whether it was applied. Credit: Ján Pastorek, AutoGraphForge.
"""

from __future__ import annotations

import time

from ragusa_a.agf_novelty import load_cegis_novelty

# (label, apply arrow adapter, column list)
VERDICT_SETS = (
    ("arrow adapter, table columns", True, "table"),
    ("arrow adapter, _GC2TABLE columns", True, "gc2table"),
    ("no adapter, table columns", False, "table"),
)


def classify(names, table_columns) -> tuple[dict, str]:
    cn, commit = load_cegis_novelty()
    out = {}
    for label, adapt, colset in VERDICT_SETS:
        cols = list(table_columns) if colset == "table" else list(cn._GC2TABLE.keys())
        t0 = time.perf_counter()
        verdicts, errors = [], 0
        for name in names:
            stmt = name.replace("→", "⇒") if adapt else name
            try:
                verdicts.append(cn.classify_statement(stmt, cols))
            except Exception as e:  # counted and reported, never silently dropped
                errors += 1
                verdicts.append((False, f"error: {type(e).__name__}"))
        out[label] = {
            "verdicts": verdicts,
            "errors": errors,
            "n_cols": len(cols),
            "wall_s": round(time.perf_counter() - t0, 3),
        }
    return out, commit
