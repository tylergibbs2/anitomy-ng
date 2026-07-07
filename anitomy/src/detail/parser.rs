// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Port of `include/anitomy/detail/parser.hpp`: runs the per-category
//! sub-parsers (in `detail::parser::*`, one module per
//! `include/anitomy/detail/parser/*.hpp`) in a fixed order — later parsers
//! rely on earlier ones having already claimed (and mutated the `kind`/
//! `element_kind` of) their tokens — then sorts the collected elements by
//! position.
//!
//! Each sub-parser lives in a fairly self-contained file matching an upstream
//! header; `tests/conformance.rs` checks their combined output against the
//! upstream and anitopy fixture suites.

mod episode;
mod episode_title;
mod file_checksum;
mod file_extension;
mod keywords;
mod part;
mod release_group;
mod season;
mod title;
mod video_resolution;
mod volume;
mod year;

use super::token::Token;
use crate::element::{Element, ElementKind};
use crate::options::Options;

pub(crate) fn parse(mut tokens: Vec<Token>, options: &Options) -> Vec<Element> {
    let mut elements: Vec<Element> = Vec::new();

    let contains =
        |elements: &[Element], kind: ElementKind| elements.iter().any(|e| e.kind == kind);

    if options.parse_file_extension {
        elements.extend(file_extension::parse_file_extension(&mut tokens));
    }

    elements.extend(keywords::parse_keywords(&mut tokens, options));

    if options.parse_file_checksum {
        elements.extend(file_checksum::parse_file_checksum(&mut tokens));
    }

    if options.parse_video_resolution {
        elements.extend(video_resolution::parse_video_resolution(&mut tokens));
    }

    if options.parse_year {
        elements.extend(year::parse_year(&mut tokens));
    }

    if options.parse_season {
        elements.extend(season::parse_season(&mut tokens));
    }

    if options.parse_part {
        elements.extend(part::parse_part(&mut tokens));
    }

    if options.parse_episode {
        elements.extend(volume::parse_volume(&mut tokens));
        elements.extend(episode::parse_episode(&mut tokens));
    }

    if options.parse_title {
        elements.extend(title::parse_title(&mut tokens));
    }

    if options.parse_release_group && !contains(&elements, ElementKind::ReleaseGroup) {
        elements.extend(release_group::parse_release_group(&mut tokens));
    }

    if options.parse_episode_title && contains(&elements, ElementKind::Episode) {
        elements.extend(episode_title::parse_episode_title(&mut tokens));
    }

    // Minimum-description-length repair: the resolution parser's special case
    // claims a bare, shapeless number (`1080`/`720` with no `p`/`i`/`×WxH`) as
    // a resolution when nothing better exists. If that left the filename with
    // no episode at all, the bare number explains the parse better as the
    // episode: an episodeless anime filename is atypical, and a shapeless
    // resolution is a weaker reading than a plain trailing number. This flips
    // only the shapeless form and only when no episode was found elsewhere, so
    // a genuine `1080p`/`1920x1080` is never touched.
    if options.parse_episode
        && !contains(&elements, ElementKind::Episode)
        && contains(&elements, ElementKind::Title)
    {
        // Only an *unenclosed* bare number reads as an episode: a bracketed
        // one (e.g. `[BD-1080]`) sits with source/quality tags and is a real
        // resolution, so a movie/OVA with no episode keeps it.
        let unenclosed_at = |pos: usize| tokens.iter().any(|t| t.position == pos && !t.is_enclosed);
        if let Some(e) = elements.iter_mut().find(|e| {
            e.kind == ElementKind::VideoResolution
                && e.value.chars().all(|c| c.is_ascii_digit())
                && unenclosed_at(e.position)
        }) {
            e.kind = ElementKind::Episode;
        }
    }

    // Reconciliation pass: a number that appears twice in different notations
    // (e.g. `S02E06` and `[Episode 6]`, or `Season 1` and `S01`) yields two
    // elements of the same kind for one logical value. No reference parser
    // has this pass — they emit both. Collapse each padded/unpadded pair to
    // the cleaner (leading-zero-free) representative.
    dedupe_zero_padded(&mut elements, ElementKind::Episode);
    dedupe_zero_padded(&mut elements, ElementKind::Season);

    elements.sort_by_key(|e| e.position);
    elements
}

/// The integer a pure-decimal element value denotes, ignoring leading zeros,
/// or `None` if it isn't a plain integer (fractional, ranged, alphanumeric).
fn canonical_int(value: &str) -> Option<u64> {
    if value.is_empty() || !value.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    value.parse().ok()
}

/// Collapse `kind` elements whose values denote the same integer to a single
/// element, keeping the shortest notation (fewest leading zeros), tie-broken
/// by earliest position — so `["1", "01"]` -> `["1"]`, `["06", "6"]` -> `["6"]`.
fn dedupe_zero_padded(elements: &mut Vec<Element>, kind: ElementKind) {
    use std::collections::HashMap;

    // canonical value -> index of the current best representative.
    let mut best: HashMap<u64, usize> = HashMap::new();
    let mut drop: Vec<usize> = Vec::new();
    for (i, e) in elements.iter().enumerate() {
        if e.kind != kind {
            continue;
        }
        let Some(n) = canonical_int(&e.value) else {
            continue;
        };
        match best.get(&n).copied() {
            None => {
                best.insert(n, i);
            }
            Some(prev) => {
                let keep_new = elements.get(prev).is_some_and(|prev_e| {
                    (e.value.len(), e.position) < (prev_e.value.len(), prev_e.position)
                });
                if keep_new {
                    drop.push(prev);
                    best.insert(n, i);
                } else {
                    drop.push(i);
                }
            }
        }
    }
    if drop.is_empty() {
        return;
    }
    let mut i = 0;
    elements.retain(|_| {
        let keep = !drop.contains(&i);
        i += 1;
        keep
    });
}
