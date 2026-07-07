// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

// Compile-only test of the generated TypeScript surface (tsify/wasm-bindgen).
// `tsc --noEmit` over this file fails if the published .d.ts drifts from the
// intended API — it never runs, so it needs no wasm. Built types live at
// dist-npm/bundler/ (see scripts/build-npm.sh), which is what the package's
// `types` field points at.
import { parse } from "../../dist-npm/bundler/anitomy_ng";
import type { Element, ElementKind, Options } from "../../dist-npm/bundler/anitomy_ng";

// parse() returns Element[]; each field has its expected type.
const elements: Element[] = parse("[Grp] Show - 01 [1080p].mkv");
for (const el of elements) {
  const kind: ElementKind = el.kind;
  const value: string = el.value;
  const position: number = el.position;
  void kind;
  void value;
  void position;
}

// options is optional, nullable, and a *partial* object of booleans.
parse("Show - 01.mkv");
parse("Show - 01.mkv", null);
const opts: Options = { parse_title: false };
parse("Show - 01.mkv", opts);

// ElementKind is the snake_case string union.
const known: ElementKind = "release_group";
void known;

// @ts-expect-error — a non-member string is not a valid ElementKind.
const bad: ElementKind = "not_a_real_kind";
void bad;

// @ts-expect-error — unknown option keys are rejected.
const badOpts: Options = { parse_everything: true };
void badOpts;
