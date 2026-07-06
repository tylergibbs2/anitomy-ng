// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Conformance harness: replays each fixture suite under `tests/fixtures/`
//! against `anitomy_ng::parse`.
//!
//! Four independent suites (see `third_party/README.md` and
//! `scripts/build_fixtures.py`), checked under two different policies:
//!
//! - `self_rolled` is this project's own suite, so it is the ground truth and
//!   a hard gate: every case must pass.
//! - `anitomy_develop`, `anitomy_master`, and `anitopy` are external suites
//!   whose ground truth is imperfect and mutually contradictory (some cases no
//!   implementation passes, including upstream's own). Requiring 100% would
//!   mean overfitting to wrong fixtures, so each is checked against a
//!   checked-in known-failures manifest (`tests/known_failures/<suite>.txt`).
//!   The suite fails only when the failing set *changes*: a newly-failing case
//!   is a regression; a newly-passing case must be removed from the manifest
//!   (ratchet down). Each such change is a deliberate, reviewed commit, so the
//!   git history is the record of how conformance moves over time.
//!
//! After a deliberate parser or fixture change, regenerate the manifests:
//!
//! ```sh
//! UPDATE_KNOWN_FAILURES=1 cargo test -p anitomy-ng --test conformance
//! ```
//!
//! A case marked `"skip"` in its fixture (it exercises an `Options` field the
//! current API doesn't support) is counted separately and never dropped.

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use serde::Deserialize;

#[derive(Deserialize)]
struct Case {
    input: String,
    output: BTreeMap<String, serde_json::Value>,
    #[serde(default)]
    skip: Option<String>,
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

/// Parsed output and fixture-expected output for one case, grouped by kind.
fn parse_case(case: &Case) -> (ElementsByKind, ElementsByKind) {
    let elements = anitomy_ng::parse(&case.input, anitomy_ng::Options::default());
    let mut actual: ElementsByKind = BTreeMap::new();
    for element in &elements {
        actual
            .entry(element.kind.as_str().to_string())
            .or_default()
            .push(element.value.clone());
    }
    let expected: ElementsByKind = case
        .output
        .iter()
        .map(|(k, v)| (k.clone(), normalize(v)))
        .collect();
    (expected, actual)
}

fn load_cases(name: &str, data: &str) -> Vec<Case> {
    assert!(!data.trim().is_empty(), "fixtures/{name}.json is empty");
    serde_json::from_str(data).unwrap_or_else(|e| panic!("{name} fixtures must parse: {e}"))
}

/// Inputs of the non-skipped cases whose parsed output doesn't match the fixture.
fn failing_inputs(cases: &[Case]) -> BTreeSet<String> {
    cases
        .iter()
        .filter(|c| c.skip.is_none())
        .filter_map(|c| {
            let (expected, actual) = parse_case(c);
            (actual != expected).then(|| c.input.clone())
        })
        .collect()
}

fn manifest_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("known_failures")
        .join(format!("{name}.txt"))
}

fn load_known_failures(name: &str) -> BTreeSet<String> {
    match std::fs::read_to_string(manifest_path(name)) {
        Ok(s) => s
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty())
            .map(String::from)
            .collect(),
        Err(_) => BTreeSet::new(),
    }
}

fn write_known_failures(name: &str, failing: &BTreeSet<String>) {
    let path = manifest_path(name);
    std::fs::create_dir_all(path.parent().unwrap()).expect("create known_failures dir");
    // BTreeSet iterates sorted, so the manifest diffs cleanly; one input per line.
    let body: String = failing.iter().map(|s| format!("{s}\n")).collect();
    std::fs::write(&path, body).expect("write known_failures manifest");
}

fn updating() -> bool {
    std::env::var_os("UPDATE_KNOWN_FAILURES").is_some()
}

fn print_diff(cases: &[Case], input: &str) {
    let case = cases.iter().find(|c| c.input == input).unwrap();
    let (expected, actual) = parse_case(case);
    eprintln!("  input:    {input}\n  expected: {expected:?}\n  actual:   {actual:?}");
}

/// External suite: the set of failing inputs must equal the checked-in manifest.
fn run_manifest_suite(name: &str, data: &str) {
    let cases = load_cases(name, data);
    let tested = cases.iter().filter(|c| c.skip.is_none()).count();
    let failing = failing_inputs(&cases);

    if updating() {
        write_known_failures(name, &failing);
        println!(
            "{name}: wrote {} known failures ({}/{tested} passing)",
            failing.len(),
            tested - failing.len(),
        );
        return;
    }

    let known = load_known_failures(name);
    let regressions: Vec<String> = failing.difference(&known).cloned().collect();
    let fixed: Vec<String> = known.difference(&failing).cloned().collect();

    println!(
        "{name}: {}/{tested} passing ({} known failures)",
        tested - failing.len(),
        known.len(),
    );

    if regressions.is_empty() && fixed.is_empty() {
        return;
    }
    if !regressions.is_empty() {
        eprintln!(
            "[{name}] {} newly-failing case(s) (regression):",
            regressions.len()
        );
        for input in &regressions {
            print_diff(&cases, input);
        }
    }
    if !fixed.is_empty() {
        eprintln!(
            "[{name}] {} case(s) now pass — remove them from tests/known_failures/{name}.txt:",
            fixed.len(),
        );
        for input in &fixed {
            eprintln!("  {input}");
        }
    }
    panic!(
        "[{name}] conformance set changed; after reviewing, regenerate with \
         `UPDATE_KNOWN_FAILURES=1 cargo test -p anitomy-ng --test conformance`"
    );
}

/// Own suite: every non-skipped case must pass.
fn run_strict_suite(name: &str, data: &str) {
    let cases = load_cases(name, data);
    let tested = cases.iter().filter(|c| c.skip.is_none()).count();
    let failing = failing_inputs(&cases);
    println!(
        "{name}: {}/{tested} passing (strict)",
        tested - failing.len()
    );
    if !failing.is_empty() {
        for input in &failing {
            eprintln!("---");
            print_diff(&cases, input);
        }
        panic!(
            "[{name}] {} case(s) failed; this is the project's own suite and must pass 100%",
            failing.len(),
        );
    }
}

#[test]
fn anitomy_develop() {
    run_manifest_suite(
        "anitomy_develop",
        include_str!("fixtures/anitomy_develop.json"),
    );
}

#[test]
fn anitomy_master() {
    run_manifest_suite(
        "anitomy_master",
        include_str!("fixtures/anitomy_master.json"),
    );
}

#[test]
fn anitopy() {
    run_manifest_suite("anitopy", include_str!("fixtures/anitopy.json"));
}

#[test]
fn self_rolled() {
    run_strict_suite("self_rolled", include_str!("fixtures/self_rolled.json"));
}
