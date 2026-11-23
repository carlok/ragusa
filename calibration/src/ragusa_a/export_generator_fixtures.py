"""Reference fixtures for gate G-B2: what txgraffiti 0.4.1's generators emit, recovered exactly.

    uv run python -m ragusa_a.export_generator_fixtures

Writes to engine/fixtures/:

- txgraffiti_candidates.json: every candidate of each generator, per (feature, hypothesis)
  call, for the PROMPT.md section 1.1 scope and the full graph_data scope. txgraffiti keeps no
  expression tree, only canonical names built by `Property._binop`, so coefficients are parsed
  back from those names into exact fractions, and every parse is checked by evaluating it on the
  table against txgraffiti's own `rhs(df)`.
- limit_denominator.json: Python's `Fraction(x).limit_denominator()` on edge cases and random floats.
- graph_data_default_parse.json: graph_data values that pandas' default parser (what txgraffiti
  computes on) reads differently from the correctly rounded parse (what ragusa loads).
"""

from __future__ import annotations

import importlib.resources
import json
import struct
from fractions import Fraction

import numpy as np
import pandas as pd
import txgraffiti
from txgraffiti.example_data import graph_data
from txgraffiti.logic import Property
from txgraffiti.processing import auto_wrap

from ragusa_a import pin
from ragusa_a.pool import GENERATORS, setup_solver

FIXTURES = pin.REPO / "engine" / "fixtures"
TARGET = "independence_number"
MAX_DEN = 1_000_000
OPS = (" + ", " * ", " - ", " / ")
SCOPES = {
    "section_1_1": {"n_features": 12, "n_preds": 6},
    "graph_data_full": {"n_features": None, "n_preds": None},
}


def f64_hex(x) -> str:
    return format(struct.unpack("<Q", struct.pack("<d", float(x)))[0], "016x")


def _split_top(s: str):
    """'(A op B)' -> (A, op, B) when the outer parentheses enclose the whole string."""
    if not (s.startswith("(") and s.endswith(")")):
        return None
    depth = 0
    for i, ch in enumerate(s):
        if ch == "(":
            depth += 1
        elif ch == ")":
            depth -= 1
            if depth == 0 and i != len(s) - 1:
                return None
        elif depth == 1:
            for op in OPS:
                if s.startswith(op, i):
                    return s[1:i], op.strip(), s[i + len(op):-1]
    return None


def linear(s: str, features: set[str]):
    """A txgraffiti expression name -> ({feature: Fraction}, Fraction constant)."""
    parts = _split_top(s)
    if parts is None:
        if s in features:
            return {s: Fraction(1)}, Fraction(0)
        return {}, Fraction(s)
    a, op, b = parts
    (ca, ka), (cb, kb) = linear(a, features), linear(b, features)
    if op == "+":
        coefs = dict(ca)
        for f, v in cb.items():
            coefs[f] = coefs.get(f, Fraction(0)) + v
        return {f: v for f, v in coefs.items() if v != 0}, ka + kb
    if op == "*":
        if not ca:
            return {f: v * ka for f, v in cb.items() if v * ka != 0}, ka * kb
        if not cb:
            return {f: v * kb for f, v in ca.items() if v * kb != 0}, ka * kb
        raise ValueError(f"nonlinear product in {s!r}")
    raise ValueError(f"unexpected operator {op!r} in {s!r}")


def record(c, feature: str, hypothesis: str, names: set[str]) -> dict:
    concl = c.conclusion
    assert c.hypothesis.name == hypothesis and concl.lhs.name == TARGET, c.name
    coefs, const = linear(concl.rhs.name, names)
    # the parse must reproduce txgraffiti's own evaluation of the right-hand side
    parsed = sum((float(v) * graph_data[f].to_numpy(float) for f, v in coefs.items()),
                 np.full(len(graph_data), float(const)))
    actual = concl.rhs(graph_data).to_numpy(float)
    assert np.allclose(parsed, actual, rtol=1e-12, atol=1e-9), f"parse mismatch: {c.name}"
    return {
        "feature": feature,
        "hypothesis": hypothesis,
        "op": concl.op,
        "coefs": {f: str(v) for f, v in sorted(coefs.items())},
        "constant": str(const),
        "name": c.name,
    }


