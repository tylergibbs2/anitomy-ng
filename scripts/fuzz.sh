#!/usr/bin/env bash
# Run a fuzz target, seeding its corpus from the conformance fixtures so the
# fuzzer starts from real filenames instead of rediscovering their shape.
# Requires nightly + cargo-fuzz: rustup toolchain install nightly && cargo install cargo-fuzz
#
#   scripts/fuzz.sh                      # parse, 60s
#   scripts/fuzz.sh parse_together 300   # named target, 300s
set -euo pipefail

TARGET="${1:-parse}"
SECS="${2:-60}"
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
CORPUS="$ROOT/anitomy/fuzz/corpus/$TARGET"

mkdir -p "$CORPUS"
python3 - "$ROOT" "$CORPUS" "$TARGET" <<'PY'
import hashlib, json, pathlib, sys

root, corpus, target = pathlib.Path(sys.argv[1]), pathlib.Path(sys.argv[2]), sys.argv[3]
# Single-filename suites use "input"; together.json uses "inputs" (a list).
inputs = sorted({
    s
    for f in (root / "anitomy/tests/fixtures").glob("*.json")
    for c in json.loads(f.read_text(encoding="utf-8"))
    for s in ([c["input"]] if "input" in c else c.get("inputs", []))
})

written = 0
for s in inputs:
    body = s.encode()
    # Both targets take a struct, and Arbitrary reads trailing bytes first:
    # parse is (u16, &str) -> 2 length bytes at the end; parse_together is
    # Vec<&str> -> one element consuming the rest. Prefixing keeps the seed
    # decodable as a filename either way.
    blob = body + b"\x00\x00" if target == "parse" else body
    p = corpus / hashlib.sha256(blob).hexdigest()[:16]
    if not p.exists():
        p.write_bytes(blob)
        written += 1
print(f"seeded {written} new / {len(inputs)} fixture inputs -> {corpus}")
PY

cd "$ROOT/anitomy"
exec cargo +nightly fuzz run "$TARGET" -- -max_total_time="$SECS"
