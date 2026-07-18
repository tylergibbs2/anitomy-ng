// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Port of `include/anitomy/detail/parser/episode.hpp`.
//!
//! Ten strategies, tried in order. Strategy 1 (regex-like episode/season/
//! version token) scans every free token before deciding whether to return
//! (no early exit), unlike strategies 2 onward, which return on their first
//! match. This asymmetry matches upstream.

use std::sync::OnceLock;

use regex::Regex;

use crate::detail::container::{find_next_token, find_prev_token, mark};
use crate::detail::delimiter::is_space;
use crate::detail::keyword::KeywordKind;
use crate::detail::token::{
    is_close_bracket_token, is_dash_token, is_delimiter_token, is_free_token,
    is_not_delimiter_token, is_numeric_token, Token, TokenKind,
};
use crate::detail::util::{byte_to_char_offset, equal_ignore_ascii_case, to_int};
use crate::element::{Element, ElementKind};

fn add_element_from_token(tokens: &mut [Token], idx: usize, elements: &mut Vec<Element>) {
    mark(tokens, idx, ElementKind::Episode);
    if let Some(token) = tokens.get(idx) {
        elements.push(Element {
            kind: ElementKind::Episode,
            value: token.value.to_string(),
            position: token.position,
        });
    }
}

fn add_element_with_value(
    tokens: &mut [Token],
    idx: usize,
    value: String,
    position: usize,
    elements: &mut Vec<Element>,
) {
    mark(tokens, idx, ElementKind::Episode);
    elements.push(Element {
        kind: ElementKind::Episode,
        value,
        position,
    });
}

// --- Strategy 1: `S01E02`, `1x02`, `E05`, `#5`, `05v2`, plus a delimited range like `01-02` ---

struct EpisodeTokenMatch {
    season_s: Option<(String, usize)>,
    season_x: Option<(String, usize)>,
    episode: (String, usize),
    version: Option<(String, usize)>,
}

/// `(?:S(\d{1,2})|(\d{1,2})x)?[E#]?(\d{1,4})(?:[vV](\d))?`, full match.
fn episode_token_pattern() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        crate::detail::regex_util::compile(
            r"^(?:S([0-9]{1,2})|([0-9]{1,2})x)?[E#]?([0-9]{1,4})(?:[vV]([0-9]))?$",
        )
    })
}

fn match_episode_token(value: &str) -> Option<EpisodeTokenMatch> {
    let caps = episode_token_pattern().captures(value)?;

    let group = |i: usize| {
        caps.get(i).map(|m| {
            (
                m.as_str().to_string(),
                byte_to_char_offset(value, m.start()),
            )
        })
    };

    // Group 3 (episode) is mandatory in the pattern, so it always participates
    // when the overall regex matches; `?` is a panic-free formality.
    let episode = group(3)?;

    Some(EpisodeTokenMatch {
        season_s: group(1),
        season_x: group(2),
        episode,
        version: group(4),
    })
}

fn is_episode_delimiter(token: &Token) -> bool {
    is_delimiter_token(token) && matches!(token.value.chars().next(), Some('-' | '~' | '&' | '+'))
}

/// Beyond upstream: a keyword-prefixed episode match (e.g. `EP07`, `OVA3`,
/// tokenized as keyword `EP`/`OVA` + free number `07`/`3`) followed by a
/// `.5` fraction (`EP07.5`, `OVA3.5`) should keep the fraction, as strategy
/// 3 already does for a bare number. Without this, strategy 1 matches the
/// integer alone via its keyword-prefix path
/// (`starts_with_episode_or_type_keyword`) and returns before strategy 3
/// runs, losing the `.5`. Upstream has this gap too.
fn trailing_fraction(tokens: &[Token], idx: usize) -> Option<(usize, usize)> {
    let dot_idx = idx + 1;
    let five_idx = idx + 2;
    let is_dot = tokens
        .get(dot_idx)
        .is_some_and(|t| is_delimiter_token(t) && t.value == ".");
    let is_five = tokens
        .get(five_idx)
        .is_some_and(|t| is_free_token(t) && t.value == "5");
    (is_dot && is_five).then_some((dot_idx, five_idx))
}

