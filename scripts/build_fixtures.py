# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.
"""Build independent conformance fixture suites from vendored upstream test
data. Unlike the old merge_fixtures.py, suites are NOT unioned or
deduplicated against each other -- each is tested independently, so the same
input filename may legitimately appear in more than one suite.

Run with: uv run scripts/build_fixtures.py

Reads:
  third_party/anitomy-develop/test_data.json  (current ElementKind schema)
  third_party/anitomy-master/test_data.json   (old schema)
  third_party/anitopy/table.py                (old schema, anitopy-passing)
  third_party/anitopy/failing_table.py        (old schema, anitopy-buggy but
                                                still valid anitomy ground truth)
Writes:
  anitomy/tests/fixtures/anitomy_develop.json
  anitomy/tests/fixtures/anitomy_master.json
  anitomy/tests/fixtures/anitopy.json

Does NOT touch anitomy/tests/fixtures/self_rolled.json -- that suite is
hand-maintained, not generated.

Nothing is silently dropped. A case whose only obstacle is an
`Options` field the current API doesn't support (`allowed_delimiters`,
`ignored_strings`) is still written out, tagged with a `"skip"` reason, so
the test harnesses can report it as an explicit skip rather than making it
vanish. An output key with no current `ElementKind` equivalent raises
instead of being silently ignored, unless it's specifically handled below
(`episode_alt` / `episode_number_alt`, folded into `episode`).
"""

import ast
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
THIRD_PARTY = ROOT / "third_party"
FIXTURES_DIR = ROOT / "anitomy" / "tests" / "fixtures"

# Old ElementCategory / anitopy category name -> current ElementKind (snake_case).
# `None` means the category has no current-API equivalent and is dropped from
# an entry's *output* (the case itself is always kept). `episode_number_alt`
# merges into `episode` (current anitomy represents a repeated element as
# multiple entries of the same kind, so an alt episode number is just a
# second `episode` value).
OLD_KEY_MAP = {
    "anime_season": "season",
    "anime_season_prefix": None,
    "anime_title": "title",
    "anime_type": "type",
    "anime_year": "year",
    "audio_term": "audio_term",
    "device_compatibility": "device",
    "episode_number": "episode",
    "episode_number_alt": "episode",
    "episode_prefix": None,
    "episode_title": "episode_title",
    "file_checksum": "file_checksum",
    "file_extension": "file_extension",
    "file_name": None,  # redundant echo of the input, not a parsed element
    "language": "language",
    "other": "other",
    "release_group": "release_group",
    "release_information": "release_information",
    "release_version": "release_version",
    "source": "source",
    "subtitles": "subtitles",
    "video_resolution": "video_resolution",
    "video_term": "video_term",
    "volume_number": "volume",
    "volume_prefix": None,
    "id": None,  # carried separately as `mal_id` metadata, not an element
}

# Current ElementKind names (include/anitomy/detail/format.hpp's to_element_kind table).
CURRENT_ELEMENT_KINDS = {
    "audio_term",
    "device",
    "episode",
    "episode_title",
    "file_checksum",
    "file_extension",
    "language",
    "other",
    "part",
    "release_group",
    "release_information",
    "release_version",
    "season",
    "source",
    "subtitles",
    "title",
    "type",
    "video_resolution",
    "video_term",
    "volume",
    "year",
}

UNSUPPORTED_OPTION_KEYS = {"option_allowed_delimiters", "option_ignored_strings"}


def merge_values(new_output: dict[str, list[str]], key: str, value) -> None:
    values = value if isinstance(value, list) else [value]
    new_output.setdefault(key, []).extend(str(v) for v in values)


def finalize_output(new_output: dict[str, list[str]]) -> dict[str, str | list[str]]:
    # Collapse single-element lists to a bare string, matching upstream's
    # own test_data.json convention (see third_party/README.md).
    return {k: v[0] if len(v) == 1 else v for k, v in new_output.items()}


def convert_old_schema(old_output: dict) -> dict:
    """Map an old-schema (anitomy-master / anitopy) output dict to current schema."""
    new_output: dict[str, list[str]] = {}
    for old_key, value in old_output.items():
        new_key = OLD_KEY_MAP.get(old_key, "__unknown__")
        if new_key == "__unknown__":
            raise KeyError(f"unmapped old-schema element category: {old_key!r}")
        if new_key is None:
            continue
        merge_values(new_output, new_key, value)
    return finalize_output(new_output)


def skip_reason(used_options: set[str]) -> str:
    names = sorted(k.removeprefix("option_") for k in used_options)
    return f"uses unsupported option(s): {', '.join(names)}"


def load_anitomy_develop() -> list[dict]:
    data = json.loads(
        (THIRD_PARTY / "anitomy-develop" / "test_data.json").read_text(encoding="utf-8")
    )
    cases = []
    for entry in data:
        new_output: dict[str, list[str]] = {}
        for key, value in entry["output"].items():
            if key == "episode_alt":
                merge_values(new_output, "episode", value)
            elif key in CURRENT_ELEMENT_KINDS:
                merge_values(new_output, key, value)
            else:
                raise KeyError(
                    f"unexpected key in anitomy-develop test_data.json: {key!r}"
                )
        case = {
            "input": entry["input"],
            "source": "anitomy-develop",
            "output": finalize_output(new_output),
        }
        if "mal_id" in entry:
            case["mal_id"] = entry["mal_id"]
        cases.append(case)
    return cases


