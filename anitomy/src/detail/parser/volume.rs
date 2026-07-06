// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Port of `include/anitomy/detail/parser/volume.hpp`.

use std::sync::OnceLock;

use regex::Regex;

use crate::detail::container::{find_next_token, mark};
use crate::detail::keyword::KeywordKind;
use crate::detail::token::{is_free_token, is_not_delimiter_token, is_numeric_token, Token};
use crate::detail::util::byte_to_char_offset;
use crate::element::{Element, ElementKind};

fn is_volume_keyword(token: &Token) -> bool {
    token.keyword.is_some_and(|k| k.kind == KeywordKind::Volume)
}

/// Given the index of a token already matched as a single volume number
/// (e.g. `1` in `Vol.1&2`), checks whether it's immediately followed by
/// `&` and another numeric token, and if so returns that second token's
/// index. `&` always tokenizes as its own delimiter token, so this can't
/// be expressed as a single-token regex — see `multiple_volumes_pattern`.
fn matching_ampersand_volume(tokens: &[Token], first_idx: usize) -> Option<usize> {
    let amp = tokens.get(first_idx + 1)?;
    if amp.value != "&" {
        return None;
    }
    let second = tokens.get(first_idx + 2)?;
    (is_free_token(second) && is_numeric_token(second)).then_some(first_idx + 2)
}

/// `(\d{1,4})(?:[vV](\d))?`, full match.
fn single_volume_pattern() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    #[allow(clippy::expect_used)] // fixed literal pattern, see video_resolution.rs
    RE.get_or_init(|| Regex::new(r"^([0-9]{1,4})(?:[vV]([0-9]))?$").expect("valid regex"))
}

/// `(\d{1,4})&(\d{1,4})`, full match against a single token's value.
/// Practically dead — like `parser::episode`'s equivalent-number pattern,
/// `&` always tokenizes as its own delimiter token (see `tokenizer.rs`), so
/// no single token's value can ever contain a literal `&`. Kept as a
/// faithful translation regardless; `parse_volume` below separately handles
/// `Vol.1&2` as a token window instead.
fn multiple_volumes_pattern() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    #[allow(clippy::expect_used)]
    RE.get_or_init(|| Regex::new(r"^([0-9]{1,4})&([0-9]{1,4})$").expect("valid regex"))
}

pub(super) fn parse_volume(tokens: &mut [Token]) -> Vec<Element> {
    let mut elements = Vec::new();
    let mut search_from = 0usize;

    while let Some(volume_idx) = tokens
        .get(search_from..)
        .and_then(|s| s.iter().position(is_volume_keyword))
        .map(|i| search_from + i)
    {
        let Some(token_idx) = find_next_token(tokens, volume_idx, is_not_delimiter_token) else {
            break;
        };
        let Some(token) = tokens.get(token_idx) else {
            break;
        };
        if !is_free_token(token) {
            break;
        }
        let value = token.value.clone();
        let position = token.position;

        if let Some(caps) = single_volume_pattern().captures(&value) {
            mark(tokens, volume_idx, ElementKind::Volume);
            mark(tokens, token_idx, ElementKind::Volume);
            #[allow(clippy::expect_used)] // group 1 is mandatory in the pattern
            let number = caps.get(1).expect("group 1 always matches");
            elements.push(Element {
                kind: ElementKind::Volume,
                value: number.as_str().to_string(),
                position,
            });
            if let Some(version) = caps.get(2) {
                elements.push(Element {
                    kind: ElementKind::ReleaseVersion,
                    value: version.as_str().to_string(),
                    position: position + byte_to_char_offset(&value, version.start()),
                });
            } else if let Some(second_idx) = matching_ampersand_volume(tokens, token_idx) {
                #[allow(clippy::expect_used)]
                let second = tokens
                    .get(second_idx)
                    .expect("checked by matching_ampersand_volume");
                elements.push(Element {
                    kind: ElementKind::Volume,
                    value: second.value.clone(),
                    position,
                });
                mark(tokens, second_idx, ElementKind::Volume);
            }
        } else if let Some(caps) = multiple_volumes_pattern().captures(&value) {
            mark(tokens, volume_idx, ElementKind::Volume);
            mark(tokens, token_idx, ElementKind::Volume);
            #[allow(clippy::expect_used)]
            let first = caps
                .get(1)
                .expect("group 1 always matches")
                .as_str()
                .to_string();
            #[allow(clippy::expect_used)]
            let second = caps
                .get(2)
                .expect("group 2 always matches")
                .as_str()
                .to_string();
            elements.push(Element {
                kind: ElementKind::Volume,
                value: first,
                position,
            });
            elements.push(Element {
                kind: ElementKind::Volume,
                value: second,
                position,
            });
        }

        search_from = volume_idx + 1;
    }

    elements
}
