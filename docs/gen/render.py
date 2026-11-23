"""Render every document whose numbers come from results/.

    python3 docs/gen/render.py              # lint templates, then render
    python3 docs/gen/render.py --lint-only

Templates use ${key} placeholders filled from results/rows/. A digit left in a
template outside a placeholder must match an allowlisted non-result pattern
(section numbers, versions, years, identifiers, URLs); otherwise the run fails.
Standard library only.
"""

from __future__ import annotations

import argparse
import csv
import json
import pathlib
import re
import string
import sys

REPO = pathlib.Path(__file__).resolve().parents[2]
ROWS = REPO / "results" / "rows"
CENSUS = "census_graph_data_independence_number"
TEMPLATES = {
    REPO / "calibration/note/calibration.md.tmpl": REPO / "calibration/note/calibration.md",
    # Letters to third parties are private correspondence: templates and rendered drafts both live
    # in the gitignored external/ tree. On a clone without them only the calibration note renders.
    REPO / "external/outreach/templates/davila_email.md.tmpl": REPO / "external/outreach/davila_email.md",
    REPO / "external/outreach/templates/pastorek_email.md.tmpl": REPO / "external/outreach/pastorek_email.md",
}
PLACEHOLDER = re.compile(r"\$\{[a-z0-9_]+\}")
ALLOWED = [
    r"https?://\S+",
    r"[\w.+-]+@[\w.-]+",
    r"arXiv:\d{4}\.\d{4,5}(?:v\d+)?",
    r"10\.\d{4,}/[^\s),]+",
    r"§\s?\d+(?:\.\d+)*",
    r"\b\d+\.\d+\.\d+\b",
    r"\b(?:19|20)\d{2}\b",
    r"\bG-A\d\b",
    r"\bSprint \d\b",
    r"\bLean 4\b",
    r"\bv\d\b",
    r"(?m)^\s*\d+\.\s",                       # list markers
    r"(?m)^#+\s*\d+\.",                       # numbered headings
    r"\b(?=\w*[A-Za-z_])(?=\w*\d)\w+\b",      # identifiers mixing letters and digits (sha256, _GC2TABLE)
    r"[≤≥<>=]\s*\d+\b",                       # constants inside theorem statements (χ ≤ 2)
    r"\b\d+·",                                # coefficients inside theorem statements (2·rad)
]

RULES = ("dalmatian", "sophie", "morgan")
RULE_TITLE = {
    "dalmatian": "`filter_with_dalmatian` (heuristics.fajtlowicz)",
    "sophie": "`sophie_accept`, greedy (heuristics.delavina)",
    "morgan": "`filter_with_morgan` (heuristics.davila)",
}
METRIC_TITLE = {
    "tight_objects": "objects with a tight survivor",
    "false_kept": "false conjectures kept",
    "hypotheses_kept": "hypotheses kept",
    "distinct_table_classes": "distinct table classes kept",
}


def lint(path: pathlib.Path) -> list[str]:
    problems = []
    for n, line in enumerate(path.read_text().splitlines(), 1):
        stripped = PLACEHOLDER.sub(" ", line)
        for pat in ALLOWED:
            stripped = re.sub(pat, " ", stripped)
        if re.search(r"\d", stripped):
            problems.append(f"{path.relative_to(REPO)}:{n}: hand-typed digit: {line.strip()[:100]}")
    return problems


def load_jsonl(name):
    return [json.loads(line) for line in (ROWS / f"{name}.jsonl").read_text().splitlines()]


def pct(a, b):
    return f"{100 * a / b:.1f}%" if b else "n/a"


def md_table(header, body):
    out = ["| " + " | ".join(header) + " |", "|" + "|".join("---" for _ in header) + "|"]
    out += ["| " + " | ".join(str(c) for c in row) + " |" for row in body]
    return "\n".join(out)


def vs_chance(stat):
    if stat["real"] > stat["random_p97_5"]:
        return "above the random 95% range"
    if stat["real"] < stat["random_p2_5"]:
        return "below the random 95% range"
    return "inside the random 95% range"


