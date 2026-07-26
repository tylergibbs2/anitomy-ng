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

fn assert_no_panic_together(inputs: &[&str]) {
    let result =
        panic::catch_unwind(|| anitomy_ng::parse_together(inputs, anitomy_ng::Options::default()));
    assert!(
        result.is_ok(),
        "anitomy_ng::parse_together panicked on inputs: {inputs:?}"
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

/// `parse_together` does more index arithmetic than `parse` (path splitting,
/// cross-input diffing), so it gets the same treatment plus set-shaped edge
/// cases: empty set, one input, duplicates, and inputs sharing no prefix.
#[test]
fn never_panics_on_edge_case_sets() {
    let prev_hook = panic::take_hook();
    panic::set_hook(Box::new(|_| {}));

    assert_no_panic_together(&[]);
    for input in EDGE_CASES {
        assert_no_panic_together(&[input]);
        assert_no_panic_together(&[input, input]);
        assert_no_panic_together(&[input, "Show - 01.mkv"]);
    }
    assert_no_panic_together(EDGE_CASES);

    let long = "a".repeat(10_000);
    assert_no_panic_together(&[&long, &long]);

    // Path-shaped: separators at both ends, mixed styles, deep nesting.
    assert_no_panic_together(&[
        "/",
        "\\",
        "//",
        "C:\\",
        "\\\\server\\share",
        "a/b/c/d/e/f/g/01.mkv",
        "a\\b/c\\d/01.mkv",
        "/leading/Show - 01.mkv",
        "trailing/",
    ]);

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
        include_str!("fixtures/anitomy_ng.json"),
    ];

    let prev_hook = panic::take_hook();
    panic::set_hook(Box::new(|_| {}));

    for data in SUITES {
        let cases: Vec<Case> = serde_json::from_str(data).expect("fixture suite must parse");
        for case in &cases {
            assert_no_panic(&case.input);
        }
        // Whole suite as one set, and each adjacent pair — unrelated inputs
        // exercise the diffing path differently than a coherent batch does.
        let inputs: Vec<&str> = cases.iter().map(|c| c.input.as_str()).collect();
        assert_no_panic_together(&inputs);
        for pair in inputs.windows(2) {
            assert_no_panic_together(pair);
        }
    }

    panic::set_hook(prev_hook);
}
