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

/// Whether `_` acts as a word separator in this file, i.e. it is at least as
/// common a delimiter as the space. When space is the more common delimiter,
/// an `_` inside an element is a literal character rather than a separator
/// (e.g. `Data_01_Login`, `NieR-Automata Ver1_1a`) and must not be folded to
/// a space; when `_` dominates (e.g. `bodlerov_&_torrents_ru`,
/// `Howl's_Moving_Castle`) it is the separator and folds like any other.
pub(crate) fn underscore_is_separator(tokens: &[Token]) -> bool {
    let mut spaces = 0usize;
    let mut underscores = 0usize;
    for ch in tokens
        .iter()
        .filter(|t| is_delimiter_token(t))
        .filter_map(first_char)
    {
        if is_space(ch) {
            spaces += 1;
        } else if ch == '_' {
            underscores += 1;
        }
    }
    underscores >= spaces
}

/// Joins `tokens` into a single string. Trailing delimiters are trimmed
/// (unless `keep_delimiters`), and delimiters within the run are turned
/// into spaces per upstream's heuristic (based on which delimiter
/// characters are actually used in this particular run of tokens).
/// `file_has_space` is the whole-file signal from `file_uses_space_delimiter`.
pub(crate) fn build_element_value(
    tokens: &[Token],
    keep_delimiters: bool,
    underscore_separator: bool,
) -> String {
    let delimiters: HashSet<char> = tokens
        .iter()
        .filter(|t| is_delimiter_token(t))
        .filter_map(first_char)
        .collect();
    let has_single_delimiter = delimiters.len() == 1;
    let has_spaces = delimiters.iter().copied().any(is_space);
    let has_underscores = delimiters.contains(&'_');
    // A `&` in a kept-delimiter run (a release group) marks a collaboration of
    // teams (`A_&_B_&_C`), where the `_` are word separators that should fold
    // to spaces. Without a `&`, an `_` in a group is part of one stylized name
    // (`Black_Sheep`, `Seto_Otaku`) and stays literal.
    let has_ampersand = delimiters.contains(&'&');

    let is_transformable_delimiter = |token: &Token| -> bool {
        if is_not_delimiter_token(token) {
            return false;
        }
        let Some(ch) = first_char(token) else {
            return false;
        };
        // The file's `_` separator folds to a space even when delimiters are
        // otherwise kept (release groups) — but only in a collaboration name
        // (`bodlerov_&_torrents_ru` -> `bodlerov & torrents ru`); a lone
        // stylized group name keeps it (`Black_Sheep`), as does a dash in
        // `UTW-THORA`.
        if ch == '_' {
            return underscore_separator && (!keep_delimiters || has_ampersand);
        }
        if keep_delimiters {
            return false;
        }
        if ch == ',' || ch == '&' || ch == '~' {
            return false;
        }
        if is_space(ch) {
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

    // A run of 3+ trailing dots is a canonical ellipsis (`I Am...`), not a
    // separator run. The trim above dropped all of them; restore a single
    // canonical `...` (one of the dots may have been the field separator, so
    // the run length isn't preserved).
    let trailing_ellipsis = !keep_delimiters
        && tokens.get(end..).map_or(0, |s| {
            s.iter().take_while(|t| first_char(t) == Some('.')).count()
        }) >= 3;

    let mut value = String::new();
    for token in tokens.get(..end).unwrap_or(&[]) {
        if is_transformable_delimiter(token) {
            value.push(' ');
        } else {
            value.push_str(&token.value);
        }
    }
    if trailing_ellipsis {
        value.push_str("...");
    }
    value
}
