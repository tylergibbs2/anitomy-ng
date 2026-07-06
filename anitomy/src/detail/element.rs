// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Port of `include/anitomy/detail/element.hpp`.

use std::collections::HashSet;

use super::delimiter::is_space;
use super::token::{is_delimiter_token, is_not_delimiter_token, Token};
use crate::element::{Element, ElementKind};

pub(crate) fn element_from_token(
    kind: ElementKind,
    token: &Token,
    value: Option<&str>,
    position: Option<usize>,
) -> Element {
    Element {
        kind,
        value: value
            .map(str::to_string)
            .unwrap_or_else(|| token.value.clone()),
        position: position.unwrap_or(token.position),
    }
}

fn first_char(token: &Token) -> Option<char> {
    token.value.chars().next()
}

/// Joins `tokens` into a single string. Trailing delimiters are trimmed
/// (unless `keep_delimiters`), and delimiters within the run are turned
/// into spaces per upstream's heuristic (based on which delimiter
/// characters are actually used in this particular run of tokens).
pub(crate) fn build_element_value(tokens: &[Token], keep_delimiters: bool) -> String {
    let delimiters: HashSet<char> = tokens
        .iter()
        .filter(|t| is_delimiter_token(t))
        .filter_map(first_char)
        .collect();
    let has_single_delimiter = delimiters.len() == 1;
    let has_spaces = delimiters.iter().copied().any(is_space);
    let has_underscores = delimiters.contains(&'_');

    let is_transformable_delimiter = |token: &Token| -> bool {
        if keep_delimiters || is_not_delimiter_token(token) {
            return false;
        }
        let Some(ch) = first_char(token) else {
            return false;
        };
        if ch == ',' || ch == '&' || ch == '~' {
            return false;
        }
        if is_space(ch) || ch == '_' {
            return true;
        }
        if has_spaces || has_underscores {
            return false;
        }
        if ch == '.' {
            return true;
        }
        has_single_delimiter
    };

    let mut end = tokens.len();
    if !keep_delimiters {
        let mut prev_delimiter: Option<char> = None;
        while end > 0 {
            let Some(last) = tokens.get(end - 1) else {
                break;
            };
            if !is_delimiter_token(last) {
                break;
            }
            let Some(delim) = first_char(last) else { break };
            if delim == '~' {
                break;
            }
            if delim == '.' && prev_delimiter.is_some_and(is_space) {
                break;
            }
            prev_delimiter = Some(delim);
            end -= 1;
        }
    }

    let mut value = String::new();
    for token in tokens.get(..end).unwrap_or(&[]) {
        if is_transformable_delimiter(token) {
            value.push(' ');
        } else {
            value.push_str(&token.value);
        }
    }
    value
}
