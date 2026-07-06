# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.

"""Refresh the conformance corpus and known-failures manifests in one step.

The external suites track living upstream projects, so their fixtures and the
resulting scores drift over time. This automates the mechanical loop:

  1. (--fetch) re-download the vendored upstream fixtures that are copied
     verbatim (erengy/anitomy's develop + master `test/data.json`).
  2. regenerate anitomy/tests/fixtures/*.json via scripts/build_fixtures.py.
  3. re-bless anitomy/tests/known_failures/*.txt via
     `UPDATE_KNOWN_FAILURES=1 cargo test -p anitomy-ng --test conformance`.
  4. print the per-suite known-failure delta.

Then review the change (`git diff anitomy/tests/known_failures`) — it shows
exactly which cases moved and in which direction — and re-run the suites
(`cargo test -p anitomy-ng` and, from anitomy-py/, `uv run --extra test pytest`).

anitopy's vendored tables (third_party/anitopy/{table,failing_table}.py) are
curated extracts, not a verbatim upstream file, so --fetch does not touch them;
update them by hand when anitopy's own test data changes.

Run with: uv run scripts/refresh_conformance.py [--fetch]
"""

from __future__ import annotations

import argparse
import os
import subprocess
import sys
import urllib.request
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
THIRD_PARTY = ROOT / "third_party"
MANIFEST_DIR = ROOT / "anitomy" / "tests" / "known_failures"
SUITES = ("anitomy_develop", "anitomy_master", "anitopy")

# Verbatim upstream fixtures that can be re-fetched (see third_party/README.md).
FETCH_SOURCES = {
    THIRD_PARTY / "anitomy-develop" / "test_data.json": (
        "https://raw.githubusercontent.com/erengy/anitomy/develop/test/data.json"
    ),
    THIRD_PARTY / "anitomy-master" / "test_data.json": (
        "https://raw.githubusercontent.com/erengy/anitomy/master/test/data.json"
    ),
}


def _manifest_counts() -> dict[str, int]:
    counts = {}
    for suite in SUITES:
        path = MANIFEST_DIR / f"{suite}.txt"
        if path.exists():
            counts[suite] = sum(
                1 for line in path.read_text(encoding="utf-8").splitlines() if line.strip()
            )
        else:
            counts[suite] = 0
    return counts


def _fetch() -> None:
    for dest, url in FETCH_SOURCES.items():
        print(f"fetching {url}")
        with urllib.request.urlopen(url) as resp:  # noqa: S310 (trusted upstream URLs)
            data = resp.read()
        dest.write_bytes(data)
        print(f"  -> {dest.relative_to(ROOT)} ({len(data)} bytes)")


def _run(cmd: list[str], **kwargs) -> subprocess.CompletedProcess:
    print(f"$ {' '.join(cmd)}")
    return subprocess.run(cmd, cwd=ROOT, check=True, **kwargs)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--fetch",
        action="store_true",
        help="re-download the verbatim upstream fixtures before regenerating",
    )
    args = parser.parse_args()

    if args.fetch:
        _fetch()

    before = _manifest_counts()

    _run([sys.executable, str(ROOT / "scripts" / "build_fixtures.py")])
    _run(
        ["cargo", "test", "-p", "anitomy-ng", "--test", "conformance", "--", "--nocapture"],
        env={**os.environ, "UPDATE_KNOWN_FAILURES": "1"},
    )

    after = _manifest_counts()

    print("\nknown-failure delta (lower is better):")
    for suite in SUITES:
        b, a = before[suite], after[suite]
        arrow = "=" if a == b else ("improved" if a < b else "regressed")
        print(f"  {suite:<16} {b:>4} -> {a:<4} ({arrow})")
    print(
        "\nReview `git diff anitomy/tests/known_failures` for the case-level "
        "changes, then re-run the suites to confirm they are green."
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
