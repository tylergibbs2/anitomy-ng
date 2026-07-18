// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Port of `include/anitomy/detail/parser/file_checksum.hpp`.

use crate::detail::element::element_from_token;
use crate::detail::token::{is_free_token, Token};
use crate::element::{Element, ElementKind};

/// A CRC-32 checksum has 8 hexadecimal digits (e.g. `ABCD1234`).
fn is_checksum(value: &str) -> bool {
    value.chars().count() == 8 && value.chars().all(|c| c.is_ascii_hexdigit())
}

pub(super) fn parse_file_checksum(tokens: &mut [Token]) -> Option<Element> {
    let index = tokens
        .iter()
        .enumerate()
        .rev()
        .find(|(_, t)| is_free_token(t) && is_checksum(t.value))
        .map(|(i, _)| i)?;

    let token = tokens.get_mut(index)?;
    token.element_kind = Some(ElementKind::FileChecksum);

    Some(element_from_token(
        ElementKind::FileChecksum,
        token,
        None,
        None,
    ))
}
