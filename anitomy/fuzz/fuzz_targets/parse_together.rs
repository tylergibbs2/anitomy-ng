#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|inputs: Vec<&str>| {
    let results = anitomy_ng::parse_together(&inputs, anitomy_ng::Options::default());
    assert_eq!(
        results.len(),
        inputs.len(),
        "parse_together must return one result per input"
    );
});
