// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Port of `include/anitomy/detail/container.hpp`, expressed over `&[Token]`
//! + index rather than generic iterators.

use super::token::Token;
use crate::element::ElementKind;

/// Sets `tokens[idx].element_kind`, if `idx` is in bounds. Shared by the
/// sub-parsers, all of which mark a token as "claimed" once they've decided
/// what it is.
pub(crate) fn mark(tokens: &mut [Token], idx: usize, kind: ElementKind) {
    if let Some(token) = tokens.get_mut(idx) {
        token.element_kind = Some(kind);
    }
}

/// Index of the nearest token before `from` (exclusive) matching `pred`,
/// searching backward. Mirrors upstream `find_prev_token(container, it,
/// predicate)`.
pub(crate) fn find_prev_token(
    tokens: &[Token],
    from: usize,
    pred: impl Fn(&Token) -> bool,
) -> Option<usize> {
    tokens.get(..from)?.iter().rposition(pred)
}

/// Index of the nearest token after `from` (exclusive) matching `pred`,
/// searching forward. Mirrors upstream `find_next_token(container, it,
/// predicate)`.
pub(crate) fn find_next_token(
    tokens: &[Token],
    from: usize,
    pred: impl Fn(&Token) -> bool,
) -> Option<usize> {
    let start = from.checked_add(1)?;
    let index_in_tail = tokens.get(start..)?.iter().position(pred)?;
    Some(start + index_in_tail)
}

/// Index of the first token at or after `start` matching `pred`, or
/// `tokens.len()` if none. Mirrors `std::find_if` returning `end()`, which
/// upstream's finders compare against in chains of finds.
pub(crate) fn find_from(tokens: &[Token], start: usize, pred: impl Fn(&Token) -> bool) -> usize {
    tokens
        .get(start..)
        .and_then(|s| s.iter().position(pred))
        .map_or(tokens.len(), |i| start + i)
}