fn starts_with_episode_or_type_keyword(tokens: &[Token], idx: usize) -> bool {
    let Some(keyword) = tokens.get(idx).and_then(|t| t.keyword) else {
        return false;
    };
    match keyword.kind {
        KeywordKind::Episode => true,
        KeywordKind::Type | KeywordKind::EpisodeType => {
            tokens.get(idx).is_some_and(|t| t.value != "Movie")
        }
        _ => false,
    }
}

/// The `x1 || x2` marker test as a standalone predicate, so the marker-priority
/// pre-scan and the per-token check agree: the token is an explicit
/// `E##`/`#`/`SxxExx` episode (non-numeric, x1) or is prefixed by an
/// episode/type keyword (x2). A bare numeric range (`17-26`, valid only via x3)
/// is *not* marked.
fn is_marked_episode(tokens: &[Token], idx: usize) -> bool {
    let x1 = !tokens.get(idx).is_some_and(is_numeric_token);
    let x2 = find_prev_token(tokens, idx, is_not_delimiter_token)
        .is_some_and(|p| starts_with_episode_or_type_keyword(tokens, p));
    x1 || x2
}

fn apply_episode_match(
    tokens: &mut [Token],
    idx: usize,
    m: &EpisodeTokenMatch,
    elements: &mut Vec<Element>,
) {
    let Some(base_position) = tokens.get(idx).map(|t| t.position) else {
        return;
    };

    if let Some((season, offset)) = &m.season_s {
        elements.push(Element {
            kind: ElementKind::Season,
            value: season.clone(),
            position: base_position + offset,
        });
    } else if let Some((season, offset)) = &m.season_x {
        elements.push(Element {
            kind: ElementKind::Season,
            value: season.clone(),
            position: base_position + offset,
        });
    }

    let (ep_value, ep_offset) = &m.episode;
    add_element_with_value(
        tokens,
        idx,
        ep_value.clone(),
        base_position + ep_offset,
        elements,
    );

    if let Some((version, offset)) = &m.version {
        elements.push(Element {
            kind: ElementKind::ReleaseVersion,
            value: version.clone(),
            position: base_position + offset,
        });
    }
}

