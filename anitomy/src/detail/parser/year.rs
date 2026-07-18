// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Port of `include/anitomy/detail/parser/year.hpp`.

use crate::detail::token::{is_free_token, Token, TokenKind};
use crate::detail::util::to_int;
use crate::element::{Element, ElementKind};

fn is_year_shaped(value: &str) -> bool {
    let number = to_int(value);
    1950 < number && number < 2050
}

pub(super) fn parse_year(tokens: &mut [Token]) -> Option<Element> {
    let len = tokens.len();
    if len < 3 {
        return None;
    }

    // Find the first free isolated number within the interval.
    let index = (0..=len - 3).find(|&i| {
        let is_isolated = tokens
            .get(i)
            .is_some_and(|t| t.kind == TokenKind::OpenBracket)
            && tokens
                .get(i + 2)
                .is_some_and(|t| t.kind == TokenKind::CloseBracket);
        if !is_isolated {
            return false;
        }
        let Some(middle) = tokens.get(i + 1) else {
            return false;
        };
        is_free_token(middle) && middle.is_number && is_year_shaped(middle.value)
    });

    if let Some(index) = index {
        let token = tokens.get_mut(index + 1)?;
        token.element_kind = Some(ElementKind::Year);
        return Some(Element {
            kind: ElementKind::Year,
            value: token.value.to_string(),
            position: token.position,
        });
    }

    // Beyond-upstream fix: fall back to a bare (non-bracketed) free number in
    // the year range, e.g. `... [DivX5 AC3] 1994 [852X480] ...`. Upstream only
    // looks for a bracket-enclosed year, though its own fixture wants the bare
    // year recognized too. Runs only when the bracketed strategy above found
    // nothing, so it can't override a more confident match.
    let index = tokens
        .iter()
        .position(|t| is_free_token(t) && t.is_number && is_year_shaped(t.value))?;
    let token = tokens.get_mut(index)?;
    token.element_kind = Some(ElementKind::Year);
    Some(Element {
        kind: ElementKind::Year,
        value: token.value.to_string(),
        position: token.position,
    })
}
