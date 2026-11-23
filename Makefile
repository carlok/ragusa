# `just` is not installed on this machine; plain make keeps setup at zero.
PY := calibration/.venv/bin/python

.PHONY: test reproduce-a reproduce-b docs frontier-gates

# Frontier tables (git-ignored external/): export to ragusa CSV, then gates G-B6 (exports load as
# their manifests say) and G-B9 (the knot catalogue is not refuted by KnotInfo).
# Needs the Sprint 6 downloads in external/knotinfo and external/fano3c.
frontier-gates:
	cd calibration && uv run python -m ragusa_a.export_knotinfo
	cd calibration && uv run python -m ragusa_a.export_fano
	cd engine && cargo test --release -p ragusa-table --test g_b6 -- --ignored
	cd engine && cargo test --release -p ragusa-report --test g_b9 -- --ignored --nocapture

test:
	@python3 -c "import json; json.load(open('results/schema.json'))" && echo "schema ok"
	@python3 docs/gen/render.py --lint-only
	@if [ -f calibration/pyproject.toml ]; then cd calibration && uv run python -m pytest -q; fi
	@if [ -f engine/Cargo.toml ]; then cd engine && cargo test --workspace; fi

# Project A from scratch: G-A1, G-A2, the census (numbers 1-4), then every rendered document.
reproduce-a:
	cd calibration && uv sync --frozen
	cd calibration && uv run python -m ragusa_a.run_txg
	cd calibration && uv run python -m ragusa_a.run_txg --generators convex_hull,ratios,linear_programming
	cd calibration && uv run python -m ragusa_a.agf_novelty
	cd calibration && uv run python -m ragusa_a.census
	$(MAKE) docs

# Project B: regenerate the pandas reference fixture, then every engine gate.
reproduce-b:
	cd calibration && uv run python -m ragusa_a.export_manifest
	cd calibration && uv run python -m ragusa_a.export_generator_fixtures
	cd engine && cargo test --workspace --release
	cd engine && cargo run --release -q -p ragusa-cli -- floor graph_data --target independence_number
	cd engine && cargo run --release -q -p ragusa-cli -- floor graph_data --target independence_number --redundancy sharp_coverage --out ../results/floor/graph_data_independence_number_sharp_coverage
	cd calibration && uv run python -m ragusa_a.compare_b4
	cd engine && cargo run --release -q -p ragusa-cli -- implications magma_corpus_etp4694

docs:
	python3 docs/gen/render.py