fn parse_episode_token_strategy(tokens: &mut [Token], elements: &mut Vec<Element>) {
    // Candidate indices are collected once, but upstream iterates a *lazy*
    // filtered view, so a token consumed as the second half of a range by
    // an earlier iteration (via `apply_episode_match`) is invisible to
    // later iterations there. Re-check freshness per-iteration to match.
    let free_indices: Vec<usize> = tokens
        .iter()
        .enumerate()
        .filter(|(_, t)| is_free_token(t))
        .map(|(i, _)| i)
        .collect();

    // Marker priority (issue #2): does the name carry an explicitly *marked*
    // episode (`E01`, `S01E02`, `EP07`) anywhere? If so, a bare unmarked range
    // found below is title numbering, not episodes, and is skipped.
    let has_marked_episode = tokens.iter().enumerate().any(|(i, t)| {
        is_free_token(t) && match_episode_token(&t.value).is_some() && is_marked_episode(tokens, i)
    });

    for idx in free_indices {
        if !tokens.get(idx).is_some_and(is_free_token) {
            continue;
        }
        let Some(value) = tokens.get(idx).map(|t| t.value) else {
            continue;
        };
        let Some(m1) = match_episode_token(&value) else {
            continue;
        };

        // x1/x2: an explicit `E##`/`#`/`SxxExx` token or a keyword-prefixed one.
        let is_marked = is_marked_episode(tokens, idx);

        // A single-digit range glued straight to a title word (`Ranma 1-2`,
        // `Ranma 1+2`) is a title's own numbering (½-style), not an episode
        // range: the first number is one digit and its left neighbour is a
        // free title token, not a dash/episode marker. A real batch range is
        // dash/marker-anchored (`- 01-02`) or multi-digit. Suppressing the
        // range leaves `1-2` in the title and lets the real episode (a later
        // `-NN` or `SxxExx`) be found instead.
        let glued_to_title = tokens.get(idx).is_some_and(|t| t.value.len() == 1)
            && find_prev_token(tokens, idx, is_not_delimiter_token)
                .and_then(|p| tokens.get(p))
                .is_some_and(|t| {
                    is_free_token(t)
                        && !t.is_enclosed
                        && !is_numeric_token(t)
                        && t.keyword.is_none()
                });

        // Always attempted (even if x1/x2 already make this valid) — matches upstream's
        // unconditional `is_episode_range` call, whose side effect (populating a second
        // match) is used below regardless of why `valid` ended up true.
        let range_next = tokens
            .get(idx + 1)
            .filter(|t| is_episode_delimiter(t))
            .and_then(|_| {
                let after_idx = idx + 2;
                let after_value = tokens.get(after_idx)?.value;
                let m2 = match_episode_token(&after_value)?;
                // A both-single-digit range glued to a title (`1-2`, `1+2`) is
                // title numbering, not a batch. A real batch reaches double
                // digits (`1-13`) or isn't glued, so it is unaffected.
                if glued_to_title && after_value.len() == 1 {
                    return None;
                }
                (to_int(&m1.episode.0) < to_int(&m2.episode.0)).then_some((after_idx, m2))
            });
        let x3 = range_next.is_some();

        if !(is_marked || x3) {
            continue;
        }
        // Marker priority (issue #2 / erengy/anitomy #36, beyond upstream): when
        // a marked episode exists elsewhere in the name, a bare unmarked range
        // (`17-26` beside `E01`) is title numbering, not episodes — the explicit
        // marker wins, so skip the range and leave it for the title.
        if has_marked_episode && !is_marked {
            continue;
        }

        let mut m1 = m1;
        let fraction = trailing_fraction(tokens, idx);
        // When the token already carried a version marker (`E06v1` + `.5`), the
        // fraction belongs to the *version* (`v1.5`), not the episode number:
        // gluing it onto the episode fabricates a half-episode (`06.5`) that
        // isn't there. Extend the version instead; only a versionless match
        // (`EP07` + `.5`) is a genuine fractional episode.
        let fraction_on_version = fraction.is_some() && m1.version.is_some();
        if fraction.is_some() {
            if let Some(version) = m1.version.as_mut() {
                version.0.push_str(".5");
            } else {
                m1.episode.0.push_str(".5");
            }
        }

        apply_episode_match(tokens, idx, &m1, elements);
        if let Some((dot_idx, five_idx)) = fraction {
            let kind = if fraction_on_version {
                ElementKind::ReleaseVersion
            } else {
                ElementKind::Episode
            };
            mark(tokens, dot_idx, kind);
            mark(tokens, five_idx, kind);
        }
        if let Some((after_idx, m2)) = range_next {
            apply_episode_match(tokens, after_idx, &m2, elements);
        }
    }
}

// --- Strategy 2: separated episodes, e.g. `8 & 10`, `1 ~ 12`, `01 of 24` ---

fn parse_separated_episodes(tokens: &mut [Token], elements: &mut Vec<Element>) -> bool {
    let candidates: Vec<usize> = tokens
        .iter()
        .enumerate()
        .filter(|(_, t)| is_free_token(t) && is_numeric_token(t))
        .map(|(i, _)| i)
        .collect();

    for idx in candidates {
        let search_start = idx + 1;
        let Some(sep_idx) = tokens.get(search_start..).and_then(|s| {
            s.iter()
                .position(|t| matches!(t.value, "&" | "~" | "of"))
                .map(|i| search_start + i)
        }) else {
            continue;
        };
        if tokens
            .get(search_start..sep_idx)
            .is_some_and(|s| s.iter().any(is_not_delimiter_token))
        {
            continue;
        }
        let Some(after_idx) = find_next_token(tokens, sep_idx, is_not_delimiter_token) else {
            continue;
        };
        if !tokens
            .get(after_idx)
            .is_some_and(|t| is_free_token(t) && is_numeric_token(t))
        {
            continue;
        }

        let is_of = tokens.get(sep_idx).is_some_and(|t| t.value == "of");

        add_element_from_token(tokens, idx, elements);
        if !is_of {
            add_element_from_token(tokens, after_idx, elements);
        }
        return true;
    }
    false
}

