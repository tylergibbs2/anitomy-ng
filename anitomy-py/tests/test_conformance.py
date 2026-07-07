# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.

"""Same conformance fixture suites as anitomy/tests/conformance.rs, run against
the Python bindings (see that file, scripts/build_fixtures.py, and
third_party/README.md).

Four independent suites, each its own parametrized test so pass/fail reads per
suite. The policies match the Rust harness:
- anitomy_ng is this project's own hand-curated ground-truth suite; like
  the others it is checked against the shared known-failures manifest.
- anitomy_develop / anitomy_master / anitopy are external suites with imperfect
  ground truth, checked against the shared known-failures manifests in
  anitomy/tests/known_failures/. Cases listed there are marked xfail(strict), so
  a newly-failing case is a regression and a newly-passing case (an unexpected
  pass) fails the suite until removed from the manifest. Rust is the source of
  truth for those files; regenerate with
  `UPDATE_KNOWN_FAILURES=1 cargo test -p anitomy-ng --test conformance`.

Cases marked `"skip"` in their fixture (they exercise an unsupported `Options`
field) show up as explicit pytest skips.

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


def _known_failures(name: str) -> set[str]:
    path = KNOWN_FAILURES_DIR / f"{name}.txt"
    if not path.exists():
        return set()
    return {line for line in path.read_text(encoding="utf-8").splitlines() if line.strip()}


def _load_suite(name: str) -> list:
    cases = json.loads((FIXTURES_DIR / f"{name}.json").read_text(encoding="utf-8"))
    known = _known_failures(name)
    params = []
    for case in cases:
        if "skip" in case:
            marks = [pytest.mark.skip(reason=case["skip"])]
        elif case["input"] in known:
            marks = [pytest.mark.xfail(reason="known conformance gap", strict=True)]
        else:
            marks = []
        params.append(pytest.param(case, id=case["input"], marks=marks))
    return params


def _run_case(case: dict) -> None:
    actual = _group(anitomy.parse(case["input"]))
    expected = {k: _normalize(v) for k, v in case["output"].items()}
    assert actual == expected


@pytest.mark.parametrize("case", _load_suite("anitomy_develop"))
def test_conformance_anitomy_develop(case: dict) -> None:
    _run_case(case)


@pytest.mark.parametrize("case", _load_suite("anitomy_master"))
def test_conformance_anitomy_master(case: dict) -> None:
    _run_case(case)


@pytest.mark.parametrize("case", _load_suite("anitopy"))
def test_conformance_anitopy(case: dict) -> None:
    _run_case(case)


@pytest.mark.parametrize("case", _load_suite("anitomy_ng"))
def test_conformance_anitomy_ng(case: dict) -> None:
    _run_case(case)