def build_context() -> dict:
    ctx: dict = {}
    m = json.loads((ROWS / f"{CENSUS}.metrics.json").read_text())
    pin, pool = m["pinning"], m["pool"]
    M, N = pool["n_dedup"], pool["n_objects"]
    ctx.update(
        txg_version=pin["txgraffiti"], table=pin["table"], target=pin["target"],
        table_sha_short=pin["table_sha256"][:12], n_features=len(pin["features"]),
        n_preds=len(pin["predicates"]), generators=", ".join(f"`{g}`" for g in pin["generators"]),
        seed=pin["seed"], chance_draws=pin["chance_draws"],
        pool_raw=pool["n_raw"], pool_dedup=M, n_objects=N, pool_false=pool["n_false"],
        pool_classes=pool["n_table_classes"], pool_hyps=pool["n_hypotheses"],
        dedup_killed=pool["n_raw"] - M, dedup_pct=pct(pool["n_raw"] - M, pool["n_raw"]),
        per_generator=", ".join(f"`{g}` {n}" for g, n in pool["per_generator"].items()),
    )

    # number 1
    body = []
    for rule in RULES:
        p = m["rules"][rule]["partition"]
        killed = p["n_alias"] + p["n_discarded"]
        body.append([RULE_TITLE[rule], p["n_in"], p["n_representative"], p["n_alias"], p["n_discarded"],
                     p["n_discarded_false"], f"{killed} ({pct(killed, p['n_in'])})", m["rules"][rule]["shipped_wall_s"]])
        ctx[f"{rule}_rep"] = p["n_representative"]
        ctx[f"{rule}_alias"] = p["n_alias"]
        ctx[f"{rule}_disc"] = p["n_discarded"]
        ctx[f"{rule}_killed"] = killed
        ctx[f"{rule}_kill_pct"] = pct(killed, p["n_in"])
        ctx[f"{rule}_rep_false"] = p["n_representative_false"]
        ctx["pool_size"] = p["n_in"]
    ctx["table_partition"] = md_table(
        ["rule (shipped code)", "input", "representative", "alias", "discarded", "discarded & false",
         "killed", "wall s"], body)

    d = m["rules"]["dalmatian"].get("dalmatian", {})
    for key, v in d.items():
        ctx[f"dal_{key}"] = v

    # Dalmatian by operator, from the per-candidate decisions
    with (ROWS / f"{CENSUS}.decisions.csv").open(newline="") as fh:
        dec = list(csv.DictReader(fh))
    for op, tag in (("<=", "upper"), (">=", "lower")):
        mine = [r for r in dec if r["op"] == op]
        true = [r for r in mine if r["is_false"] == "False"]
        ctx[f"pool_{tag}"] = len(mine)
        ctx[f"pool_{tag}_true"] = len(true)
        ctx[f"dal_rep_{tag}"] = sum(r["dalmatian_class"] == "representative" for r in mine)
        ctx[f"dal_rej_{tag}_true"] = sum(r["dalmatian_class"] != "representative" for r in true)
    ctx["dal_lower_reading"] = ("so on this table Dalmatian filters lower bounds by validity alone"
                                if ctx["dal_rej_lower_true"] == 0 else
                                "so lower bounds are also filtered beyond validity")

    # number 2
    body = []
    for rule in RULES:
        fl = m["rules"][rule].get("floor")
        if fl is None:
            body.append([RULE_TITLE[rule], "kernel failed its gate; no floor reported", "", "", "", "", ""])
            continue
        top = ", ".join(f"row {o['row']} (n={o.get('order', '?')}, m={o.get('size', '?')}): {o['flips']}"
                        for o in fl["top_objects"]) or "none"
        body.append([RULE_TITLE[rule], f"{fl['n_fragile']} ({pct(fl['n_fragile'], fl['n_decisions'])})",
                     fl["n_fragile_accepted"], fl["n_fragile_rejected"], fl["objects_with_flips"],
                     f"{fl['k_objects_50pct_flips']} / {fl['k_objects_90pct_flips']}", top])
        for key in ("n_fragile", "n_fragile_accepted", "n_fragile_rejected", "objects_with_flips", "total_flips",
                    "k_objects_50pct_flips", "k_objects_90pct_flips", "n_fragile_via_top_objects"):
            ctx[f"{rule}_{key}"] = fl[key]
        ctx[f"{rule}_fragile_pct"] = pct(fl["n_fragile"], fl["n_decisions"])
        ctx[f"{rule}_top_objects_count"] = len(fl["top_objects"])
    ctx["table_floor"] = md_table(
        ["rule", "fragile decisions", "fragile & accepted", "fragile & rejected", "objects with a flip",
         "objects for 50% / 90% of flips", "top objects: flips"], body)

    # number 3
    body = []
    for rule in RULES:
        ch = m["rules"][rule]["chance"]
        for key, stat in ch["metrics"].items():
            body.append([RULE_TITLE[rule], ch["k"], METRIC_TITLE[key], stat["real"],
                         f"{stat['random_mean']} [{stat['random_p2_5']:g}, {stat['random_p97_5']:g}]",
                         stat["frac_random_ge_real"], vs_chance(stat)])
            ctx[f"{rule}_{key}_vs_chance"] = vs_chance(stat)
    ctx["table_chance"] = md_table(
        ["rule", "accepted (k)", "metric", "rule", "random mean [2.5%, 97.5%]", "P(random ≥ rule)", "reading"], body)

    # pipeline
    ctx["pipe_after_dalmatian"] = m["pipeline"]["n_after_dalmatian"]
    ctx["pipe_after_morgan"] = m["pipeline"]["n_after_morgan"]
    g5 = m["gates"].get("G-A5:txgraffiti2")
    ctx["txgraffiti2_count"] = g5["txgraffiti2_count"] if g5 else "not run"

    # number 4, TxGraffiti corpus
    ctx["agf_commit_short"] = m["agf"]["commit"][:7]
    body = []
    for label, s in m["agf"]["sets"].items():
        body.append([label, s["n_cols"], s["errors"], f"{s['known_in_pool']} ({pct(s['known_in_pool'], M)})",
                     f"{s['known_in_pipeline_survivors']} ({pct(s['known_in_pipeline_survivors'], ctx['pipe_after_morgan'])})"])
    first = next(iter(m["agf"]["sets"].values()))
    ctx["agf_txg_known_pool"] = first["known_in_pool"]
    ctx["agf_txg_known_survivors"] = first["known_in_pipeline_survivors"]
    ctx["table_agf_txg"] = md_table(
        ["verdict set", "columns", "errors", f"known in pool", "known in pipeline survivors"], body)

    # number 4 / G-A2, AutoGraphForge's own corpus
    agf = {r["filter"]: r for r in load_jsonl("agf_novelty")}
    std, stored, both = (agf["classify_statement[_GC2TABLE]"], agf["stored metadata.novel"],
                         agf["overlap[_GC2TABLE & stored]"])
    ctx.update(agf_cegis_n=std["n_in"], agf_gc2_known=std["n_discarded"], agf_stored_known=stored["n_discarded"],
               agf_overlap=both["n_discarded"], agf_either_known=both["n_discarded"] + both["n_alias"],
               agf_one_only=both["n_alias"], agf_gc2_only=std["n_discarded"] - both["n_discarded"],
               agf_stored_only=stored["n_discarded"] - both["n_discarded"], agf_gc2_wall=std["wall_s"])

    # G-A1
    base = {r["filter"]: r for r in load_jsonl("txg_independence_number_convex_hull-ratios_f12_p6")}
    lp = {r["filter"]: r for r in load_jsonl("txg_independence_number_convex_hull-ratios-linear_programming_f12_p6")}
    for tag, stages in (("ga1", base), ("ga1lp", lp)):
        for stage in ("generated", "remove_duplicates", "filter_with_dalmatian", "filter_with_morgan"):
            ctx[f"{tag}_{stage}"] = stages[stage]["n_representative"]

    # gates
    body = []
    for gate, v in m["gates"].items():
        detail = {k: v[k] for k in v if k not in ("pass", "checks")}
        if "checks" in v:
            detail = f"{len(v['checks'])} removals re-run through shipped code, mismatches " + \
                     ", ".join(str(c["mismatches"]) for c in v["checks"])
        body.append([gate, "pass" if v["pass"] else "**FAIL**", detail])
    ctx["table_gates"] = md_table(["gate", "result", "detail"], body)
    ctx["all_gates_pass"] = "all pass" if all(v["pass"] for v in m["gates"].values()) else "**at least one failed**"
    return {k: str(v) for k, v in ctx.items()}


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--lint-only", action="store_true")
    args = ap.parse_args()

    problems = [p for tmpl in TEMPLATES if tmpl.exists() for p in lint(tmpl)]
    if problems:
        print("\n".join(problems))
        sys.exit(1)
    print(f"lint ok ({sum(t.exists() for t in TEMPLATES)} templates)")
    if args.lint_only:
        return

    ctx = build_context()
    # Recipients' addresses are third parties' details, so they are not in the tracked templates.
    # They are read from external/outreach/addresses.json (git-ignored); without it the drafts keep a
    # visible FILL IN marker instead of an address.
    addr_file = REPO / "external" / "outreach" / "addresses.json"
    addrs = json.loads(addr_file.read_text()) if addr_file.exists() else {}
    ctx["to_pastorek"] = addrs.get("pastorek", "[FILL IN: Ján Pastorek's address; see arXiv:2609.03478 or the AutoGraphForge repository]")
    ctx["to_davila"] = addrs.get("davila", "[FILL IN: Randy Davila's address; see the TxGraffiti site or arXiv:2409.19379]")
    for tmpl, out in TEMPLATES.items():
        if not tmpl.exists():
            continue
        text = string.Template(tmpl.read_text()).substitute(ctx)  # KeyError on any missing number
        if out.parent.name == "outreach":
            text = text.replace("`", "")  # emails are plain text; the generated tables carry Markdown code marks
        out.write_text(f"<!-- generated by docs/gen/render.py from results/rows; edit {tmpl.name} -->\n{text}")
        print(f"rendered {out.relative_to(REPO)}")


if __name__ == "__main__":
    main()