# Some third_party/anitomy-master/test_data.json entries omit "file_extension"
# even though the filename clearly ends in one of these -- a data-entry gap in
# the vendored file (master's own compiled binary extracts the extension too).
# Filled in from the filename rather than by editing the vendored JSON, so the
# vendored copy stays verbatim.
_KNOWN_EXTENSIONS = {
    "3gp",
    "avi",
    "divx",
    "flv",
    "m2ts",
    "m4v",
    "mkv",
    "mov",
    "mp4",
    "mpg",
    "ogm",
    "rm",
    "rmvb",
    "ts",
    "webm",
    "wmv",
}

# Pruned: these anitomy-master/test_data.json entries expect a bare season
# marker (e.g. "S2") to stay fused into the title rather than split into a
# separate `season` element. Three sources split it for these exact inputs:
# upstream's compiled `develop` binary, Rapptz/anitomy-rs, and anitopy's own
# fixture (for the one case they share). master predates the rewrite that
# added this split, so treat its expectation as outdated here. Not pruned
# from anitopy or anitomy-develop, whose fixtures already agree with the split.
_PRUNED_OUTDATED = {
    "Ookiku Furikabutte S2 - 09 (Central Anime) [BD841253].mkv": "season fused into title; develop/Rapptz/anitopy all split it",
    "[Conclave-Mendoi]_Mobile_Suit_Gundam_00_S2_-_01v2_[1280x720_H.264_AAC][4863FBE8].mkv": "season fused into title; develop/Rapptz/anitopy all split it",
    "[Hatsuyuki]_Kuroko_no_Basuke_S3_-_01_(51)_[720p][10bit][619C57A0].mkv": "season fused into title; develop/Rapptz/anitopy all split it",
    "[SFW]_Queen's_Blade_S2": "season fused into title; develop/Rapptz/anitopy all split it",
}


def load_anitomy_master() -> list[dict]:
    data = json.loads(
        (THIRD_PARTY / "anitomy-master" / "test_data.json").read_text(encoding="utf-8")
    )
    cases = []
    pruned = 0
    for entry in data:
        if entry["file_name"] in _PRUNED_OUTDATED:
            pruned += 1
            continue
        used_options = UNSUPPORTED_OPTION_KEYS & entry.keys()
        old_output = {
            k: v
            for k, v in entry.items()
            if k not in ("file_name", "id") and k not in UNSUPPORTED_OPTION_KEYS
        }
        if "file_extension" not in old_output:
            ext = (
                entry["file_name"].rsplit(".", 1)[-1].lower()
                if "." in entry["file_name"]
                else ""
            )
            if ext in _KNOWN_EXTENSIONS:
                old_output["file_extension"] = ext
        case = {
            "input": entry["file_name"],
            "source": "anitomy-master",
            "output": convert_old_schema(old_output),
        }
        if "id" in entry:
            case["mal_id"] = entry["id"]
        if used_options:
            case["skip"] = skip_reason(used_options)
        cases.append(case)
    if pruned:
        print(f"anitomy_master: pruned {pruned} outdated case(s), see _PRUNED_OUTDATED")
    return cases


def load_anitopy_table(path: Path, source: str) -> list[dict]:
    module = ast.parse(path.read_text(encoding="utf-8"))
    table_name = "failing_table" if source == "anitopy-failing" else "table"
    table_literal = next(
        node.value
        for node in ast.walk(module)
        if isinstance(node, ast.Assign)
        and any(isinstance(t, ast.Name) and t.id == table_name for t in node.targets)
    )
    table = ast.literal_eval(table_literal)

    cases = []
    for filename, options, old_output in table:
        used_options = UNSUPPORTED_OPTION_KEYS & options.keys() if options else set()
        case = {
            "input": filename,
            "source": source,
            "output": convert_old_schema(
                {k: v for k, v in old_output.items() if k != "id"}
            ),
        }
        if "id" in old_output:
            case["mal_id"] = old_output["id"]
        if used_options:
            case["skip"] = skip_reason(used_options)
        cases.append(case)
    return cases


def write_suite(name: str, cases: list[dict]) -> None:
    cases = sorted(cases, key=lambda c: c["input"])
    out_path = FIXTURES_DIR / f"{name}.json"
    out_path.write_text(
        json.dumps(cases, indent=2, ensure_ascii=False) + "\n", encoding="utf-8"
    )
    skipped = sum(1 for c in cases if "skip" in c)
    print(
        f"{name}: {len(cases)} cases ({skipped} skipped) -> {out_path.relative_to(ROOT)}"
    )


def main() -> None:
    FIXTURES_DIR.mkdir(parents=True, exist_ok=True)

    write_suite("anitomy_develop", load_anitomy_develop())
    write_suite("anitomy_master", load_anitomy_master())

    anitopy_cases = load_anitopy_table(THIRD_PARTY / "anitopy" / "table.py", "anitopy")
    anitopy_failing_cases = load_anitopy_table(
        THIRD_PARTY / "anitopy" / "failing_table.py", "anitopy-failing"
    )
    write_suite("anitopy", anitopy_cases + anitopy_failing_cases)

    self_rolled_path = FIXTURES_DIR / "self_rolled.json"
    if not self_rolled_path.exists():
        self_rolled_path.write_text("[]\n", encoding="utf-8")
        print(
            f"self_rolled: 0 cases (seeded empty, hand-maintained) -> {self_rolled_path.relative_to(ROOT)}"
        )
    else:
        existing = json.loads(self_rolled_path.read_text(encoding="utf-8"))
        print(
            f"self_rolled: {len(existing)} cases (hand-maintained, not regenerated) -> "
            f"{self_rolled_path.relative_to(ROOT)}"
        )


if __name__ == "__main__":
    main()
