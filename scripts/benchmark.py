# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.

"""Cross-implementation conformance benchmark.

Scores anitomy-ng and any available reference parsers against the SAME corpus
(anitomy/tests/fixtures/*.json), so the numbers are apples-to-apples rather
than each project graded on its own bundled data. Emits a Markdown table.

Two kinds of engine:
- Python engines, called in-process: `anitomy_ng` (required) and, if installed,
  `anitopy`.
- External engines, described by --engines-config JSON as
  `[{"name": "...", "label": "...", "schema": "current"|"old", "cmd": ["..."]}]`.
  Each command is run once; the benchmark writes every corpus input to its stdin
  (one per line, UTF-8) and reads back JSONL, one object per line:
  `{"input": "<the input>", "output": {"<kind>": ["<value>", ...], ...}}`.
  `schema` says whether the emitted keys are current `ElementKind` names or the
  old anitomy/anitopy category names (converted here via build_fixtures).
  `label` (optional) is the column header shown in the table; defaults to `name`.

Engines that aren't available (module not importable, command missing/failing)
are reported as skipped rather than silently omitted.

Run with: uv run --with anitopy scripts/benchmark.py [--engines-config cfg.json]
"""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
FIXTURES_DIR = ROOT / "anitomy" / "tests" / "fixtures"
SUITES = ("anitomy_develop", "anitomy_master", "anitopy", "self_rolled")

sys.path.insert(0, str(ROOT / "scripts"))
from build_fixtures import OLD_KEY_MAP  # noqa: E402  (path set up just above)

Grouped = dict[str, list[str]]


def _normalize(value: object) -> list[str]:
    return [str(v) for v in value] if isinstance(value, list) else [str(value)]


def _to_current(raw: dict, schema: str) -> Grouped:
    """Normalize one engine's raw output to current-ElementKind, list-valued form."""
    out: Grouped = {}
    for key, value in raw.items():
        new_key = key if schema == "current" else OLD_KEY_MAP.get(key, None)
        if new_key is None:  # no current-API equivalent, or unmapped: drop
            continue
        out.setdefault(new_key, []).extend(_normalize(value))
    return out


def load_corpus() -> dict[str, list[dict]]:
    """Non-skipped cases per suite: {input, expected(current-schema grouped)}."""
    corpus: dict[str, list[dict]] = {}
    for suite in SUITES:
        path = FIXTURES_DIR / f"{suite}.json"
        cases = json.loads(path.read_text(encoding="utf-8"))
        corpus[suite] = [
            {"input": c["input"], "expected": {k: _normalize(v) for k, v in c["output"].items()}}
            for c in cases
            if "skip" not in c
        ]
    return corpus


def all_inputs(corpus: dict[str, list[dict]]) -> list[str]:
    seen: dict[str, None] = {}  # dict preserves order, dedupes across suites
    for cases in corpus.values():
        for case in cases:
            seen.setdefault(case["input"], None)
    return list(seen)


def _safe_parse(parse: object, inp: str, schema: str) -> Grouped:
    """Parse one input, treating any crash as empty output (a miss, not an abort).

    Reference parsers can raise on adversarial filenames — anitomy-ng is
    panic-free by contract, but others aren't — so robustness itself shows up as
    a lower score rather than a broken run.
    """
    try:
        raw = parse(inp) or {}
    except Exception:  # noqa: BLE001 — a crashing parse is a failed case, not our error
        return {}
    if not isinstance(raw, dict):  # anitomy_ng returns elements; handled by caller
        return raw
    return _to_current(raw, schema)


def python_engine(name: str) -> dict[str, Grouped] | None:
    """Run an in-process Python parser over the corpus, or None if unavailable."""
    inputs = all_inputs(load_corpus())
    if name == "anitomy_ng":
        import anitomy_ng

        results = {}
        for inp in inputs:
            grouped: Grouped = {}
            try:
                elements = anitomy_ng.parse(inp)
            except Exception:  # noqa: BLE001 — should never happen (panic-free), scored as miss
                elements = []
            for el in elements:
                grouped.setdefault(el.kind.value, []).append(el.value)
            results[inp] = grouped
        return results
    if name == "anitopy":
        try:
            import anitopy
        except ImportError:
            return None
        return {inp: _safe_parse(anitopy.parse, inp, "old") for inp in inputs}
    raise ValueError(f"unknown python engine {name!r}")


