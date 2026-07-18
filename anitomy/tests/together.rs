// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Batch-parsing conformance harness for the path/directory schemas collected
//! in <https://github.com/tylergibbs2/anitomy-ng/issues/3>.
//!
//! Each fixture in `tests/fixtures/together.json` is a *set* of related filenames
//! plus the hand-authored, ground-truth output each record should produce (the
//! ideal per-file parse, not whatever the parser happens to emit today). A case
//! passes only when every record matches its expected map exactly.
//!
//! Same ratchet as `conformance.rs`: the set of failing case *names* must equal
//! the checked-in manifest (`tests/known_failures/together.txt`). A newly-failing
//! case is a regression; a newly-passing one must be removed from the manifest.
//! This is how "what is good" (passing) and "what is bad" (tracked, with the
//! target still recorded) are written down and kept honest over time.
//!
//! After a deliberate change, regenerate the manifest:
//!
//! ```sh
//! UPDATE_KNOWN_FAILURES=1 cargo test -p anitomy-ng --test together
//! ```

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use serde::Deserialize;

#[derive(Deserialize)]
struct Case {
    name: String,
    inputs: Vec<String>,
    outputs: Vec<BTreeMap<String, serde_json::Value>>,
    #[serde(default)]
    #[allow(dead_code)]
    source: Option<String>,
    #[serde(default)]
    #[allow(dead_code)]
    note: Option<String>,
}

type ElementsByKind = BTreeMap<String, Vec<String>>;

fn normalize(value: &serde_json::Value) -> Vec<String> {
    match value {
        serde_json::Value::Array(items) => items
            .iter()
            .map(|v| {
                v.as_str()
                    .expect("fixture array element must be a string")
                    .to_string()
            })
            .collect(),
        serde_json::Value::String(s) => vec![s.clone()],
        other => panic!("unexpected fixture value: {other:?}"),
    }
}

/// Parsed batch output and fixture-expected output, one grouped-by-kind map per
/// input, aligned by index.
fn parse_case(case: &Case) -> (Vec<ElementsByKind>, Vec<ElementsByKind>) {
    let inputs: Vec<&str> = case.inputs.iter().map(String::as_str).collect();
    let results = anitomy_ng::parse_together(&inputs, anitomy_ng::Options::default());

    let actual: Vec<ElementsByKind> = results
        .iter()
        .map(|elements| {
            let mut by_kind: ElementsByKind = BTreeMap::new();
            for element in elements {
                by_kind
                    .entry(element.kind.as_str().to_string())
                    .or_default()
                    .push(element.value.clone());
            }
            by_kind
        })
        .collect();

    let expected: Vec<ElementsByKind> = case
        .outputs
        .iter()
        .map(|record| {
            record
                .iter()
                .map(|(k, v)| (k.clone(), normalize(v)))
                .collect()
        })
        .collect();

    (expected, actual)
}

fn load_cases(data: &str) -> Vec<Case> {
    assert!(!data.trim().is_empty(), "fixtures/together.json is empty");
    let cases: Vec<Case> =
        serde_json::from_str(data).unwrap_or_else(|e| panic!("batch fixtures must parse: {e}"));
    for case in &cases {
        assert_eq!(
            case.inputs.len(),
            case.outputs.len(),
            "case {:?}: inputs and outputs must be the same length",
            case.name
        );
    }
    cases
}

/// Names of the cases whose batch output doesn't match the fixture.
fn failing_names(cases: &[Case]) -> BTreeSet<String> {
    cases
        .iter()
        .filter_map(|c| {
            let (expected, actual) = parse_case(c);
            (actual != expected).then(|| c.name.clone())
        })
        .collect()
}

fn manifest_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("known_failures")
        .join("together.txt")
}

fn load_known_failures() -> BTreeSet<String> {
    match std::fs::read_to_string(manifest_path()) {
        Ok(s) => s
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty())
            .map(String::from)
            .collect(),
        Err(_) => BTreeSet::new(),
    }
}

fn write_known_failures(failing: &BTreeSet<String>) {
    let path = manifest_path();
    std::fs::create_dir_all(path.parent().unwrap()).expect("create known_failures dir");
    let body: String = failing.iter().map(|s| format!("{s}\n")).collect();
    std::fs::write(&path, body).expect("write known_failures manifest");
}

fn updating() -> bool {
    std::env::var_os("UPDATE_KNOWN_FAILURES").is_some()
}

fn print_diff(cases: &[Case], name: &str) {
    let case = cases.iter().find(|c| c.name == name).unwrap();
    let (expected, actual) = parse_case(case);
    eprintln!("  case: {name}");
    for (i, (exp, act)) in expected.iter().zip(&actual).enumerate() {
        if exp != act {
            eprintln!("    input[{i}]: {}", case.inputs.get(i).map_or("", |s| s));
            eprintln!("      expected: {exp:?}");
            eprintln!("      actual:   {act:?}");
        }
    }
}

