// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

// Speed/conformance adapter for anitomy-ng's *wasm* build (the JS cohort form of
// the library), so the JS-runtime speed table compares like-for-like against
// yjl9903. Reads the corpus from stdin (one filename per line), emits JSONL
// `{input, output}` (schema=current), then a final `{"__per_file_ns__": N}`
// with the median per-file parse time in this Node process.
//
// Requires the Node/CJS wasm build at dist-npm/node (scripts/build-npm.sh).
const path = require("node:path");
const pkgDir =
  process.argv[2] || path.join(__dirname, "..", "..", "..", "dist-npm", "node");
const { parse } = require(path.join(pkgDir, "anitomy_ng.js"));

function flatten(elements) {
  const out = {};
  for (const el of elements) (out[el.kind] ??= []).push(el.value);
  return out;
}

let data = "";
process.stdin.setEncoding("utf8");
process.stdin.on("data", (c) => (data += c));
process.stdin.on("end", () => {
  const inputs = data.split("\n").filter((l) => l.length > 0);
  const lines = inputs.map((inp) => {
    let out = {};
    try {
      out = flatten(parse(inp));
    } catch {
      out = {};
    }
    return JSON.stringify({ input: inp, output: out });
  });

  const TIMED_PASSES = 200;
  for (let w = 0; w < 5; w++) for (const inp of inputs) parse(inp);
  const passNs = [];
  for (let p = 0; p < TIMED_PASSES; p++) {
    const t0 = process.hrtime.bigint();
    for (const inp of inputs) parse(inp);
    passNs.push(Number(process.hrtime.bigint() - t0));
  }
  passNs.sort((a, b) => a - b);
  const perFileNs = passNs[passNs.length >> 1] / inputs.length;
  lines.push(JSON.stringify({ __per_file_ns__: perFileNs }));

  process.stdout.write(lines.join("\n") + "\n");
});
