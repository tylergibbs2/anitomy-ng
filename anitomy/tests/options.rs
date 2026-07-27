// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Each `Options` field must suppress exactly its own kind. The conformance
//! suites only ever run with the defaults, so nothing else covers this: a
//! category emitted from another parser's code path (`S02` matching the
//! episode-token regex and pushing a `Season`) slips through unnoticed.

use anitomy_ng::{Element, ElementKind, Options};

fn has(elements: &[Element], kind: ElementKind) -> bool {
    elements.iter().any(|e| e.kind == kind)
}

fn values(elements: &[Element], kind: ElementKind) -> Vec<&str> {
    elements
        .iter()
        .filter(|e| e.kind == kind)
        .map(|e| e.value.as_str())
        .collect()
}

/// One filename producing all ten option-gated categories.
const ALL: &str = "[Grp] Show Name (2019) S02 (Part 2) - 05 - Episode Name [1080p][ABCD1234].mkv";

fn cases() -> Vec<(&'static str, ElementKind, Options)> {
    let d = Options::default;
    vec![
        (
            "parse_episode",
            ElementKind::Episode,
            Options {
                parse_episode: false,
                ..d()
            },
        ),
        (
            "parse_episode_title",
            ElementKind::EpisodeTitle,
            Options {
                parse_episode_title: false,
                ..d()
            },
        ),
        (
            "parse_file_checksum",
            ElementKind::FileChecksum,
            Options {
                parse_file_checksum: false,
                ..d()
            },
        ),
        (
            "parse_file_extension",
            ElementKind::FileExtension,
            Options {
                parse_file_extension: false,
                ..d()
            },
        ),
        (
            "parse_part",
            ElementKind::Part,
            Options {
                parse_part: false,
                ..d()
            },
        ),
        (
            "parse_release_group",
            ElementKind::ReleaseGroup,
            Options {
                parse_release_group: false,
                ..d()
            },
        ),
        (
            "parse_season",
            ElementKind::Season,
            Options {
                parse_season: false,
                ..d()
            },
        ),
        (
            "parse_title",
            ElementKind::Title,
            Options {
                parse_title: false,
                ..d()
            },
        ),
        (
            "parse_video_resolution",
            ElementKind::VideoResolution,
            Options {
                parse_video_resolution: false,
                ..d()
            },
        ),
        (
            "parse_year",
            ElementKind::Year,
            Options {
                parse_year: false,
                ..d()
            },
        ),
    ]
}

#[test]
fn every_option_has_a_case() {
    let covered: Vec<&str> = cases().iter().map(|(f, _, _)| *f).collect();
    assert_eq!(
        covered,
        Options::FIELDS,
        "an Options field has no coverage here"
    );
}

#[test]
fn defaults_emit_every_gated_kind() {
    let elements = anitomy_ng::parse(ALL, Options::default());
    for (field, kind, _) in cases() {
        assert!(
            has(&elements, kind),
            "{ALL:?} must produce {kind} to test {field}"
        );
    }
}

#[test]
fn disabling_an_option_suppresses_its_kind() {
    for (field, kind, options) in cases() {
        let elements = anitomy_ng::parse(ALL, options);
        assert!(
            !has(&elements, kind),
            "{field}=false still emitted {kind}: {:?}",
            values(&elements, kind)
        );
    }
}

#[test]
fn disabling_an_option_leaves_other_kinds_alone() {
    for (field, kind, options) in cases() {
        // Title and episode are upstream of most other parsers, so turning
        // them off legitimately cascades; the rest should be independent.
        if matches!(kind, ElementKind::Title | ElementKind::Episode) {
            continue;
        }
        let elements = anitomy_ng::parse(ALL, options);
        for (_, other, _) in cases() {
            if other == kind || matches!(other, ElementKind::EpisodeTitle) {
                continue;
            }
            assert!(has(&elements, other), "{field}=false also removed {other}");
        }
    }
}

#[test]
fn all_options_off_emits_nothing_gated() {
    let off = Options {
        parse_episode: false,
        parse_episode_title: false,
        parse_file_checksum: false,
        parse_file_extension: false,
        parse_part: false,
        parse_release_group: false,
        parse_season: false,
        parse_title: false,
        parse_video_resolution: false,
        parse_year: false,
    };
    let elements = anitomy_ng::parse(ALL, off);
    for (field, kind, _) in cases() {
        assert!(
            !has(&elements, kind),
            "{field} is off but {kind} was emitted"
        );
    }
}
