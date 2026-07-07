// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Criterion speed benchmark for the Rust anime-filename parsers over the same
//! real-world corpus the conformance matrix uses. Each benchmark parses the
//! whole corpus per iteration; divide Criterion's estimate by the corpus size
//! (printed to stderr) for a per-file figure. `scripts/benchmark.py` reads the
//! machine-readable `target/criterion/<group>/<bench>/new/estimates.json`.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use serde::Deserialize;

#[derive(Deserialize)]
struct Case {
    input: String,
    #[serde(default)]
    skip: Option<String>,
}

/// The union of non-skipped fixture inputs, de-duplicated in first-seen order —
/// the same corpus `scripts/benchmark.py::all_inputs` builds.
fn corpus() -> Vec<String> {
    let suites = [
        include_str!("../../../anitomy/tests/fixtures/anitomy_develop.json"),
        include_str!("../../../anitomy/tests/fixtures/anitomy_master.json"),
        include_str!("../../../anitomy/tests/fixtures/anitopy.json"),
        include_str!("../../../anitomy/tests/fixtures/anitomy_ng.json"),
    ];
    let mut seen = std::collections::HashSet::new();
    let mut inputs = Vec::new();
    for data in suites {
        let cases: Vec<Case> = serde_json::from_str(data).expect("fixtures parse");
        for case in cases {
            if case.skip.is_none() && seen.insert(case.input.clone()) {
                inputs.push(case.input);
            }
        }
    }
    inputs
}

fn bench_parsers(c: &mut Criterion) {
    let inputs = corpus();
    eprintln!("corpus_size={}", inputs.len());

    let mut group = c.benchmark_group("parse");
    group.bench_function("anitomy_ng", |b| {
        b.iter(|| {
            for input in &inputs {
                black_box(anitomy_ng::parse(
                    black_box(input),
                    anitomy_ng::Options::default(),
                ));
            }
        });
    });
    group.bench_function("rapptz", |b| {
        b.iter(|| {
            for input in &inputs {
                let parsed: Vec<_> = anitomy::parse(black_box(input)).into_iter().collect();
                black_box(parsed);
            }
        });
    });
    group.finish();
}

criterion_group!(benches, bench_parsers);
criterion_main!(benches);
