// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Port of `include/anitomy/detail/parser/season.hpp`.
//!
//! Three strategies run in sequence. The second always runs even when the
//! first already found something (matching upstream); only the third is
//! gated on `elements.is_empty()`.

use std::sync::OnceLock;

use regex::Regex;

use crate::detail::container::mark;
use crate::detail::keyword::KeywordKind;
use crate::detail::token::{
    is_dash_token, is_delimiter_token, is_free_token, is_numeric_token, Token,
};
use crate::detail::util::{byte_to_char_offset, from_ordinal_number, from_roman_number};
use crate::element::{Element, ElementKind};

fn is_season_keyword(token: &Token) -> bool {
    token.keyword.is_some_and(|k| k.kind == KeywordKind::Season)
}

/// `S(\d{1,2})`, full match, case-sensitive (only capital `S`).
fn s_prefixed_pattern() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| crate::detail::regex_util::compile(r"^S([0-9]{1,2})$"))
}

/// `(?:第)?(\d{1,2})期`, full match.
fn japanese_counter_pattern() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| crate::detail::regex_util::compile(r"^(?:第)?([0-9]{1,2})期$"))
}

pub(super) fn parse_season(tokens: &mut [Token]) -> Vec<Element> {
    let mut elements = Vec::new();

    // `2nd Season`, `Season 2`, `Season II`
    let len = tokens.len();
    if len >= 3 {
        for i in 0..=(len - 3) {
            let ends_with_season = tokens.get(i + 2).is_some_and(is_season_keyword)
                && tokens.get(i + 1).is_some_and(is_delimiter_token)
                && tokens.get(i).is_some_and(is_free_token);
            if ends_with_season {
                if let Some(value) = tokens.get(i).map(|t| t.value.clone()) {
                    if let Some(number) = from_ordinal_number(&value) {
                        let position = tokens.get(i).map_or(0, |t| t.position);
                        mark(tokens, i, ElementKind::Season);
                        mark(tokens, i + 2, ElementKind::Season);
                        elements.push(Element {
                            kind: ElementKind::Season,
                            value: number.to_string(),
                            position,
                        });
                        break;
                    }
                }
            }

            let starts_with_season = tokens.get(i).is_some_and(is_season_keyword)
                && tokens.get(i + 1).is_some_and(is_delimiter_token)
                && tokens.get(i + 2).is_some_and(is_free_token);
            if starts_with_season {
                if let Some((is_numeric, value, position)) = tokens
                    .get(i + 2)
                    .map(|t| (is_numeric_token(t), t.value.clone(), t.position))
                {
                    let resolved = if is_numeric {
                        Some(value.clone())
                    } else {
                        from_roman_number(&value).map(str::to_string)
                    };
                    if let Some(value) = resolved {
                        mark(tokens, i, ElementKind::Season);
                        mark(tokens, i + 2, ElementKind::Season);
                        elements.push(Element {
                            kind: ElementKind::Season,
                            value,
                            position,
                        });
                        // Range like `Season 4-6`: capture the endpoint too
                        // (mirroring `S01-02`), so the trailing number isn't
                        // left for the episode parser to claim as an episode.
                        if is_numeric
                            && tokens.get(i + 3).is_some_and(is_dash_token)
                            && tokens
                                .get(i + 4)
                                .is_some_and(|t| is_free_token(t) && is_numeric_token(t))
                        {
                            if let Some((v2, p2)) =
                                tokens.get(i + 4).map(|t| (t.value.clone(), t.position))
                            {
                                mark(tokens, i + 4, ElementKind::Season);
                                elements.push(Element {
                                    kind: ElementKind::Season,
                                    value: v2,
                                    position: p2,
                                });
                            }
                        }
                        break;
                    }
                }
            }
        }
    }

    // Season pattern (e.g. `S2`, `S01-02`)
    let free_indices: Vec<usize> = tokens
        .iter()
        .enumerate()
        .filter(|(_, t)| is_free_token(t))
        .map(|(i, _)| i)
        .collect();
    for idx in free_indices {
        if !tokens.get(idx).is_some_and(is_free_token) {
            continue;
        }
        let Some(value) = tokens.get(idx).map(|t| t.value.clone()) else {
            continue;
        };
        let Some(caps) = s_prefixed_pattern().captures(&value) else {
            continue;
        };
        // Group 1 is mandatory in the pattern; `else continue` is unreachable
        // in practice but keeps this panic-free without an `expect`.
        let Some(group1) = caps.get(1) else {
            continue;
        };
        let group1_offset = byte_to_char_offset(&value, group1.start());
        let group1 = group1.as_str().to_string();
        let position = tokens.get(idx).map_or(0, |t| t.position);

        mark(tokens, idx, ElementKind::Season);
        elements.push(Element {
            kind: ElementKind::Season,
            value: group1,
            position: position + group1_offset,
        });

        let Some(next) = tokens.get(idx + 1) else {
            continue;
        };
        if !is_dash_token(next) {
            continue;
        }
        let next2_idx = idx + 2;
        let is_match = tokens
            .get(next2_idx)
            .is_some_and(|t| is_free_token(t) && is_numeric_token(t));
        if !is_match {
            continue;
        }
        if let Some((value2, position2)) =
            tokens.get(next2_idx).map(|t| (t.value.clone(), t.position))
        {
            mark(tokens, next2_idx, ElementKind::Season);
            elements.push(Element {
                kind: ElementKind::Season,
                value: value2,
                position: position2,
            });
        }
        break;
    }

    // Japanese counter (e.g. `第2期`)
    if elements.is_empty() {
        for idx in 0..tokens.len() {
            if !tokens.get(idx).is_some_and(is_free_token) {
                continue;
            }
            let Some(value) = tokens.get(idx).map(|t| t.value.clone()) else {
                continue;
            };
            let Some(caps) = japanese_counter_pattern().captures(&value) else {
                continue;
            };
            // Group 1 is mandatory in the pattern; `else continue` is
            // unreachable in practice but keeps this panic-free without an `expect`.
            let Some(group1) = caps.get(1) else {
                continue;
            };
            let offset = byte_to_char_offset(&value, group1.start());
            let group1 = group1.as_str().to_string();
            let position = tokens.get(idx).map_or(0, |t| t.position);

            mark(tokens, idx, ElementKind::Season);
            elements.push(Element {
                kind: ElementKind::Season,
                value: group1,
                position: position + offset,
            });
            break;
        }
    }

    elements
}
