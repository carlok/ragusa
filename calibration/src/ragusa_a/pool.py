"""Candidate pool on a TxGraffiti table: shipped generators and shipped dedup, evaluated once.

Every array comes from txgraffiti's own objects (`hypothesis(df)`, `rhs(df)`,
`conj(df)`, `slack(df)`), evaluated once on the full table, so the analysis
kernels never re-enter pandas and never re-derive a value txgraffiti computes.
"""

from __future__ import annotations

import os
import pathlib
import shutil
import time
from collections import defaultdict
from dataclasses import dataclass
from itertools import combinations

import numpy as np
import pandas as pd
import pulp
from txgraffiti.generators import convex_hull, linear_programming, ratios
from txgraffiti.heuristics.davila import normalize_inequality_key
from txgraffiti.logic import Property
from txgraffiti.processing import auto_wrap, remove_duplicates

GENERATORS = {"linear_programming": linear_programming, "convex_hull": convex_hull, "ratios": ratios}
# txgraffiti.systems.txgraffiti2's default order; dedup and Morgan are order-dependent.
DEFAULT_GENERATORS = ("linear_programming", "convex_hull", "ratios")


def setup_solver(gen_names) -> str | None:
    """txgraffiti's get_available_solver() only looks for `cbc`/`glpsol` on PATH and
    ignores PuLP's bundled binary. Put the bundled one on PATH; the package is untouched."""
    if "linear_programming" not in gen_names:
        return None
    if not (shutil.which("cbc") or shutil.which("glpsol")):
        bundled = pulp.PULP_CBC_CMD(msg=False)          # PuLP picks the binary for this platform
        if bundled.available():
            cbc_dir = pathlib.Path(os.path.realpath(bundled.path)).parent
            os.environ["PATH"] = f"{cbc_dir}{os.pathsep}{os.environ['PATH']}"
    return f"lp solver: {_solver_id(shutil.which('cbc') or shutil.which('glpsol'))} (pulp {pulp.__version__})"


def _solver_id(path: str | None) -> str:
    """Which binary was used, without recording where this machine keeps it.

    The absolute path ended up in committed run artifacts, so it is reported relative to
    PuLP's package directory when the bundled binary is used, and as a bare name otherwise."""
    if path is None:
        return "none"
    p = pathlib.Path(path)
    bundled = pathlib.Path(pulp.__file__).parent
    if p.is_relative_to(bundled):
        return f"pulp-bundled {p.relative_to(bundled)}"
    return f"{p.name} on PATH"


def generate(df, target, features, preds, gen_names, feature_size=1):
    """Generation loop of txgraffiti2, with a per-generator candidate count.

    `safe_generator` swallows exceptions, so a broken generator looks like one that
    found nothing. Refuse to continue in that case."""
    raw = []
    per_gen = dict.fromkeys(gen_names, 0)
    for feat_tuple in combinations(features, feature_size):
        for name in gen_names:
            for hyp in preds:
                out = list(GENERATORS[name](df=df, features=list(feat_tuple), target=target, hypothesis=hyp))
                per_gen[name] += len(out)
                raw.extend(out)
    empty = [name for name, n in per_gen.items() if n == 0]
    if empty:
        raise SystemExit(f"generator(s) yielded no candidates: {empty} (per_gen={per_gen})")
    return raw, per_gen


def _values(x, n):
    """A txgraffiti evaluation as Python scalars (Fractions kept exact)."""
    if isinstance(x, pd.Series):
        return np.array(x.tolist(), dtype=object)
    return np.array([x] * n, dtype=object)


