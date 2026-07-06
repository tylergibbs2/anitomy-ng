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
use crate::detail::container::{find_next_token, mark};
use crate::detail::element::element_from_token;
use crate::detail::keyword::KeywordKind;
use crate::detail::token::{is_not_delimiter_token, is_numeric_token, Token};
use crate::element::{Element, ElementKind};

fn is_part_keyword(token: &Token) -> bool {
    token
        .keyword
        .is_some_and(|k| k.kind == KeywordKind::Part && (!k.ambiguous || token.is_enclosed))
}

pub(super) fn parse_part(tokens: &mut [Token]) -> Option<Element> {
    let keyword_indices: Vec<usize> = tokens
        .iter()
        .enumerate()
        .filter(|(_, t)| is_part_keyword(t))
        .map(|(i, _)| i)
        .collect();

    for keyword_idx in keyword_indices {
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

    None
}
