"""Exhaustive control for the leave-one-object-out kernels on a small table.

For every rule and every removed object, the kernel's decision vector must equal
the shipped txgraffiti rule re-run on the table without that object.
"""

import numpy as np
import pytest
from txgraffiti.example_data import graph_data

from ragusa_a.pool import build_pool
from ragusa_a.rules import KERNELS, RULES, shipped

N_ROWS = 30


@pytest.fixture(scope="module")
def pool():
    df = graph_data.iloc[:N_ROWS]
    return build_pool(df, table="graph_data[:30]", table_sha256="0" * 64, target="independence_number",
                      gen_names=("convex_hull", "ratios"), n_features=4, n_preds=5)


@pytest.mark.parametrize("rule", RULES)
def test_kernel_matches_shipped_on_every_removal(pool, rule):
    acc = shipped(rule, pool.conjs, pool.df)
    full, loo, _ = KERNELS[rule](pool, exact=True)
    assert (full == acc).all()
    for r in range(pool.N):
        got = shipped(rule, pool.conjs, pool.df.drop(index=pool.df.index[r]))
        assert np.array_equal(got, loo[:, r]), f"{rule}: removal of row {r}"


def test_pool_is_nontrivial(pool):
    # the control is only meaningful if the rules actually reject something
    assert pool.M > 20
    assert not shipped("dalmatian", pool.conjs, pool.df).all()
