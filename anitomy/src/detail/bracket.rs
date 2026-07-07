// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Port of `include/anitomy/detail/bracket.hpp`: a lookup table over bracket
//! characters.

/// The recognized bracket pairs, `(open, close)`. Single source of truth for
/// all three predicates below.
const BRACKET_PAIRS: [(char, char); 9] = [
    ('(', ')'),
    ('[', ']'),
    ('{', '}'),
    ('\u{300C}', '\u{300D}'),
    ('\u{300E}', '\u{300F}'),
    ('\u{3010}', '\u{3011}'),
    ('\u{FF08}', '\u{FF09}'),
    ('\u{FF3B}', '\u{FF3D}'),
    ('\u{FF5B}', '\u{FF5D}'),
];

pub(crate) fn is_open_bracket(ch: char) -> bool {
    BRACKET_PAIRS.iter().any(|&(open, _)| open == ch)
}

pub(crate) fn is_close_bracket(ch: char) -> bool {
    BRACKET_PAIRS.iter().any(|&(_, close)| close == ch)
}

/// The closing bracket that matches `open`, or `None` if `open` isn't an
/// opening bracket.
pub(crate) fn matching_close(open: char) -> Option<char> {
    BRACKET_PAIRS
        .iter()
        .find(|&&(o, _)| o == open)
        .map(|&(_, close)| close)
}
