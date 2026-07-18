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
    merge_regional_languages(&mut tokens, &mut elements);

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

    detect_content_bundle(&tokens, &mut elements);

    elements.sort_by_key(|e| e.position);
    elements
}

/// Folds a trailing `-<2-letter region>` back into a `Language` code the dash
/// split off, e.g. `[POR-BR]` (tokenized `POR` `-` `BR`) -> `POR-BR`. Generic
/// over every code, unlike the literal `PT-BR` keyword.
fn merge_regional_languages(tokens: &mut [Token], elements: &mut [Element]) {
    use super::keyword::is_language_code;
    use super::token::{is_dash_token, TokenKind};

    for i in 0..tokens.len() {
        let Some(base) = tokens.get(i) else { continue };
        if base.element_kind != Some(ElementKind::Language) || !is_language_code(base.value) {
            continue;
        }
        let position = base.position;

        let Some(dash) = tokens.get(i + 1).filter(|t| is_dash_token(t)) else {
            continue;
        };
        let dash_value = dash.value;
        // A 2-char Text token is a whole word (Text runs are maximal).
        let Some(region) = tokens.get(i + 2) else {
            continue;
        };
        if region.kind != TokenKind::Text
            || region.element_kind.is_some()
            || region.value.len() != 2
            || !region.value.bytes().all(|b| b.is_ascii_alphabetic())
        {
            continue;
        }
        let region_value = region.value;

        if let Some(e) = elements
            .iter_mut()
            .find(|e| e.kind == ElementKind::Language && e.position == position)
        {
            e.value = format!("{}{dash_value}{region_value}", e.value);
            if let Some(region) = tokens.get_mut(i + 2) {
                region.element_kind = Some(ElementKind::Language);
            }
        }
    }
}

/// Content-bundle detection. An enclosed `+` joining content descriptors —
/// `[Season 2 + Movie]`, `(S01+S02+S03+S04+...+Specials+OVAs)` — is a *bundle
/// manifest*: the torrent packages multiple works (a season plus a movie, or a
/// whole franchise), not a single release. The `+` is the tell; no ordinary
/// filename composes seasons/types with it. The extracted season(s) and
/// type(s) are the batch's *contents*, which is fine to keep — the missing
/// piece is the batch nature itself, so flag it `release_information: Batch`
/// (matching how upstream labels these). Audio/video runs like `5.1+2.0` or
/// `x264+OGG` are excluded: both sides must be season/episode/type content.
fn detect_content_bundle(tokens: &[Token], elements: &mut Vec<Element>) {
    use super::token::{is_delimiter_token, is_open_bracket_token};

    let is_content = |kind: Option<ElementKind>| {
        matches!(
            kind,
            Some(ElementKind::Season) | Some(ElementKind::Episode) | Some(ElementKind::Type)
        )
    };
    // The nearest non-delimiter token to one side of `i`, bounded by the
    // enclosing bracket, is content.
    let content_side = |range: &mut dyn Iterator<Item = usize>| {
        for j in range {
            match tokens.get(j) {
                Some(t) if is_open_bracket_token(t) || is_delimiter_token(t) => {
                    if is_open_bracket_token(t) {
                        return false;
                    }
                }
                Some(t) => return is_content(t.element_kind),
                None => return false,
            }
        }
        false
    };

    let mut batch_position: Option<usize> = None;
    for (i, plus) in tokens.iter().enumerate() {
        if !(is_delimiter_token(plus) && plus.value == "+" && plus.is_enclosed) {
            continue;
        }
        if content_side(&mut (0..i).rev()) && content_side(&mut (i + 1..tokens.len())) {
            batch_position = Some(plus.position);
            break;
        }
    }

    let Some(position) = batch_position else {
        return;
    };
    let has_batch = elements.iter().any(|e| {
        e.kind == ElementKind::ReleaseInformation && e.value.eq_ignore_ascii_case("Batch")
    });
    if !has_batch {
        elements.push(Element {
            kind: ElementKind::ReleaseInformation,
            value: "Batch".to_string(),
            position,
        });
    }
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