#[test]
fn together() {
    let cases = load_cases(include_str!("fixtures/together.json"));
    let failing = failing_names(&cases);

    if updating() {
        write_known_failures(&failing);
        println!(
            "together: wrote {} known failures ({}/{} passing)",
            failing.len(),
            cases.len() - failing.len(),
            cases.len(),
        );
        return;
    }

    let known = load_known_failures();
    let regressions: Vec<String> = failing.difference(&known).cloned().collect();
    let fixed: Vec<String> = known.difference(&failing).cloned().collect();

    println!(
        "together: {}/{} passing ({} known failures)",
        cases.len() - failing.len(),
        cases.len(),
        known.len(),
    );

    if regressions.is_empty() && fixed.is_empty() {
        return;
    }
    if !regressions.is_empty() {
        eprintln!(
            "[together] {} newly-failing case(s) (regression):",
            regressions.len()
        );
        for name in &regressions {
            print_diff(&cases, name);
        }
    }
    if !fixed.is_empty() {
        eprintln!(
            "[together] {} case(s) now pass — remove them from tests/known_failures/together.txt:",
            fixed.len(),
        );
        for name in &fixed {
            eprintln!("  {name}");
        }
    }
    panic!(
        "[together] conformance set changed; after reviewing, regenerate with \
         `UPDATE_KNOWN_FAILURES=1 cargo test -p anitomy-ng --test together`"
    );
}

/// Consistency invariant tying `parse_together` back to the single-file pipeline: a
/// batch of one input has no cross-file signal, so it must parse *identically*
/// to [`anitomy_ng::parse`] — never differently, and so never worse. Replaying
/// every single-file conformance input (`conformance.rs`'s four suites) as a
/// one-element batch pins that across the whole existing corpus for free, with
/// no hand-authored expectations: the fixtures' ground truth is irrelevant here,
/// only that the two entry points agree.
///
/// Inputs carrying a path separator are excluded on purpose: for a real path,
/// `parse_together` re-parses the filename component to suppress directory noise
/// (see `parse_one`), so diverging from the raw-string `parse` there is correct,
/// not a bug. The property is about the single-file *string* pipeline.
#[test]
fn single_input_matches_single_parse() {
    #[derive(Deserialize)]
    struct InputOnly {
        input: String,
    }

    // The same suites `conformance.rs` replays against `parse`.
    let suites = [
        ("anitomy_ng", include_str!("fixtures/anitomy_ng.json")),
        (
            "anitomy_develop",
            include_str!("fixtures/anitomy_develop.json"),
        ),
        (
            "anitomy_master",
            include_str!("fixtures/anitomy_master.json"),
        ),
        ("anitopy", include_str!("fixtures/anitopy.json")),
    ];

    let options = anitomy_ng::Options::default();
    let mut checked = 0usize;
    let mut divergent: Vec<String> = Vec::new();

    for (name, data) in suites {
        let cases: Vec<InputOnly> = serde_json::from_str(data)
            .unwrap_or_else(|e| panic!("{name} fixtures must parse: {e}"));
        for case in cases {
            // A path separator triggers directory-noise suppression in
            // parse_together, which legitimately differs from raw-string parse.
            if case.input.contains('/') || case.input.contains('\\') {
                continue;
            }
            checked += 1;

            let single = anitomy_ng::parse(&case.input, options);
            let batched = anitomy_ng::parse_together(&[case.input.as_str()], options)
                .into_iter()
                .next()
                .unwrap_or_default();

            if batched != single {
                divergent.push(format!(
                    "  input:       {}\n    parse:       {single:?}\n    parse_together: {batched:?}",
                    case.input
                ));
            }
        }
    }

    assert!(
        checked > 0,
        "no path-less single-file fixture inputs were checked"
    );
    assert!(
        divergent.is_empty(),
        "parse_together(&[x]) diverged from parse(x) for {} of {checked} path-less inputs:\n{}",
        divergent.len(),
        divergent.join("\n"),
    );
}

/// `parse_together` returns exactly one record per input, order-preserving. This is
/// structurally guaranteed — it maps 1:1 then reconciles through a `&mut [_]`
/// slice, which can't change the count — but pinned here so a refactor can't
/// silently regress it.
#[test]
fn result_count_matches_input_count() {
    let options = anitomy_ng::Options::default();

    assert_eq!(anitomy_ng::parse_together(&[], options).len(), 0);
    assert_eq!(
        anitomy_ng::parse_together(&["[G] Show - 01.mkv"], options).len(),
        1
    );

    for case in load_cases(include_str!("fixtures/together.json")) {
        let inputs: Vec<&str> = case.inputs.iter().map(String::as_str).collect();
        assert_eq!(
            anitomy_ng::parse_together(&inputs, options).len(),
            inputs.len(),
            "case {:?}",
            case.name
        );
    }

    // Heterogeneous: unrelated shows, a path, an empty string, and a string with
    // no parseable elements — still one record each.
    let mixed = [
        "[A] Alpha - 01 [1080p].mkv",
        "[B] Beta - 05 [720p].mkv",
        "C:\\Anime\\[G] Gamma (01-12)\\[G] Gamma - 03.mkv",
        "",
        "???",
    ];
    assert_eq!(
        anitomy_ng::parse_together(&mixed, options).len(),
        mixed.len()
    );
}
