// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Port of `include/anitomy/detail/parser/episode_title.hpp`.

use crate::detail::container::{find_from, mark};
use crate::detail::element::{build_element_value, underscore_is_separator};
use crate::detail::token::{
    is_close_bracket_token, is_enclosed_token, is_free_token, is_identified_token,
    is_open_bracket_token, Token,
};
use crate::element::{Element, ElementKind};

fn is_invalid_token(token: &Token) -> bool {
    if !is_identified_token(token) {
        return false;
    }
    !matches!(
        token.element_kind,
        Some(ElementKind::Episode) | Some(ElementKind::ReleaseVersion) | Some(ElementKind::Season)
    )
}

fn is_part_token(token: &Token) -> bool {
    token.element_kind == Some(ElementKind::Part)
}

fn find_episode_title(tokens: &[Token]) -> (usize, usize) {
    let len = tokens.len();

    let episode = find_from(tokens, 0, |t| t.element_kind == Some(ElementKind::Episode));
    if episode == len {
        return (len, len);
    }

    // Find the first free unenclosed range after episode,
    // e.g. `[Group] Title - Episode - Episode Title [Info]`.
    let mut first = find_from(tokens, episode, |t| {
        is_free_token(t) && !is_enclosed_token(t)
    });
    let any_invalid = tokens
        .get(episode..first)
        .is_some_and(|s| s.iter().any(is_invalid_token));
    if any_invalid {
        first = len;
    }
    let mut last = find_from(tokens, first, |t| {
        is_open_bracket_token(t) || (is_identified_token(t) && !is_part_token(t))
    });

    // Fall back to the first free range in corner brackets after episode,
    // e.g. `[Group] Title - Episode 「Episode Title」`.
    if first == len {
        first = find_from(tokens, episode, |t| {
            is_open_bracket_token(t) && t.value == "\u{300c}"
        });
        if first != len {
            first += 1;
        }
        last = find_from(tokens, first, |t| {
            is_close_bracket_token(t) && t.value == "\u{300d}"
        });
        if last == len {
            return (len, len);
        }
        if tokens
            .get(first..last)
            .is_some_and(|s| s.iter().any(is_identified_token))
        {
            return (len, len);
        }
    }

    (first, last)
}

pub(super) fn parse_episode_title(tokens: &mut [Token]) -> Option<Element> {
    let (first, last) = find_episode_title(tokens);
    if first >= last {
        return None;
    }

    let value =
        build_element_value(tokens.get(first..last)?, false, underscore_is_separator(tokens));
    // Avoid single-character episode titles.
    if value.chars().count() <= 1 {
        return None;
    }

    let position = tokens.get(first)?.position;
    for i in first..last {
        mark(tokens, i, ElementKind::EpisodeTitle);
    }

    Some(Element {
        kind: ElementKind::EpisodeTitle,
        value,
        position,
    })
}
