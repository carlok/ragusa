"""Partition, object-database floor and chance baseline over shipped decisions."""

from __future__ import annotations

import numpy as np


def partition(pool, accept, among=None) -> dict:
    """Representative / alias / discarded over the candidates in `among`.

    Shipped filters return lists, so the alias class is defined here, not by
    txgraffiti: a rejected candidate is an alias when some accepted candidate is
    identical to it on the table (same lhs, op, hypothesis rows and rhs values there).
    """
    M = pool.M
    among = np.ones(M, dtype=bool) if among is None else among
    if (accept & ~among).any():
        raise ValueError("accepted candidate outside the input set")
    reps = {}
    for i in np.flatnonzero(accept):
        reps.setdefault(pool.class_id[i], i)
    cls = np.full(M, "", dtype=object)
    alias_of = np.full(M, -1)
    for i in np.flatnonzero(among):
        if accept[i]:
            cls[i] = "representative"
        elif pool.class_id[i] in reps:
            cls[i] = "alias"
            alias_of[i] = reps[pool.class_id[i]]
        else:
            cls[i] = "discarded"
    return {
        "cls": cls,
        "alias_of": alias_of,
        "n_in": int(among.sum()),
        "n_representative": int((cls == "representative").sum()),
        "n_alias": int((cls == "alias").sum()),
        "n_discarded": int((cls == "discarded").sum()),
        "n_discarded_false": int(((cls == "discarded") & pool.is_false).sum()),
        "n_representative_false": int(((cls == "representative") & pool.is_false).sum()),
    }


def floor(pool, accept, loo, top=5) -> dict:
    """Single-object pivotality: which decisions flip when one object is removed."""
    F = loo != accept[:, None]
    per_obj = F.sum(axis=0)
    total = int(per_obj.sum())
    order = np.argsort(-per_obj, kind="stable")
    cum = np.cumsum(per_obj[order])

    def k_for(share):
        return int(np.searchsorted(cum, share * total) + 1) if total else 0

    top_rows = [int(r) for r in order[:top] if per_obj[r] > 0]
    fragile = F.any(axis=1)
    return {
        "n_decisions": int(len(accept)),
        "n_objects": int(pool.N),
        "n_fragile": int(fragile.sum()),
        "n_fragile_accepted": int((fragile & accept).sum()),
        "n_fragile_rejected": int((fragile & ~accept).sum()),
        "n_fragile_via_top_objects": int(F[:, top_rows].any(axis=1).sum()) if top_rows else 0,
        "objects_with_flips": int((per_obj > 0).sum()),
        "total_flips": total,
        "k_objects_50pct_flips": k_for(0.5),
        "k_objects_90pct_flips": k_for(0.9),
        "top_objects": [dict(pool.object_label(r), flips=int(per_obj[r])) for r in top_rows],
        "per_object_flips": per_obj.tolist(),
    }


def chance(pool, accept, rng, draws) -> dict:
    """Uniform random acceptance of the same number of candidates, `draws` times."""
    M, k = pool.M, int(accept.sum())
    is_false = pool.is_false

    def metrics(idx):
        return {
            "tight_objects": int(pool.tight[idx].any(axis=0).sum()),
            "false_kept": int(is_false[idx].sum()),
            "hypotheses_kept": int(np.unique(pool.hidx[idx]).size),
            "distinct_table_classes": int(np.unique(pool.class_id[idx]).size),
        }

    real = metrics(np.flatnonzero(accept))
    samples = {key: np.empty(draws) for key in real}
    for d in range(draws):
        m = metrics(rng.choice(M, size=k, replace=False))
        for key, v in m.items():
            samples[key][d] = v
    out = {}
    for key, v in samples.items():
        out[key] = {
            "real": real[key],
            "random_mean": round(float(v.mean()), 2),
            "random_p2_5": float(np.percentile(v, 2.5)),
            "random_p97_5": float(np.percentile(v, 97.5)),
            "frac_random_ge_real": round(float((v >= real[key]).mean()), 4),
            "frac_random_le_real": round(float((v <= real[key]).mean()), 4),
        }
    return {"k": k, "M": M, "draws": draws, "metrics": out}