// --- Strategy 3: fractional episode, e.g. `07.5` ---

fn parse_fractional_episode(tokens: &mut [Token], elements: &mut Vec<Element>) -> bool {
    let len = tokens.len();
    if len < 3 {
        return false;
    }
    for i in 0..=(len - 3) {
        let is_number = tokens
            .get(i)
            .is_some_and(|t| is_free_token(t) && is_numeric_token(t));
        let is_dot = tokens
            .get(i + 1)
            .is_some_and(|t| is_delimiter_token(t) && t.value == ".");
        let is_fraction = tokens
            .get(i + 2)
            .is_some_and(|t| is_free_token(t) && t.value == "5");
        if !(is_number && is_dot && is_fraction) {
            continue;
        }

        let Some((number_value, position)) = tokens.get(i).map(|t| (t.value, t.position))
        else {
            continue;
        };
        add_element_with_value(tokens, i, format!("{number_value}.5"), position, elements);
        mark(tokens, i + 1, ElementKind::Episode);
        mark(tokens, i + 2, ElementKind::Episode);
        return true;
    }
    false
}

// --- Strategy 4: Japanese counter, e.g. `第01話` ---

/// `(?:第)?(\d{1,4})話`, full match.
fn japanese_episode_counter_pattern() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| crate::detail::regex_util::compile(r"^(?:第)?([0-9]{1,4})話$"))
}

fn parse_japanese_counter(tokens: &mut [Token], elements: &mut Vec<Element>) -> bool {
    for idx in 0..tokens.len() {
        if !tokens.get(idx).is_some_and(is_free_token) {
            continue;
        }
        let Some(value) = tokens.get(idx).map(|t| t.value) else {
            continue;
        };
        let Some(caps) = japanese_episode_counter_pattern().captures(&value) else {
            continue;
        };
        // Group 1 is mandatory in the pattern; `else continue` is unreachable
        // in practice but keeps this panic-free without an `expect`.
        let Some(group1) = caps.get(1) else {
            continue;
        };
        let offset = byte_to_char_offset(&value, group1.start());
        let group1 = group1.as_str().to_string();
        let position = tokens.get(idx).map_or(0, |t| t.position);
        add_element_with_value(tokens, idx, group1, position + offset, elements);
        return true;
    }
    false
}

// --- Strategy 5: equivalent numbers, e.g. `01 (176)`, `29 (04)` ---
//
// Upstream matches `(\d{1,4})\s*\((\d{1,4})\)` against a single token's
// value, which can practically never match: brackets always tokenize as
// their own `OpenBracket`/`CloseBracket` tokens (see `tokenizer.rs`), so no
// token's value can ever contain a literal `(`.
//
// Beyond upstream: a token-window check (number, `(`, number, `)`) that
// adds both numbers as separate `Episode` elements (outer number first,
// matching every corpus case's expected order), rather than upstream's
// dead single-token regex. Replacing the outer number with the
// parenthesized one instead of adding both fixes `Tegami Bachi ... 29 (04)`
// but breaks others like `Fairy_Tail_2_-_52_(227)` that want the outer
// number kept, so add both.
fn parse_equivalent_number(tokens: &mut [Token], elements: &mut Vec<Element>) -> bool {
    for outer_idx in 0..tokens.len() {
        if !tokens
            .get(outer_idx)
            .is_some_and(|t| is_free_token(t) && is_numeric_token(t))
        {
            continue;
        }
        let Some(open_idx) = find_next_token(tokens, outer_idx, is_not_delimiter_token) else {
            continue;
        };
        if !tokens
            .get(open_idx)
            .is_some_and(|t| t.kind == TokenKind::OpenBracket)
        {
            continue;
        }
        let Some(inner_idx) = find_next_token(tokens, open_idx, is_not_delimiter_token) else {
            continue;
        };
        if !tokens
            .get(inner_idx)
            .is_some_and(|t| is_free_token(t) && is_numeric_token(t))
        {
            continue;
        }
        let Some(close_idx) = find_next_token(tokens, inner_idx, is_not_delimiter_token) else {
            continue;
        };
        if !tokens
            .get(close_idx)
            .is_some_and(|t| t.kind == TokenKind::CloseBracket)
        {
            continue;
        }

        add_element_from_token(tokens, outer_idx, elements);
        add_element_from_token(tokens, inner_idx, elements);
        return true;
    }
    false
}

