// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Port of `include/anitomy/detail/parser/keywords.hpp`.

use std::sync::OnceLock;

use regex::Regex;

use crate::detail::keyword::KeywordKind;
use crate::detail::token::{is_dash_token, is_delimiter_token, is_keyword_token, Token};
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

/// e.g. `AACx2`, `AACx3`: a codec name fused with its own channel-count
/// suffix.
fn composite_audio_codec_pattern() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    #[allow(clippy::expect_used)]
    RE.get_or_init(|| Regex::new(r"^[A-Za-z]+[xX][0-9]$").expect("valid regex"))
}

/// A bare channel-layout spec, e.g. `5.1`, `2.0`.
fn bare_channel_layout_pattern() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    #[allow(clippy::expect_used)]
    RE.get_or_init(|| Regex::new(r"^[0-9]\.[0-9]$").expect("valid regex"))
}

/// Is the token at `idx` a `Language`-kind keyword joined by a bare `_` to
/// *another* `Language`-kind keyword on either side, e.g. `Jpn_Chs_Cht`?
fn is_underscore_chained_language(tokens: &[Token], idx: usize) -> bool {
    let is_language_at = |i: usize| {
        tokens
            .get(i)
            .and_then(|t| t.keyword)
            .is_some_and(|k| k.kind == KeywordKind::Language)
    };
    let is_underscore_at = |i: usize| {
        tokens
            .get(i)
            .is_some_and(|t| is_delimiter_token(t) && t.value == "_")
    };

    (idx >= 2 && is_underscore_at(idx - 1) && is_language_at(idx - 2))
        || (is_underscore_at(idx + 1) && is_language_at(idx + 2))
}

/// Is the token at `idx` immediately followed by a `.<digits>.<digits>`
/// tail, e.g. `divx` in `divx5.2.1`? That shape is a dotted version string
/// rather than a codec name; a codec is followed by a separator, never
/// another `.`-number pair.
fn is_dotted_version_tail(tokens: &[Token], idx: usize) -> bool {
    let is_dot = |i: usize| {
        tokens
            .get(i)
            .is_some_and(|t| is_delimiter_token(t) && t.value == ".")
    };
    let is_number = |i: usize| tokens.get(i).is_some_and(|t| t.is_number);

    is_dot(idx + 1) && is_number(idx + 2) && is_dot(idx + 3) && is_number(idx + 4)
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
        let starts_with_divx = token.value.to_ascii_lowercase().starts_with("divx");

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
        //
        // Beyond-upstream fix: a `_`-chained run of 2+ Language keywords in
        // one bracket, e.g. `[Jpn_Chs_Cht]`, is a multi-language tag rather
        // than the release's own language (as a lone `[JPN]` would be), so
        // suppress it. Upstream extracts all three as separate `language`
        // values; the fixture corpus wants none of them.
        let chained_language =
            keyword.kind == KeywordKind::Language && is_underscore_chained_language(tokens, idx);
        // Beyond-upstream fix: `divx`/`DivX5`/`DivX6` followed by a
        // `.<digits>.<digits>` tail (e.g. `divx5.2.1`) is a dotted version
        // string with a codec-shaped prefix, not a codec mention, so suppress
        // it. Upstream still extracts `divx` here; the fixture corpus wants
        // nothing.
        let divx_version_string = keyword.kind == KeywordKind::VideoCodec
            && starts_with_divx
            && is_dotted_version_tail(tokens, idx);
        let suppressed = (keyword.kind == KeywordKind::Language && !identified)
            || dash_glued_release_group
            || chained_language
            || divx_version_string;
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

    // Beyond-upstream fix: a `+`-joined list of bare channel-layout specs
    // (e.g. `[5.1+2.0]`) alongside a composite codec+channel value (e.g.
    // `AACx2`) is redundant detail the composite already encodes, so drop it.
    // Upstream emits both (`AACx2`, `5.1`, `2.0`) as separate `audio_term`
    // values; the fixture corpus wants only the composite kept.
    if elements.iter().any(|e| {
        e.kind == ElementKind::AudioTerm && composite_audio_codec_pattern().is_match(&e.value)
    }) {
        elements.retain(|e| {
            !(e.kind == ElementKind::AudioTerm && bare_channel_layout_pattern().is_match(&e.value))
        });
    }

    elements
}
