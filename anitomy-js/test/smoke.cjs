// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

// Smoke test for the built npm package (Node/CommonJS target). Run after
// scripts/build-npm.sh:  node anitomy-js/test/smoke.cjs [dist-npm dir]
// Exits non-zero on any failure so CI catches a broken build.

const assert = require("node:assert");
const path = require("node:path");

const pkgDir = process.argv[2] || path.join(__dirname, "..", "..", "dist-npm");
const { parse } = require(path.join(pkgDir, "node", "anitomy_ng.js"));

const name =
  "[TaigaSubs] Toradora! (2008) - 01v2 [1280x720 H.264 FLAC][1234ABCD].mkv";
const elements = parse(name);

// Shape: array of { kind, value, position }.
assert(Array.isArray(elements) && elements.length > 0, "expected some elements");
for (const el of elements) {
  assert(typeof el.kind === "string", "kind should be a string");
  assert(typeof el.value === "string", "value should be a string");
  assert(typeof el.position === "number", "position should be a number");
}

const byKind = Object.fromEntries(elements.map((e) => [e.kind, e.value]));
assert.strictEqual(byKind.release_group, "TaigaSubs");
assert.strictEqual(byKind.title, "Toradora!");
assert.strictEqual(byKind.episode, "01");
assert.strictEqual(byKind.file_extension, "mkv");

// Options: disabling a kind drops it.
const noEpisode = parse("Show - 01.mkv", { parse_episode: false });
assert(
  !noEpisode.some((e) => e.kind === "episode"),
  "parse_episode:false should drop episode elements",
);

// Non-ASCII round-trips intact through the JS<->wasm UTF-8 boundary.
const unicode = parse("[グループ] 進撃の巨人 - 05 [1080p].mkv");
assert(
  unicode.some((e) => e.kind === "episode" && e.value === "05"),
  "unicode filename should still yield episode 05",
);

console.log(`smoke test passed (${elements.length} elements parsed)`);
