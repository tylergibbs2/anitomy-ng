// Benchmark adapter for yjl9903/anitomy (npm `anitomy`). Reads the corpus from
// stdin (one filename per line) and emits JSONL — one `{input, output}` object
// per line — where `output` is already flattened to current ElementKind names,
// so the benchmark harness treats it as `schema: "current"`.
//
// After the results, it emits a final `{"__per_file_ns__": N}` line: the median
// per-file parse time (parse only, process startup and I/O excluded), measured
// by re-parsing the whole corpus many times in this same Node process.
import { parse } from "anitomy";
import process from "node:process";

function flatten(r) {
  const out = {};
  const push = (k, v) => {
    if (v !== undefined && v !== null && v !== "") (out[k] ??= []).push(String(v));
  };
  push("title", r.title);
  push("type", r.type);
  push("season", r.season);
  push("year", r.year);
  push("language", r.language);
  push("subtitles", r.subtitles);
  push("source", r.source);
  push("episode", r.episode?.number);
  push("episode", r.episode?.numberAlt);
  push("episode_title", r.episode?.title);
  push("volume", r.volume?.number);
  push("video_term", r.video?.term);
  push("video_resolution", r.video?.resolution);
  push("audio_term", r.audio?.term);
  push("release_version", r.release?.version);
  push("release_group", r.release?.group);
  push("file_checksum", r.file?.checksum);
  push("file_extension", r.file?.extension);
  return out;
}

let data = "";
process.stdin.setEncoding("utf8");
for await (const chunk of process.stdin) data += chunk;
const inputs = data.split("\n").filter((l) => l.length > 0);

const lines = inputs.map((inp) => {
  let out = {};
  try {
    out = flatten(parse(inp) || {});
  } catch {
    out = {};
  }
  return JSON.stringify({ input: inp, output: out });
});

// Timing: warm up, then time repeated full-corpus passes and take the median
// per-file time. A fresh `parse()` call per input avoids any result cache.
const TIMED_PASSES = 200;
for (let w = 0; w < 5; w++) for (const inp of inputs) parse(inp);
const passNs = [];
for (let p = 0; p < TIMED_PASSES; p++) {
  const t0 = process.hrtime.bigint();
  for (const inp of inputs) parse(inp);
  passNs.push(Number(process.hrtime.bigint() - t0));
}
passNs.sort((a, b) => a - b);
const medianPassNs = passNs[passNs.length >> 1];
const perFileNs = medianPassNs / inputs.length;
lines.push(JSON.stringify({ __per_file_ns__: perFileNs }));

process.stdout.write(lines.join("\n") + "\n");
