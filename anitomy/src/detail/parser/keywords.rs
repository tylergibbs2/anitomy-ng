// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Port of `include/anitomy/detail/parser/keywords.hpp`.

use crate::detail::keyword::KeywordKind;
use crate::detail::token::{is_dash_token, is_keyword_token, Token};
use crate::element::{Element, ElementKind};
use crate::options::Options;

fn to_element_kind(kind: KeywordKind) -> ElementKind {
    use KeywordKind::*;
    match kind {
        AudioChannels | AudioCodec | AudioLanguage => ElementKind::AudioTerm,
        Device => ElementKind::Device,
        Episode => ElementKind::Episode,
        EpisodeType => ElementKind::Type,
        Language => ElementKind::Language,
        Other => ElementKind::Other,
        Part => ElementKind::Part,
        ReleaseGroup => ElementKind::ReleaseGroup,
        ReleaseInformation => ElementKind::ReleaseInformation,
        ReleaseVersion => ElementKind::ReleaseVersion,
        Season => ElementKind::Season,
        Source => ElementKind::Source,
        Subtitles | SubtitleLanguage => ElementKind::Subtitles,
        Type => ElementKind::Type,
        VideoCodec | VideoColorDepth | VideoDynamicRange | VideoFormat | VideoFrameRate
        | VideoProfile | VideoQuality => ElementKind::VideoTerm,
        VideoResolution => ElementKind::VideoResolution,
        Volume => ElementKind::Volume,
    }
}

fn is_prefix(kind: KeywordKind) -> bool {
    matches!(
        kind,
        KeywordKind::Episode | KeywordKind::Part | KeywordKind::Season | KeywordKind::Volume
    )
}

fn is_allowed(token: &Token, options: &Options) -> bool {
    let Some(keyword) = token.keyword else {
        return false;
    };
    match keyword.kind {
        KeywordKind::ReleaseGroup => options.parse_release_group,
        KeywordKind::VideoResolution => options.parse_video_resolution,
        _ => true,
    }
}

/// `v2` -> `2`, otherwise unchanged.
fn token_value(token: &Token) -> String {
    let Some(keyword) = token.keyword else {
        return token.value.clone();
    };
    if keyword.kind == KeywordKind::ReleaseVersion {
        token.value.chars().skip(1).collect()
    } else {
        token.value.clone()
    }
}

pub(super) fn parse_keywords(tokens: &mut [Token], options: &Options) -> Vec<Element> {
    let mut elements = Vec::new();

    let keyword_indices: Vec<usize> = tokens
        .iter()
        .enumerate()
        .filter(|(_, t)| is_keyword_token(t))
        .map(|(i, _)| i)
        .collect();

    for idx in keyword_indices {
        let Some(token) = tokens.get(idx) else {
            continue;
        };
        let Some(keyword) = token.keyword else {
            continue;
        };
        if !is_allowed(token, options) {
            continue;
        }

        // Beyond-upstream fix: an enclosed `ReleaseGroup` keyword (the narrow
        // set `THORA`/`VARYG`/`0x539`, known to appear bare) preceded by a
        // dash is the tail of a dash-joined name in the same bracket, e.g.
        // `[UTW-THORA]`. Suppress the bare `THORA` claim so `release_group.rs`
        // can return the full `UTW-THORA` span. Upstream has this collision too.
        let dash_glued_release_group = keyword.kind == KeywordKind::ReleaseGroup
            && token.is_enclosed
            && idx > 0
            && tokens.get(idx - 1).is_some_and(is_dash_token);
        // "Opening" is a common English word that only reads as an episode
        // type when bracketed (`[Opening]`); bare, it belongs to the title or
        // episode title (e.g. "Pool Opening"), so suppress it unless enclosed.
        // Other ambiguous type terms are left surfacing even unenclosed: the
        // acronyms (OVA/ONA/TV/SP/...) are rarely false positives, and the
        // remaining words (Special/Ending/Preview/PV) have contradictory
        // fixture ground truth, so suppressing them regresses more than it fixes.
        let common_word_type = matches!(keyword.kind, KeywordKind::EpisodeType)
            && token.value.eq_ignore_ascii_case("opening");

        let element_kind = to_element_kind(keyword.kind);
        let identified = (!keyword.ambiguous || token.is_enclosed) && !dash_glued_release_group;
        if identified {
            if let Some(token) = tokens.get_mut(idx) {
                token.element_kind = Some(element_kind);
            }
        }
        // Ambiguous Language keywords (e.g. "ITA" in "Bokura ga Ita") are
        // common-word false positives when unenclosed, unlike other ambiguous
        // kinds (audio/source/type terms), which should still surface even
        // unenclosed.
        let suppressed = ((keyword.kind == KeywordKind::Language || common_word_type)
            && !identified)
            || dash_glued_release_group;
        if !is_prefix(keyword.kind) && !suppressed {
            let Some(token) = tokens.get(idx) else {
                continue;
            };
            elements.push(Element {
                kind: element_kind,
                value: token_value(token),
                position: token.position,
            });
        }
    }

    elements
}