// --- Strategy 6: separated number, e.g. ` - 08` ---

fn parse_separated_number(tokens: &mut [Token], elements: &mut Vec<Element>) -> bool {
    let dash_indices: Vec<usize> = tokens
        .iter()
        .enumerate()
        .filter(|(_, t)| is_dash_token(t))
        .map(|(i, _)| i)
        .collect();

    for &dash_idx in dash_indices.iter().rev() {
        let start = dash_idx + 1;
        let Some(after_idx) = tokens
            .get(start..)
            .and_then(|s| s.iter().position(is_not_delimiter_token).map(|i| start + i))
        else {
            continue;
        };
        if tokens
            .get(after_idx)
            .is_some_and(|t| is_free_token(t) && is_numeric_token(t))
        {
            add_element_from_token(tokens, after_idx, elements);
            return true;
        }
    }
    false
}

// --- Strategy 7: isolated number, e.g. `[12]`, `(2006)` ---

fn parse_isolated_number(tokens: &mut [Token], elements: &mut Vec<Element>) -> bool {
    let len = tokens.len();
    if len < 3 {
        return false;
    }
    for i in 0..=(len - 3) {
        let is_isolated = tokens
            .get(i)
            .is_some_and(|t| t.kind == TokenKind::OpenBracket)
            && tokens
                .get(i + 2)
                .is_some_and(|t| t.kind == TokenKind::CloseBracket);
        if !is_isolated {
            continue;
        }
        if tokens
            .get(i + 1)
            .is_some_and(|t| is_free_token(t) && is_numeric_token(t))
        {
            add_element_from_token(tokens, i + 1, elements);
            return true;
        }
    }
    false
}

// --- Strategy 8: partial episode, e.g. `4a`, `111C` ---

/// `\d{1,4}[ABCabc]`, full match.
fn is_partial_episode(value: &str) -> bool {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| crate::detail::regex_util::compile(r"^[0-9]{1,4}[ABCabc]$"))
        .is_match(value)
}

fn parse_partial_episode(tokens: &mut [Token], elements: &mut Vec<Element>) -> bool {
    for idx in 0..tokens.len() {
        let Some(value) = tokens
            .get(idx)
            .filter(|t| is_free_token(t))
            .map(|t| t.value)
        else {
            continue;
        };
        if !is_partial_episode(&value) {
            continue;
        }
        // e.g. `NieR:Automata Ver1.1a`
        if idx > 1 && value == "1a" && tokens.get(idx - 2).is_some_and(|t| t.value == "Ver1") {
            continue;
        }
        add_element_from_token(tokens, idx, elements);
        return true;
    }
    false
}

// --- Strategy 9: first number ---

fn parse_first_number(tokens: &mut [Token], elements: &mut Vec<Element>) -> bool {
    let starts_with_episode_number = tokens
        .first()
        .is_some_and(|t| is_free_token(t) && is_numeric_token(t))
        && {
            if tokens.len() <= 2
                || tokens.get(1).is_some_and(is_dash_token)
                || tokens.get(2).is_some_and(is_dash_token)
            {
                true
            } else if tokens.get(1).is_some_and(|t| t.value == ".") {
                let starts_with_space = tokens
                    .get(2)
                    .is_some_and(|t| t.value.chars().next().is_some_and(is_space));
                let is_extension = tokens
                    .get(2)
                    .is_some_and(|t| t.element_kind == Some(ElementKind::FileExtension));
                starts_with_space || is_extension
            } else {
                false
            }
        };

    if starts_with_episode_number {
        add_element_from_token(tokens, 0, elements);
        return true;
    }
    false
}