@dataclass
class Pool:
    df: pd.DataFrame
    table: str
    table_sha256: str
    target: str
    gen_names: list
    features: list
    preds: list
    solver_note: str | None
    per_gen: dict
    n_raw: int
    gen_wall: float
    dedup_wall: float
    conjs: list
    names: list
    hyp: list            # hypothesis name per candidate
    hidx: np.ndarray     # index into Hu
    Hu: np.ndarray       # (h, N) distinct hypothesis masks
    lhs: list
    op: list
    upper: np.ndarray    # op is '<=' (the only op dalmatian_accept collects as an old bound)
    R_obj: np.ndarray    # (M, N) rhs values, exact Python scalars
    R_float: np.ndarray  # (M, N) rhs values as float64
    cex: np.ndarray      # (M, N) rows violating the conjecture
    tight: np.ndarray    # (M, N) hypothesis holds and slack == 0
    concl_key: list      # normalize_inequality_key(conclusion)
    class_id: np.ndarray # table-equivalence class (same lhs, op, hypothesis rows, rhs values there)

    @property
    def M(self):
        return len(self.conjs)

    @property
    def N(self):
        return len(self.df)

    @property
    def is_false(self):
        return self.cex.any(axis=1)

    def object_label(self, r: int) -> dict:
        row = {"row": int(r)}
        for col in ("order", "size"):
            if col in self.df.columns:
                row[col] = int(self.df[col].iloc[r])
        return row


def build_pool(df, *, table, table_sha256, target, gen_names=DEFAULT_GENERATORS,
               n_features=None, n_preds=None, feature_size=1) -> Pool:
    numeric, bool_preds = auto_wrap(df)
    target_prop = Property(target, lambda d, c=target: d[c])
    features = [p for p in numeric if p.name != target][:n_features]
    preds = bool_preds[:n_preds]
    solver_note = setup_solver(gen_names)

    t0 = time.perf_counter()
    raw, per_gen = generate(df, target_prop, features, preds, list(gen_names), feature_size)
    gen_wall = time.perf_counter() - t0
    t0 = time.perf_counter()
    conjs = remove_duplicates(raw, df)
    dedup_wall = time.perf_counter() - t0

    names = [c.name for c in conjs]
    if len(set(names)) != len(names):
        raise AssertionError("candidate names not unique after remove_duplicates")

    M, N = len(conjs), len(df)
    R_obj = np.empty((M, N), dtype=object)
    cex = np.zeros((M, N), dtype=bool)
    tight = np.zeros((M, N), dtype=bool)
    H = np.zeros((M, N), dtype=bool)
    hyp, lhs, op, concl_key = [], [], [], []
    for i, c in enumerate(conjs):
        concl = c.conclusion
        if concl.op not in ("<=", ">="):
            raise ValueError(f"unexpected operator {concl.op!r} in {c.name}")
        H[i] = c.hypothesis(df).to_numpy(dtype=bool)
        R_obj[i] = _values(concl.rhs(df), N)
        cex[i] = ~c(df).to_numpy(dtype=bool)
        tight[i] = (c.hypothesis(df) & (concl.slack(df) == 0)).to_numpy(dtype=bool)
        hyp.append(c.hypothesis.name)
        lhs.append(concl.lhs.name)
        op.append(concl.op)
        concl_key.append(normalize_inequality_key(concl))

    hyp_names = list(dict.fromkeys(hyp))
    hidx = np.array([hyp_names.index(h) for h in hyp])
    Hu = np.array([H[hyp.index(h)] for h in hyp_names])
    if not (Hu[hidx] == H).all():
        raise AssertionError("hypothesis mask not determined by hypothesis name")

    classes: dict = defaultdict(lambda: len(classes))
    class_id = np.array([
        classes[(lhs[i], op[i], H[i].tobytes(), tuple(R_obj[i][H[i]]))] for i in range(M)
    ])

    return Pool(
        df=df, table=table, table_sha256=table_sha256, target=target, gen_names=list(gen_names),
        features=[p.name for p in features], preds=[p.name for p in preds], solver_note=solver_note,
        per_gen=per_gen, n_raw=len(raw), gen_wall=gen_wall, dedup_wall=dedup_wall,
        conjs=conjs, names=names, hyp=hyp, hidx=hidx, Hu=Hu, lhs=lhs, op=op,
        upper=np.array([o == "<=" for o in op]), R_obj=R_obj, R_float=R_obj.astype(float),
        cex=cex, tight=tight, concl_key=concl_key, class_id=class_id,
    )
