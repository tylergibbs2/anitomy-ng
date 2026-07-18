// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Port of `include/anitomy/detail/parser/part.hpp`.
//!
//! Beyond upstream: upstream's `is_part_keyword` accepts any token whose
//! keyword kind is `Part` and never checks the keyword's `ambiguous` flag
//! (`"Part"` is marked `AMBIGUOUS` in `keyword.rs`, e.g. for "Extra Part"/
//! "Part-Timer" false positives), even though `keywords.rs` already applies
//! the rule "an ambiguous keyword only counts as identified if it's
//! enclosed" for the same reason (see its `KeywordKind::Language` handling).
//! Applying that rule here fixes an over-claiming bug: an unenclosed, bare
//! "Part N" at the end of a long free-text run is almost always part of an
//! episode title (e.g. "... Humanity's Comeback, Part 1", "... The Two Magi
//! Part1"), not a multi-part release marker. Those are reliably either
//! enclosed (`(Cour 2)`, `(Season 1 Part 2)`) or use a non-ambiguous variant
//! (`Cour`, `Parte`), both unaffected here since only the bare `"Part"`
//! variant carries `AMBIGUOUS`.
use std::sync::OnceLock;

use regex::Regex;

use crate::detail::container::{find_next_token, mark};
use crate::detail::element::element_from_token;
use crate::detail::keyword::KeywordKind;
use crate::detail::token::{
    is_delimiter_token, is_free_token, is_not_delimiter_token, is_numeric_token, Token,
};
use crate::detail::util::from_ordinal_number;
use crate::element::{Element, ElementKind};

fn is_part_keyword(token: &Token) -> bool {
    token
        .keyword
        .is_some_and(|k| k.kind == KeywordKind::Part && (!k.ambiguous || token.is_enclosed))
}

/// `P(\d{1,2})`, full match, case-sensitive (only capital `P`), e.g. the `P2`
/// in `S3 P2` — a compact part marker paralleling the season's `S2`.
fn p_prefixed_pattern() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| crate::detail::regex_util::compile(r"^P([0-9]{1,2})$"))
}

pub(super) fn parse_part(tokens: &mut [Token]) -> Option<Element> {
    let keyword_indices: Vec<usize> = tokens
        .iter()
        .enumerate()
        .filter(|(_, t)| is_part_keyword(t))
        .map(|(i, _)| i)
        .collect();

    for keyword_idx in keyword_indices {
        // Ordinal before the keyword, e.g. `2nd Cour` (mirrors `2nd Season`).
        // Checked first so it wins over a following episode number: in
        // `... 2nd Cour - 01` the `01` is the episode, not the part.
        if keyword_idx >= 2
            && tokens.get(keyword_idx - 1).is_some_and(is_delimiter_token)
            && tokens.get(keyword_idx - 2).is_some_and(is_free_token)
        {
            if let Some((value, position)) =
                tokens.get(keyword_idx - 2).map(|t| (t.value, t.position))
            {
                if let Some(number) = from_ordinal_number(&value) {
                    mark(tokens, keyword_idx - 2, ElementKind::Part);
                    mark(tokens, keyword_idx, ElementKind::Part);
                    return Some(Element {
                        kind: ElementKind::Part,
                        value: number.to_string(),
                        position,
                    });
                }
            }
        }

        // Number after the keyword, e.g. `Part 2`, `Cour 2`.
        let Some(next_idx) = find_next_token(tokens, keyword_idx, is_not_delimiter_token) else {
            continue;
        };
        let Some(next) = tokens.get(next_idx) else {
            continue;
        };
        if !is_numeric_token(next) {
            continue;
        }

        mark(tokens, keyword_idx, ElementKind::Part);
        mark(tokens, next_idx, ElementKind::Part);

        let token = tokens.get(next_idx)?;
        return Some(element_from_token(ElementKind::Part, token, None, None));
    }

    // Compact part marker `P<n>` (e.g. `S3 P2`), paralleling the season `S2`.
    for idx in 0..tokens.len() {
        if !tokens.get(idx).is_some_and(is_free_token) {
            continue;
        }
        let Some((value, position)) = tokens.get(idx).map(|t| (t.value, t.position)) else {
            continue;
        };
        let Some(caps) = p_prefixed_pattern().captures(&value) else {
            continue;
        };
        let Some(group1) = caps.get(1) else {
            continue;
        };
        let group1 = group1.as_str().to_string();
        mark(tokens, idx, ElementKind::Part);
        return Some(Element {
            kind: ElementKind::Part,
            value: group1,
            position,
        });
    }

    None
}
