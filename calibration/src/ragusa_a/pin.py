"""Pinning and result-row output shared by every Project A script."""

from __future__ import annotations

import datetime
import hashlib
import json
import pathlib

REPO = pathlib.Path(__file__).resolve().parents[3]
ROWS_DIR = REPO / "results" / "rows"
SCHEMA = json.loads((REPO / "results" / "schema.json").read_text())


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def sha256_file(path: pathlib.Path) -> str:
    return sha256_bytes(pathlib.Path(path).read_bytes())


def run_id(system: str) -> str:
    stamp = datetime.datetime.now(datetime.timezone.utc).strftime("%Y%m%dT%H%M%SZ")
    return f"{system}-{stamp}"


def check_row(row: dict) -> None:
    """Enforce required keys, no unknown keys, and enum values from the schema."""
    props = SCHEMA["properties"]
    missing = [k for k in SCHEMA["required"] if k not in row]
    unknown = [k for k in row if k not in props]
    if missing or unknown:
        raise ValueError(f"row schema violation: missing={missing} unknown={unknown}")
    for key, spec in props.items():
        if "enum" in spec and key in row and row[key] not in spec["enum"]:
            raise ValueError(f"row schema violation: {key}={row[key]!r} not in {spec['enum']}")
    parts = row["n_representative"] + row["n_alias"] + row["n_discarded"]
    if parts != row["n_in"]:
        raise ValueError(f"partition does not sum: {parts} != n_in={row['n_in']}")


def write_rows(name: str, rows: list[dict]) -> pathlib.Path:
    """Validate and write rows as JSON lines to results/rows/<name>.jsonl."""
    for row in rows:
        check_row(row)
    ROWS_DIR.mkdir(parents=True, exist_ok=True)
    path = ROWS_DIR / f"{name}.jsonl"
    with path.open("w") as fh:
        for row in rows:
            fh.write(json.dumps(row, ensure_ascii=False) + "\n")
    return path