def scope(n_features, n_preds) -> dict:
    numeric, preds = auto_wrap(graph_data)
    target = Property(TARGET, lambda d: d[TARGET])
    feats = [p for p in numeric if p.name != TARGET][:n_features]
    preds = preds[:n_preds]
    names = {p.name for p in numeric}
    assert all("(" not in n and " " not in n for n in names), "feature names would confuse the name parser"
    generators = {}
    for g in ("ratios", "convex_hull", "linear_programming"):
        rows = []
        for f in feats:
            for h in preds:
                for c in GENERATORS[g](df=graph_data, features=[f], target=target, hypothesis=h):
                    rows.append(record(c, f.name, h.name, names))
        if not rows:
            raise SystemExit(f"{g} yielded nothing; refusing to write a fixture")
        generators[g] = rows
    return {"target": TARGET, "features": [p.name for p in feats], "predicates": [p.name for p in preds],
            "generators": generators}


def limit_denominator_cases() -> list[dict]:
    rng = np.random.default_rng(0)
    edge = [0.0, -0.0, 1.0, -1.0, 0.5, 1 / 3, -2 / 3, 0.1, 3.141592653589793, 1e-7, 4.9e-7, 5e-7, 5.0000001e-7,
            1e-6, -5e-7, 2.0 ** -40, 2.0 ** -41, 2.0 ** -39, 1e6 + 0.5, 2.0 ** 53, -(2.0 ** 60),
            0.0001976490020751953, 14862 / 619, -630 / 619, 6.882346, 13 / 6, 999999.5, 1 / 999999, 1 / 1000001]
    rand = (list(rng.uniform(-1e3, 1e3, 700))
            + list(np.sign(rng.uniform(-1, 1, 700)) * 10.0 ** rng.uniform(-8, 12, 700))
            + [k / j for k, j in zip(rng.integers(-5000, 5000, 600), rng.integers(1, 3000, 600))]
            + list(rng.integers(-10 ** 6, 10 ** 6, 100).astype(float)))
    out = []
    for x in edge + rand:
        fr = Fraction(float(x)).limit_denominator(MAX_DEN)
        out.append({"x": f64_hex(x), "p": str(fr.numerator), "q": str(fr.denominator)})
    return out


def default_parse_values() -> list[dict]:
    base = importlib.resources.files("txgraffiti.example_data")
    rt = pd.read_csv(base.joinpath("graph_data.csv"), float_precision="round_trip")
    numeric, _ = auto_wrap(graph_data)
    out = []
    for p in numeric:
        a, b = graph_data[p.name].to_numpy(float), rt[p.name].to_numpy(float)
        for r in np.flatnonzero(a != b):
            out.append({"column": p.name, "row": int(r), "default": f64_hex(a[r]), "round_trip": f64_hex(b[r])})
    return out


def main() -> None:
    solver = setup_solver(["linear_programming"])
    header = {"generated_by": "calibration/src/ragusa_a/export_generator_fixtures.py",
              "txgraffiti": txgraffiti.__version__, "solver": solver}
    FIXTURES.mkdir(parents=True, exist_ok=True)

    scopes = {name: scope(**spec) for name, spec in SCOPES.items()}
    (FIXTURES / "txgraffiti_candidates.json").write_text(json.dumps(dict(header, scopes=scopes), indent=1) + "\n")
    for name, s in scopes.items():
        print(f"{name}: " + ", ".join(f"{g} {len(rows)}" for g, rows in s["generators"].items()))

    cases = limit_denominator_cases()
    (FIXTURES / "limit_denominator.json").write_text(
        json.dumps(dict(header, max_denominator=MAX_DEN, cases=cases), indent=0) + "\n")
    print(f"limit_denominator cases: {len(cases)}")

    diffs = default_parse_values()
    (FIXTURES / "graph_data_default_parse.json").write_text(json.dumps(dict(header, values=diffs), indent=1) + "\n")
    print(f"graph_data default-parse differences: {len(diffs)}")


if __name__ == "__main__":
    main()