// --- Strategy 10: last number ---

fn is_version_number(tokens: &[Token], idx: usize) -> bool {
    if !tokens.get(idx).is_some_and(is_numeric_token) || idx == 0 {
        return false;
    }
    tokens
        .get(idx - 1)
        .is_some_and(|t| is_delimiter_token(t) && t.value == ".")
}

/// Same "is this a `.`-glued decimal continuation" check as
/// [`is_version_number`], but looking forward from `idx` instead of back.
/// Needed for the `prev` side below: upstream applies its `is_version_number`
/// lambda to a *reverse* iterator there, and `std::prev` on a reverse
/// iterator steps to a later array index, not an earlier one — so upstream's
/// single lambda actually checks the token after `idx` in that call, not the
/// token before it (verified against upstream's compiled binary: for
/// `No.6 01`, `is_version_number(prev_token="6")` is `false`, since `"6"` is
/// followed by a space, not a dot — only the dot *preceding* `"6"` exists).
fn is_version_number_reversed(tokens: &[Token], idx: usize) -> bool {
    if !tokens.get(idx).is_some_and(is_numeric_token) {
        return false;
    }
    tokens
        .get(idx + 1)
        .is_some_and(|t| is_delimiter_token(t) && t.value == ".")
}

fn parse_last_number(tokens: &mut [Token], elements: &mut Vec<Element>) -> bool {
    let candidates: Vec<usize> = tokens
        .iter()
        .enumerate()
        .filter(|(_, t)| is_free_token(t) && is_numeric_token(t))
        .map(|(i, _)| i)
        .collect();

    for &idx in candidates.iter().rev() {
        if tokens.get(idx).is_some_and(|t| t.is_enclosed) {
            continue;
        }
        if tokens.get(idx).is_some_and(|t| t.position == 0) {
            continue;
        }

        let prev = find_prev_token(tokens, idx, is_not_delimiter_token);
        let next = find_next_token(tokens, idx, is_not_delimiter_token);

        if let Some(p) = prev {
            let Some(value) = tokens.get(p).map(|t| t.value) else {
                continue;
            };
            if equal_ignore_ascii_case(&value, "Cour") || equal_ignore_ascii_case(&value, "Part") {
                continue;
            }
            if equal_ignore_ascii_case(&value, "Movie") || equal_ignore_ascii_case(&value, "No") {
                continue;
            }
            if is_version_number_reversed(tokens, p) {
                continue;
            }
            if tokens.get(p).is_some_and(is_close_bracket_token) && value == "]" {
                continue;
            }
        }
        if let Some(n) = next {
            if is_version_number(tokens, n) {
                continue;
            }
        }
        if let (Some(p), Some(n)) = (prev, next) {
            if tokens.get(p).is_some_and(is_free_token) && tokens.get(n).is_some_and(is_free_token)
            {
                continue;
            }
        }

        add_element_from_token(tokens, idx, elements);
        return true;
    }
    false
}

pub(super) fn parse_episode(tokens: &mut [Token]) -> Vec<Element> {
    let mut elements = Vec::new();

    parse_episode_token_strategy(tokens, &mut elements);
    if !elements.is_empty() {
        return elements;
    }

    if parse_separated_episodes(tokens, &mut elements) {
        return elements;
    }
    if parse_fractional_episode(tokens, &mut elements) {
        return elements;
    }
    if parse_japanese_counter(tokens, &mut elements) {
        return elements;
    }
    if parse_equivalent_number(tokens, &mut elements) {
        return elements;
    }
    if parse_separated_number(tokens, &mut elements) {
        return elements;
    }
    if parse_isolated_number(tokens, &mut elements) {
        return elements;
    }
    if parse_partial_episode(tokens, &mut elements) {
        return elements;
    }
    if parse_first_number(tokens, &mut elements) {
        return elements;
    }
    parse_last_number(tokens, &mut elements);

    elements
}
