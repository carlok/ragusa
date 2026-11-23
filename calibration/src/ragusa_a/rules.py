"""The three shipped acceptance rules, called unmodified, and leave-one-object-out kernels.

`shipped` calls txgraffiti 0.4.1 and produces every published decision.

The `*_kernel` functions compute `loo[i, r]`: the decision the shipped rule would
make on candidate i if object r were removed from the table, holding the candidate
pool fixed. They are analysis instruments only. `census` requires each kernel to
match `shipped` exactly on the full table and on sampled removals, and
`tests/test_rules.py` checks every removal exhaustively on a small table.
"""

from __future__ import annotations

from collections import defaultdict

import numpy as np
from txgraffiti.heuristics.delavina import sophie_accept
from txgraffiti.processing import filter_with_dalmatian, filter_with_morgan

RULES = ("dalmatian", "sophie", "morgan")
RULE_LABEL = {
    "dalmatian": "filter_with_dalmatian",
    "sophie": "sophie_accept[greedy]",
    "morgan": "filter_with_morgan",
}


def _mask(conjs, kept) -> np.ndarray:
    ids = {id(c) for c in kept}
    return np.array([id(c) in ids for c in conjs], dtype=bool)


def shipped(rule, conjs, df) -> np.ndarray:
    if rule == "dalmatian":
        return _mask(conjs, filter_with_dalmatian(conjs, df))
    if rule == "morgan":
        return _mask(conjs, filter_with_morgan(conjs, df))
    if rule == "sophie":
        # The package ships no filter_with_sophie. Drive sophie_accept greedily in
        # pool order against the previously accepted list, as its docstring describes.
        accepted = []
        for c in conjs:
            if sophie_accept(c, accepted, df):
                accepted.append(c)
        return _mask(conjs, accepted)
    raise ValueError(rule)


def dalmatian_kernel(pool, exact=True):
    """Mirror of filter_with_dalmatian: accept iff valid on every row and either no other
    pool candidate shares the hypothesis name, lhs name and op '<=', or the rhs is strictly
    below those candidates' row-wise minimum on at least one row (any row, hypothesis or not).

    Removing row r only changes validity (if r is the sole counterexample) and the strict
    set (if r is its only row), so every removal follows from two counts per candidate."""
    R = pool.R_obj if exact else pool.R_float
    M, N = R.shape
    groups = defaultdict(list)
    for j in range(M):
        if pool.upper[j]:
            groups[(pool.hyp[j], pool.lhs[j])].append(j)
    strict = np.zeros((M, N), dtype=bool)
    no_others = np.zeros(M, dtype=bool)
    for i in range(M):
        others = [j for j in groups.get((pool.hyp[i], pool.lhs[i]), ()) if j != i]
        if not others:
            no_others[i] = True
            continue
        strict[i] = np.asarray(R[i] < R[others].min(axis=0), dtype=bool)

    ncex = pool.cex.sum(axis=1)
    nstrict = strict.sum(axis=1)
    full = (ncex == 0) & (no_others | (nstrict > 0))
    valid_minus = (ncex == 0)[:, None] | ((ncex == 1)[:, None] & pool.cex)
    strict_minus = (nstrict > 1)[:, None] | ((nstrict == 1)[:, None] & ~strict)
    loo = valid_minus & (no_others[:, None] | strict_minus)
    return full, loo, {"strict": strict, "no_others": no_others}


def _sophie_greedy(masks) -> np.ndarray:
    covered = np.zeros(masks.shape[1], dtype=bool)
    acc = np.zeros(masks.shape[0], dtype=bool)
    for i, m in enumerate(masks):
        if (m & ~covered).any():
            acc[i] = True
            covered |= m
    return acc


def sophie_kernel(pool, exact=True):
    """Mirror of the greedy sophie_accept driver: accept iff the hypothesis covers a row
    not covered by the union of previously accepted hypotheses."""
    M, N = pool.M, pool.N
    full = _sophie_greedy(pool.Hu[pool.hidx])
    loo = np.empty((M, N), dtype=bool)
    keep = np.ones(N, dtype=bool)
    for r in range(N):
        keep[r] = False
        loo[:, r] = _sophie_greedy(pool.Hu[:, keep][pool.hidx])
        keep[r] = True
    return full, loo, {}


def _strict_subset_matrix(Hu) -> np.ndarray:
    """sub[a, b]: mask a is a strict subset of mask b (is_strict_subset semantics)."""
    cnt = Hu.sum(axis=1)
    violates = (Hu[:, None, :] & ~Hu[None, :, :]).any(axis=2)
    return ~violates & (cnt[None, :] > cnt[:, None])


def _morgan_greedy(hidx, key, sub) -> np.ndarray:
    """Mirror of filter_with_morgan. Only accepted candidates with the same normalized
    conclusion interact, so one ordered list per conclusion reproduces the single list."""
    by_key = defaultdict(list)
    for new in range(len(hidx)):
        keep = True
        lst = by_key[key[new]]
        for old in lst[:]:
            if sub[hidx[new], hidx[old]]:
                keep = False
                break
            elif sub[hidx[old], hidx[new]]:
                lst.remove(old)
        if keep:
            lst.append(new)
    acc = np.zeros(len(hidx), dtype=bool)
    for lst in by_key.values():
        acc[lst] = True
    return acc


def morgan_kernel(pool, exact=True):
    M, N = pool.M, pool.N
    full = _morgan_greedy(pool.hidx, pool.concl_key, _strict_subset_matrix(pool.Hu))
    loo = np.empty((M, N), dtype=bool)
    keep = np.ones(N, dtype=bool)
    for r in range(N):
        keep[r] = False
        loo[:, r] = _morgan_greedy(pool.hidx, pool.concl_key, _strict_subset_matrix(pool.Hu[:, keep]))
        keep[r] = True
    return full, loo, {}


KERNELS = {"dalmatian": dalmatian_kernel, "sophie": sophie_kernel, "morgan": morgan_kernel}
