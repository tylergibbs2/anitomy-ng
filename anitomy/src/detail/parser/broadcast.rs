// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Beyond upstream: Chinese fansub seasonal-broadcast markers (`★07月新番★`,
//! `[4月新番]`, `十月新番`). These sit where a title normally would, so
//! without claiming them the title parser returns the marker instead of the
//! show name.

use std::sync::OnceLock;

use regex::Regex;

use crate::detail::token::{is_free_token, Token};
use crate::element::{Element, ElementKind};

/// Optional decoration, optional month, then `新番` ("new series").
fn pattern() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        crate::detail::regex_util::compile(
            r"^[★☆◆◇■□▲△●○※\*]*(?:(?:[0-9]{1,2}|[一二三四五六七八九十]{1,3})月)?新番[★☆◆◇■□▲△●○※\*]*$",
        )
    })
}

pub(super) fn parse_broadcast(tokens: &mut [Token]) -> Vec<Element> {
    let mut elements = Vec::new();

    for token in tokens
        .iter_mut()
        .filter(|t| is_free_token(t) && pattern().is_match(t.value))
    {
        token.element_kind = Some(ElementKind::Other);
        elements.push(Element {
            kind: ElementKind::Other,
            value: token.value.to_string(),
            position: token.position,
        });
    }

    elements
}
