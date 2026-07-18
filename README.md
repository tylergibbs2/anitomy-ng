# anitomy-ng

[![CI](https://github.com/tylergibbs2/anitomy-ng/actions/workflows/ci.yml/badge.svg)](https://github.com/tylergibbs2/anitomy-ng/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/anitomy-ng.svg)](https://crates.io/crates/anitomy-ng)
[![docs.rs](https://img.shields.io/docsrs/anitomy-ng.svg)](https://docs.rs/anitomy-ng)
[![PyPI](https://img.shields.io/pypi/v/anitomy-ng.svg)](https://pypi.org/project/anitomy-ng/)
[![npm](https://img.shields.io/npm/v/anitomy-ng.svg)](https://www.npmjs.com/package/anitomy-ng)
[![NuGet](https://img.shields.io/nuget/v/AnitomyNg.svg)](https://www.nuget.org/packages/AnitomyNg)
[![License: MPL 2.0](https://img.shields.io/badge/License-MPL_2.0-brightgreen.svg)](https://www.mozilla.org/en-US/MPL/2.0/)

A pure-Rust port of [erengy/anitomy](https://github.com/erengy/anitomy), an
anime video filename parser, with Python, JavaScript, and .NET bindings. The
core library is pure safe Rust — no `unsafe`, no C dependencies.

```
[TaigaSubs]_Toradora!_(2008)_-_01v2_-_Tiger_and_Dragon_[1280x720_H.264_FLAC][1234ABCD].mkv
```
parses into release group, title, year, episode, resolution, video/audio
codec, release version, and file checksum — see the examples below.

**Status**: conformance-tested against upstream's own bundled test data (the
current C++ rewrite on the `develop` branch and the original, long-frozen
`master` implementation), plus the
[anitopy](https://github.com/igorcmoura/anitopy) Python port's fixtures. On
each suite it scores at least as high as that suite's reference parser, run
as a compiled/installed binary rather than judged from its source.

## Install

Rust:

```sh
cargo add anitomy-ng
```

Python (wheels built via [maturin](https://www.maturin.rs/)):

```sh
pip install anitomy-ng
```

JavaScript / TypeScript (WebAssembly, works in Node and bundlers):

```sh
npm install anitomy-ng
```

Command line — prebuilt binaries for Linux, macOS, and Windows are attached to
each [GitHub release](https://github.com/tylergibbs2/anitomy-ng/releases).
Download one directly, or install with either:

```sh
cargo binstall anitomy-ng               # prebuilt binary, no toolchain needed
cargo install anitomy-ng --features cli # builds from source
```

.NET (prebuilt native binaries ship in the package — no Rust toolchain needed):

```sh
dotnet add package AnitomyNg
```

## Usage

Rust:

```rust
let elements = anitomy_ng::parse(
    "[TaigaSubs]_Toradora!_(2008)_-_01v2_-_Tiger_and_Dragon_[1280x720_H.264_FLAC][1234ABCD].mkv",
    anitomy_ng::Options::default(),
);
for element in &elements {
    println!("{:?}: {}", element.kind, element.value);
}
```

Python:

```python
import anitomy_ng

for element in anitomy_ng.parse(
    "[TaigaSubs]_Toradora!_(2008)_-_01v2_-_Tiger_and_Dragon_[1280x720_H.264_FLAC][1234ABCD].mkv"
):
    print(element.kind, element.value)
```

JavaScript / TypeScript:

```ts
import { parse } from "anitomy-ng";

for (const element of parse(
  "[TaigaSubs]_Toradora!_(2008)_-_01v2_-_Tiger_and_Dragon_[1280x720_H.264_FLAC][1234ABCD].mkv",
)) {
  console.log(element.kind, element.value);
}
```

C# / .NET:

```csharp
using AnitomyNg;

foreach (var element in Anitomy.Parse(
    "[TaigaSubs]_Toradora!_(2008)_-_01v2_-_Tiger_and_Dragon_[1280x720_H.264_FLAC][1234ABCD].mkv"))
{
    Console.WriteLine($"{element.Kind}: {element.Value}");
}
```

They all return an ordered list of elements (position in the filename, kind,
and value); `ElementKind`/`kind` covers title, episode, season, release group,
video/audio terms, resolution, checksum, and so on — see
[`anitomy/src/element.rs`](anitomy/src/element.rs) for the full set.

### Parsing a set together

`parse_together` parses a *set* of related filenames at once, returning one
element list per input (result `i` is for input `i`). Using the whole set as
shared context resolves things a single filename can't — most importantly, a
directory batch range like `(01-12)` no longer masks the real per-file episode,
and a series title that lives only in a parent folder is recovered. Unrelated or
single-item lists are a safe no-op (each is just its ordinary parse). Full and
relative paths work on every platform (`/` and `\`, including UNC/drive-letter).

Parsing each of these on its own would read `01` and `12` from the folder's
`(01-12)` range and miss the actual episode; together, they come back as `05`
and `06`:

Rust:

```rust
let results = anitomy_ng::parse_together(
    &[
        "Frieren (01-12) [Batch]/Frieren - 05 [1080p].mkv",
        "Frieren (01-12) [Batch]/Frieren - 06 [1080p].mkv",
    ],
    anitomy_ng::Options::default(),
);
// results[0] -> episode "05", results[1] -> episode "06"
```

Python:

```python
results = anitomy_ng.parse_together([
    "Frieren (01-12) [Batch]/Frieren - 05 [1080p].mkv",
    "Frieren (01-12) [Batch]/Frieren - 06 [1080p].mkv",
])
# results[0] -> episode "05", results[1] -> episode "06"
```

JavaScript / TypeScript:

```ts
import { parse_together } from "anitomy-ng";

const results = parse_together([
  "Frieren (01-12) [Batch]/Frieren - 05 [1080p].mkv",
  "Frieren (01-12) [Batch]/Frieren - 06 [1080p].mkv",
]);
// results[0] -> episode "05", results[1] -> episode "06"
```

C# / .NET:

```csharp
var results = Anitomy.ParseTogether(new[]
{
    "Frieren (01-12) [Batch]/Frieren - 05 [1080p].mkv",
    "Frieren (01-12) [Batch]/Frieren - 06 [1080p].mkv",
});
// results[0] -> episode "05", results[1] -> episode "06"
```

Command line (`anitomy`) — takes filenames as arguments or reads them from
stdin (one per line), and prints an aligned table or, with `--json`, an array
of `{ filename, elements }`:

```sh
anitomy '[TaigaSubs]_Toradora!_(2008)_-_01v2_-_Tiger_and_Dragon_[1280x720_H.264_FLAC][1234ABCD].mkv'
ls *.mkv | anitomy --json
```

Pass `--no-title`, `--no-episode`, etc. to disable individual categories; see
`anitomy --help`.

## Layout

```
anitomy/       core Rust crate (published as `anitomy-ng`) — no unsafe, no non-dev dependencies
anitomy-py/    Python bindings (pyo3 + maturin, published as `anitomy-ng`), typed:
               ElementKind is a real enum.Enum, Element a real dataclass
anitomy-js/    JavaScript/TypeScript bindings (wasm-bindgen, published to npm as `anitomy-ng`)
anitomy-c/     C ABI (cdylib/staticlib) over the core — the only crate with `unsafe`;
               the foundation for non-Rust bindings
bindings/csharp/  .NET bindings (P/Invoke over anitomy-c, published to NuGet as `AnitomyNg`)
third_party/   vendored upstream test fixtures, not compiled — see third_party/README.md
scripts/       fixture-generation tooling
```

## Development

```sh
cargo test -p anitomy-ng --test conformance     # Rust conformance suite
cd anitomy-py && uv run --extra test pytest tests/ -q   # Python conformance suite
```

## License

Licensed under the Mozilla Public License 2.0 — see [`LICENSE`](LICENSE).

This project builds on the following, **all MPL-2.0**, and is distributed
under the same license accordingly:

- [erengy/anitomy](https://github.com/erengy/anitomy) (© Eren Okka) — the C++
  implementation this project is a port of.
- [Rapptz/anitomy-rs](https://github.com/Rapptz/anitomy-rs) (© Rapptz) — an
  independent Rust reimplementation; some logic and beyond-upstream keywords
  are adapted from it.
- [igorcmoura/anitopy](https://github.com/igorcmoura/anitopy) (© Igor C.
  Moura) — its test data (`table.py`/`failing_table.py`) is used as a
  conformance fixture suite.

`third_party/` vendors this upstream material under their own MPL-2.0
licenses — see [`third_party/README.md`](third_party/README.md).
