import pytest

from ragusa_a import pin


def row(**overrides):
    base = dict(
        run_id="t", system="txgraffiti", system_version="0.4.1", table="graph_data",
        table_sha256="0" * 64, target="independence_number", generator="ratios",
        filter="filter_with_morgan", filter_kind="redundancy", stage=3, order="a>b",
        n_in=10, n_representative=6, n_alias=1, n_discarded=3, seed=None,
    )
    base.update(overrides)
    return base


def test_valid_row_passes():
    pin.check_row(row())


def test_partition_must_sum_to_n_in():
    with pytest.raises(ValueError, match="partition does not sum"):
        pin.check_row(row(n_discarded=4))


def test_unknown_key_rejected():
    with pytest.raises(ValueError, match="unknown"):
        pin.check_row(row(kill_rate=0.3))


def test_missing_key_rejected():
    r = row()
    del r["filter_kind"]
    with pytest.raises(ValueError, match="missing"):
        pin.check_row(r)


def test_filter_kind_enum_enforced():
    with pytest.raises(ValueError, match="filter_kind"):
        pin.check_row(row(filter_kind="dedup"))
