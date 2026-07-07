// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Port of `include/anitomy/detail/parser/video_resolution.hpp`.

use std::sync::OnceLock;

use regex::Regex;

use crate::detail::token::{is_free_token, is_numeric_token, Token};
use crate::element::{Element, ElementKind};

/// `\d{3,4}(?:[ipP]|(?:x|X|×)\d{3,4}[ipP]?)`, full match. A video
/// resolution can be in `1080p` or `1920x1080` format.
fn pattern() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        crate::detail::regex_util::compile(r"^[0-9]{3,4}(?:[ipP]|[xX\u{00D7}][0-9]{3,4}[ipP]?)$")
    })
}

pub(super) fn parse_video_resolution(tokens: &mut [Token]) -> Vec<Element> {
    let mut elements = Vec::new();

    // Find all free tokens matching the pattern.
    for token in tokens
        .iter_mut()
        .filter(|t| is_free_token(t) && pattern().is_match(&t.value))
    {
        token.element_kind = Some(ElementKind::VideoResolution);
        elements.push(Element {
            kind: ElementKind::VideoResolution,
            value: token.value.clone(),
            position: token.position,
        });
    }

    // If not found, look for special cases. Beyond-upstream: `720` added
    // alongside upstream's `1080`-only special case; upstream lacks it, though
    // its own fixture wants a bare `720` resolved the same way as `1080`.
    if elements.is_empty() {
        if let Some(token) = tokens.iter_mut().find(|t| {
            is_free_token(t) && is_numeric_token(t) && matches!(t.value.as_str(), "1080" | "720")
        }) {
            token.element_kind = Some(ElementKind::VideoResolution);
            elements.push(Element {
                kind: ElementKind::VideoResolution,
                value: token.value.clone(),
                position: token.position,
            });
        }
    }

    elements
}
