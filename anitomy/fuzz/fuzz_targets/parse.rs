#![no_main]

use libfuzzer_sys::fuzz_target;

/// Options are fuzzed too: the conformance suites only ever run with the
/// defaults, so a panic reachable only with a category disabled would be
/// invisible to them.
fn options(bits: u16) -> anitomy_ng::Options {
    anitomy_ng::Options {
        parse_episode: bits & 1 << 0 != 0,
        parse_episode_title: bits & 1 << 1 != 0,
        parse_file_checksum: bits & 1 << 2 != 0,
        parse_file_extension: bits & 1 << 3 != 0,
        parse_part: bits & 1 << 4 != 0,
        parse_release_group: bits & 1 << 5 != 0,
        parse_season: bits & 1 << 6 != 0,
        parse_title: bits & 1 << 7 != 0,
        parse_video_resolution: bits & 1 << 8 != 0,
        parse_year: bits & 1 << 9 != 0,
    }
}

fuzz_target!(|input: (u16, &str)| {
    let (bits, filename) = input;
    let elements = anitomy_ng::parse(filename, options(bits));
    for e in &elements {
        assert!(
            e.position <= filename.chars().count(),
            "position {} out of bounds for {filename:?}",
            e.position
        );
    }
});