def external_engine(cmd: list[str], schema: str, inputs: list[str]) -> dict[str, Grouped] | None:
    """Run an external adapter over the corpus via stdin/stdout JSONL, or None on failure."""
    try:
        proc = subprocess.run(
            cmd,
            input="\n".join(inputs) + "\n",
            capture_output=True,
            text=True,
            encoding="utf-8",
            cwd=ROOT,
            timeout=600,
        )
    except (OSError, subprocess.TimeoutExpired) as e:
        print(f"  engine command failed: {e}", file=sys.stderr)
        return None
    if proc.returncode != 0:
        print(f"  engine exited {proc.returncode}: {proc.stderr[-500:]}", file=sys.stderr)
        return None
    results: dict[str, Grouped] = {}
    for line in proc.stdout.splitlines():
        line = line.strip()
        if not line:
            continue
        obj = json.loads(line)
        results[obj["input"]] = _to_current(obj["output"], schema)
    return results


def score(corpus: dict[str, list[dict]], results: dict[str, Grouped]) -> dict[str, tuple[int, int]]:
    per_suite: dict[str, tuple[int, int]] = {}
    for suite, cases in corpus.items():
        passed = sum(1 for c in cases if results.get(c["input"]) == c["expected"])
        per_suite[suite] = (passed, len(cases))
    return per_suite


def markdown_table(scores: dict[str, dict[str, tuple[int, int]]], corpus) -> str:
    engines = list(scores)
    lines = [
        "# Conformance benchmark",
        "",
        "Each parser scored against every suite's fixtures (its declared ground",
        "truth). Higher is better; the external suites contain cases no parser",
        "passes, so 100% is neither expected nor the goal.",
        "",
        "| Suite (cases) | " + " | ".join(engines) + " |",
        "|" + "---|" * (len(engines) + 1),
    ]
    for suite in SUITES:
        total = len(corpus[suite])
        cells = []
        for eng in engines:
            passed, tot = scores[eng].get(suite, (0, total))
            cells.append(f"{passed}/{tot} ({passed / tot * 100:.1f}%)" if tot else "—")
        lines.append(f"| {suite} ({total}) | " + " | ".join(cells) + " |")
    return "\n".join(lines) + "\n"


def main() -> int:
    parser = argparse.ArgumentParser(description="Cross-implementation conformance benchmark.")
    parser.add_argument("--engines-config", type=Path, help="JSON describing external engines")
    parser.add_argument("--out", type=Path, help="write the Markdown table here (also to stdout)")
    args = parser.parse_args()

    corpus = load_corpus()
    inputs = all_inputs(corpus)
    scores: dict[str, dict[str, tuple[int, int]]] = {}
    skipped: list[str] = []

    for name in ("anitomy_ng", "anitopy"):
        results = python_engine(name)
        if results is None:
            skipped.append(name)
        else:
            scores[name] = score(corpus, results)

    if args.engines_config:
        for cfg in json.loads(args.engines_config.read_text(encoding="utf-8")):
            print(f"running external engine: {cfg['name']}", file=sys.stderr)
            results = external_engine(cfg["cmd"], cfg.get("schema", "current"), inputs)
            # `label` is the display/column name; `name` stays the internal id.
            label = cfg.get("label", cfg["name"])
            if results is None:
                skipped.append(label)
            else:
                scores[label] = score(corpus, results)

    table = markdown_table(scores, corpus)
    if skipped:
        table += f"\n_Skipped (unavailable): {', '.join(skipped)}._\n"

    print(table)
    if args.out:
        args.out.write_text(table, encoding="utf-8")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
