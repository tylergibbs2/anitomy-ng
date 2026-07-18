# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.

"""Same batch-parsing suite as anitomy/tests/together.rs, run against the Python
bindings.

Each fixture in anitomy/tests/fixtures/together.json is a *set* of related
filenames plus the ground-truth output each record should produce; a case
passes only when every record matches its expected map exactly. Cases listed in
the shared manifest anitomy/tests/known_failures/together.txt are marked
xfail(strict). Rust is the source of truth for that file; regenerate with
`UPDATE_KNOWN_FAILURES=1 cargo test -p anitomy-ng --test together`.

Requires the extension to be built first: `uv run maturin develop`.
"""

from __future__ import annotations

import json
from pathlib import Path

import pytest

import anitomy_ng as anitomy

_TESTS_ROOT = Path(__file__).resolve().parents[2] / "anitomy" / "tests"
FIXTURES_DIR = _TESTS_ROOT / "fixtures"
KNOWN_FAILURES_DIR = _TESTS_ROOT / "known_failures"


def _normalize(value: str | list[str]) -> list[str]:
    return value if isinstance(value, list) else [value]


def _group(elements: list[anitomy.Element]) -> dict[str, list[str]]:
    grouped: dict[str, list[str]] = {}
    for element in elements:
        grouped.setdefault(element.kind.value, []).append(element.value)
    return grouped


def _known_failures() -> set[str]:
    path = KNOWN_FAILURES_DIR / "together.txt"
    if not path.exists():
        return set()
    return {line for line in path.read_text(encoding="utf-8").splitlines() if line.strip()}


def _load_suite() -> list:
    cases = json.loads((FIXTURES_DIR / "together.json").read_text(encoding="utf-8"))
    known = _known_failures()
    params = []
    for case in cases:
        marks = (
            [pytest.mark.xfail(reason="known batch gap", strict=True)]
            if case["name"] in known
            else []
        )
        params.append(pytest.param(case, id=case["name"], marks=marks))
    return params


@pytest.mark.parametrize("case", _load_suite())
def test_together(case: dict) -> None:
    results = anitomy.parse_together(case["inputs"])
    actual = [_group(record) for record in results]
    expected = [
        {kind: _normalize(values) for kind, values in record.items()} for record in case["outputs"]
    ]
    assert actual == expected


def test_result_count_matches_input_count() -> None:
    assert anitomy.parse_together([]) == []
    assert len(anitomy.parse_together(["[G] Show - 01.mkv"])) == 1

    # Heterogeneous inputs — unrelated shows, an empty string, a path — are still
    # returned one record each, in order.
    mixed = ["[A] Alpha - 01.mkv", "", "C:\\x\\[G] Gamma - 03.mkv"]
    assert len(anitomy.parse_together(mixed)) == len(mixed)
