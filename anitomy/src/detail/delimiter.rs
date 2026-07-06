// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Port of `include/anitomy/detail/delimiter.hpp`: character-class predicates
//! (space, dash, delimiter).

pub(crate) fn is_space(ch: char) -> bool {
    matches!(ch, ' ' | '\t' | '\u{00A0}' | '\u{200B}' | '\u{3000}')
}

pub(crate) fn is_dash(ch: char) -> bool {
    matches!(
        ch,
        '-' | '\u{00AD}'
            | '\u{2010}'
            | '\u{2011}'
            | '\u{2012}'
            | '\u{2013}'
            | '\u{2014}'
            | '\u{2015}'
    )
}

pub(crate) fn is_delimiter(ch: char) -> bool {
    matches!(ch, '_' | '.' | ',' | '&' | '~' | '+' | '|' | ':') || is_space(ch) || is_dash(ch)
}
