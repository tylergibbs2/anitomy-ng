// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Port of `include/anitomy/detail/parser/file_extension.hpp`.

use crate::detail::token::{Token, TokenKind};
use crate::element::{Element, ElementKind};

// Beyond-upstream extensions. `7z` (seen as a `.patch.7z` wrapper) is absent
// from upstream's list. `ass`/`srt`/`ssa` (subtitles) and `zip` (archives)
// cover releases that ship standalone subtitle/archive files; Rapptz/anitomy-rs
// recognizes these too.
const EXTENSIONS: &[&str] = &[
    "3gp", "avi", "divx", "flv", "m2ts", "m4v", "mkv", "mov", "mp4", "mpg", "ogm", "rm", "rmvb",
    "ts", "webm", "wmv", "7z", "ass", "srt", "ssa", "zip",
];

pub(super) fn parse_file_extension(tokens: &mut [Token]) -> Option<Element> {
    let len = tokens.len();
    if len < 2 {
        return None;
    }

    let is_extension = tokens.get(len - 1).is_some_and(|t| {
        matches!(t.kind, TokenKind::Keyword | TokenKind::Text)
            && EXTENSIONS.contains(&t.value.as_str())
    });
    let is_dot = tokens
        .get(len - 2)
        .is_some_and(|t| t.kind == TokenKind::Delimiter && t.value == ".");
    if !is_extension || !is_dot {
        return None;
    }

    let token = tokens.get_mut(len - 1)?;
    token.kind = TokenKind::Text; // in case it was previously marked as keyword
    token.keyword = None;
    token.element_kind = Some(ElementKind::FileExtension);

    Some(Element {
        kind: ElementKind::FileExtension,
        value: token.value.clone(),
        position: token.position,
    })
}
