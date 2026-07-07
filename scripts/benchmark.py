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
  `anitopy` and `aniparse` (github.com/MeGaNeKoS/Aniparse).
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
SUITES = ("anitomy_develop", "anitomy_master", "anitopy", "anitomy_ng")

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


def _from_aniparse(raw: dict) -> Grouped:
    """Flatten aniparse's nested schema to current-ElementKind grouped form.

    aniparse (github.com/MeGaNeKoS/Aniparse) has its own structured output:
    `series: [{title, type, year:[{number}], season:[{number}],
    episode:[{number} | {start:{number}, end:{number}}]}]` plus flat term
    lists. Numbers come back as ints (so `4`, not `04`) and `type` is
    lowercased — its own conventions, preserved here rather than normalized, so
    the score reflects what the library actually emits.
    """
    out: Grouped = {}

    def add(key: str, values: object) -> None:
        if not values:
            return
        seq = values if isinstance(values, list) else [values]
        out.setdefault(key, []).extend(str(v) for v in seq if v is not None)

    add("release_group", raw.get("release_group"))
    add("release_information", raw.get("release_information"))
    add("source", raw.get("source"))
    add("audio_term", raw.get("audio_term"))
    add("video_term", raw.get("video_term"))
    add("subtitles", raw.get("subs_term"))
    add("language", raw.get("language_term"))
    add("file_checksum", raw.get("file_checksum"))
    add("file_extension", raw.get("file_extension"))
    for vr in raw.get("video_resolution") or []:
        if isinstance(vr, dict):
            width, height, scan = vr.get("video_width"), vr.get("video_height"), vr.get("scan_method")
            if width and height:
                add("video_resolution", f"{width}x{height}")
            elif height:
                add("video_resolution", f"{height}{scan or ''}")
        else:
            add("video_resolution", vr)
    for series in raw.get("series") or []:
        if not isinstance(series, dict):
            continue
        add("title", series.get("title"))
        add("type", series.get("type"))
        for year in series.get("year") or []:
            add("year", year.get("number"))
        for season in series.get("season") or []:
            add("season", season.get("number"))
        for ep in series.get("episode") or []:
            if "number" in ep:
                add("episode", ep.get("number"))
            else:  # a range: {start: {number}, end: {number}}
                add("episode", (ep.get("start") or {}).get("number"))
                add("episode", (ep.get("end") or {}).get("number"))
    return out


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
    if name == "aniparse":
        try:
            import aniparse
        except ImportError:
            return None
        results = {}
        for inp in inputs:
            try:
                raw = aniparse.parse(inp) or {}
            except Exception:  # noqa: BLE001 — a crashing parse is a failed case, not our error
                raw = {}
            results[inp] = _from_aniparse(raw) if isinstance(raw, dict) else {}
        return results
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


def _canonical_value(value: str) -> str:
    """Fold away representation differences so scoring compares parse content,
    not formatting: integers lose leading zeros (`01` == `1`) and everything
    else case-folds (`movie` == `Movie`, `10Bit` == `10bit`). Applied to both
    sides for every engine, so it never lowers a score — it only stops a
    library's value conventions (e.g. aniparse's integer episode numbers) from
    masking a correct parse. Genuine differences (`1` vs `10`, `special` vs
    `specials`) still compare unequal."""
    value = value.strip()
    return str(int(value)) if value.isdigit() else value.casefold()


def _canonical(grouped: Grouped) -> dict[str, list[str]]:
    return {k: [_canonical_value(v) for v in vs] for k, vs in grouped.items()}


def score(corpus: dict[str, list[dict]], results: dict[str, Grouped]) -> dict[str, tuple[int, int]]:
    per_suite: dict[str, tuple[int, int]] = {}
    for suite, cases in corpus.items():
        passed = sum(
            1
            for c in cases
            if _canonical(results.get(c["input"], {})) == _canonical(c["expected"])
        )
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
        "Values are compared representation-agnostically (integers lose leading",
        "zeros, everything else case-folds), uniformly for every engine, so a",
        "library's value conventions — e.g. aniparse's integer episode numbers",
        "(`1` vs `01`) or lowercased `type` — don't distort the comparison.",
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

    for name in ("anitomy_ng", "anitopy", "aniparse"):
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
