// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Port of `include/anitomy/detail/bracket.hpp`: a lookup table over bracket
//! characters.

pub(crate) fn is_open_bracket(ch: char) -> bool {
    matches!(
        ch,
        '(' | '['
            | '{'
            | '\u{300C}'
            | '\u{300E}'
            | '\u{3010}'
            | '\u{FF08}'
            | '\u{FF3B}'
            | '\u{FF5B}'
    )
}

pub(crate) fn is_close_bracket(ch: char) -> bool {
    matches!(
        ch,
        ')' | ']'
            | '}'
            | '\u{300D}'
            | '\u{300F}'
            | '\u{3011}'
            | '\u{FF09}'
            | '\u{FF3D}'
            | '\u{FF5D}'
    )
}

#[allow(dead_code)]
pub(crate) fn is_bracket(ch: char) -> bool {
    is_open_bracket(ch) || is_close_bracket(ch)
}
