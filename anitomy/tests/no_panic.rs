// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! `anitomy_ng::parse` takes arbitrary, untrusted filenames — it must never
//! panic. `src/lib.rs`'s `#![deny(clippy::unwrap_used, ...)]` catches most
//! panic sources statically; this test catches what lints can't (integer
//! overflow, an off-by-one that only breaks on a specific length, etc.) by
//! actually running the parser under `catch_unwind` over adversarial
//! inputs and the full fixture corpus.

use std::panic;

use serde::Deserialize;

const EDGE_CASES: &[&str] = &[
    "",
    ".",
    "..",
    "...",
    " ",
    "[]",
    "()",
    "[[[[[[",
    "]]]]]]",
    "----",
    "~~~~",
    "&&&&",
    "/////",
    "\\\\\\\\",
    "😀😀😀.mkv",
    "\0",
    "C:\\Users\\weird\\path.mkv",
    "🎬[Group]_Title_-_01_[💯].mkv",
    "v0v0v0v0",
    "1234567890",
    "S01E01S02E02S03E03",
];

fn assert_no_panic(input: &str) {
    let result = panic::catch_unwind(|| anitomy_ng::parse(input, anitomy_ng::Options::default()));
    assert!(
        result.is_ok(),
        "anitomy_ng::parse panicked on input: {input:?}"
    );
}

#[test]
fn never_panics_on_edge_cases() {
    let prev_hook = panic::take_hook();
    panic::set_hook(Box::new(|_| {})); // don't spam stderr for expected-to-be-caught panics

    for input in EDGE_CASES {
        assert_no_panic(input);
    }
    assert_no_panic(&"a".repeat(10_000));
    assert_no_panic(&"[".repeat(1_000));

    panic::set_hook(prev_hook);
}

#[derive(Deserialize)]
struct Case {
    input: String,
}

#[test]
fn never_panics_on_fixture_corpus() {
    const SUITES: &[&str] = &[
        include_str!("fixtures/anitomy_develop.json"),
        include_str!("fixtures/anitomy_master.json"),
        include_str!("fixtures/anitopy.json"),
        include_str!("fixtures/self_rolled.json"),
    ];

    let prev_hook = panic::take_hook();
    panic::set_hook(Box::new(|_| {}));

    for data in SUITES {
        let cases: Vec<Case> = serde_json::from_str(data).expect("fixture suite must parse");
        for case in &cases {
            assert_no_panic(&case.input);
        }
    }

    panic::set_hook(prev_hook);
}
