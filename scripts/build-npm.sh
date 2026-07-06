#!/usr/bin/env bash
# Build the npm package from anitomy-js: two wasm-pack targets (Node CommonJS +
# bundler ESM) merged under one package.json with conditional exports, so both
# `require(...)` (Node) and bundler `import` (Vite/webpack/...) resolve without
# the consumer thinking about wasm init. Output: dist-npm/.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
OUT="dist-npm"
rm -rf "$OUT"

wasm-pack build anitomy-js --release --target nodejs  --out-dir "../$OUT/node"    --out-name anitomy_ng
wasm-pack build anitomy-js --release --target bundler --out-dir "../$OUT/bundler" --out-name anitomy_ng

VERSION="$(node -p "require('./$OUT/node/package.json').version")"

# Drop the per-target scaffolding; the top-level package.json below is published.
rm -f "$OUT"/node/{package.json,README.md,LICENSE,.gitignore} \
      "$OUT"/bundler/{package.json,README.md,LICENSE,.gitignore}

cp README.md "$OUT/README.md"
cp anitomy-js/LICENSE "$OUT/LICENSE"

cat > "$OUT/package.json" <<JSON
{
  "name": "anitomy-ng",
  "version": "$VERSION",
  "description": "Anime video filename parser — WebAssembly build of the anitomy-ng Rust crate",
  "license": "MPL-2.0",
  "repository": {
    "type": "git",
    "url": "git+https://github.com/tylergibbs2/anitomy-ng.git"
  },
  "keywords": ["anime", "parser", "filename", "anitomy", "wasm"],
  "engines": { "node": ">=18" },
  "types": "./bundler/anitomy_ng.d.ts",
  "main": "./node/anitomy_ng.js",
  "module": "./bundler/anitomy_ng.js",
  "exports": {
    ".": {
      "types": "./bundler/anitomy_ng.d.ts",
      "node": "./node/anitomy_ng.js",
      "import": "./bundler/anitomy_ng.js",
      "default": "./bundler/anitomy_ng.js"
    }
  },
  "files": ["node", "bundler", "README.md", "LICENSE"]
}
JSON

echo "Built $OUT (anitomy-ng@$VERSION)"
