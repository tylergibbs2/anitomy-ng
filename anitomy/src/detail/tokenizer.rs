// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Port of `include/anitomy/detail/tokenizer.hpp`.
//!
//! Operates on `Vec<char>` (Unicode scalar values) rather than upstream's
//! UTF-32 conversion — Rust's `char` already is what upstream converts
//! into, so there's nothing to convert. `position`/token length are
//! counted in `char`s, matching upstream's UTF-32-codepoint counts.
//!
//! Never indexes directly (`chars[i]`): all access goes through
//! `slice::get`/iterator adapters, which can't panic regardless of how the
//! surrounding position arithmetic is edited later (see `tests/no_panic.rs`).

use super::bracket::{is_close_bracket, is_open_bracket, matching_close};
use super::delimiter::is_delimiter;
use super::keyword::{self, Keyword};
use super::token::{Token, TokenKind};
use crate::options::Options;

pub(crate) fn tokenize(input: &str, _options: &Options) -> Vec<Token> {
    let chars: Vec<char> = input.chars().collect();
    let mut pos = 0usize;
    let mut tokens = Vec::new();

    while let Some(token) = next_token(&chars, &mut pos) {
        tokens.push(token);
    }

    process_tokens(&mut tokens);
    tokens
}

fn is_text_char(ch: char) -> bool {
    !is_open_bracket(ch) && !is_close_bracket(ch) && !is_delimiter(ch)
}

/// A word boundary: a bracket or a delimiter (i.e. not a "text" character).
fn is_word_boundary(ch: char) -> bool {
    !is_text_char(ch)
}

fn next_token(chars: &[char], pos: &mut usize) -> Option<Token> {
    let ch = chars.get(*pos).copied()?;

    if is_open_bracket(ch) {
        return Some(Token {
            kind: TokenKind::OpenBracket,
            value: take(chars, pos, 1),
            ..Token::default()
        });
    }
    if is_close_bracket(ch) {
        return Some(Token {
            kind: TokenKind::CloseBracket,
            value: take(chars, pos, 1),
            ..Token::default()
        });
    }
    if is_delimiter(ch) {
        return Some(Token {
            kind: TokenKind::Delimiter,
            value: take(chars, pos, 1),
            ..Token::default()
        });
    }
    if let Some((value, keyword)) = take_keyword(chars, pos) {
        return Some(Token {
            kind: TokenKind::Keyword,
            value,
            keyword: Some(keyword),
            ..Token::default()
        });
    }

    Some(Token {
        kind: TokenKind::Text,
        value: take_text(chars, pos),
        ..Token::default()
    })
}

fn process_tokens(tokens: &mut [Token]) {
    // Enclosure is delimited by matching bracket pairs, not a nesting depth:
    // once an opening bracket is seen, only its matching close ends the
    // enclosed run. A stray or mismatched inner bracket (e.g. the second `[`
    // in `[[Group] Title`, or a `(` inside `[...]`) is just content and does
    // not open a new level, so an unbalanced run can't leave the rest of the
    // filename permanently "enclosed" and swallow the title.
    let mut expected_close: Option<char> = None;
    let mut position: usize = 0;

    for token in tokens.iter_mut() {
        match token.kind {
            TokenKind::OpenBracket if expected_close.is_none() => {
                expected_close = token.value.chars().next().and_then(matching_close);
            }
            TokenKind::CloseBracket if expected_close == token.value.chars().next() => {
                expected_close = None;
            }
            TokenKind::OpenBracket | TokenKind::CloseBracket => {}
            _ => token.is_enclosed = expected_close.is_some(),
        }

        token.position = position;
        position += token.value.chars().count();

        if token.kind == TokenKind::Text {
            token.is_number = token.value.chars().all(|c| c.is_ascii_digit());
        }
    }
}

/// Consumes and returns the next `n` chars from `pos` onward (fewer if
/// there aren't `n` left — never panics).
fn take(chars: &[char], pos: &mut usize, n: usize) -> String {
    let taken: String = chars.iter().skip(*pos).take(n).collect();
    *pos += taken.chars().count();
    taken
}

fn take_text(chars: &[char], pos: &mut usize) -> String {
    let n = chars
        .iter()
        .skip(*pos)
        .take_while(|&&c| is_text_char(c))
        .count();
    take(chars, pos, n)
}

fn take_keyword(chars: &[char], pos: &mut usize) -> Option<(String, Keyword)> {
    let key = find_key(chars, *pos)?;
    let n = key.chars().count();
    let keyword = keyword::get(&key)?;

    if !is_keyword_boundary(chars, pos.saturating_add(n), &keyword) {
        return None;
    }

    Some((take(chars, pos, n), keyword))
}

/// Longest prefix (starting at `start`) that is an exact keyword match,
/// searched incrementally so we can bail out as soon as no known keyword
/// could possibly match a longer prefix.
fn find_key(chars: &[char], start: usize) -> Option<String> {
    let remaining_len = chars.len().saturating_sub(start);
    let mut key = None;

    for n in 1..=remaining_len {
        let prefix: String = chars.iter().skip(start).take(n).collect();
        if keyword::get(&prefix).is_some() {
            key = Some(prefix.clone());
        }
        if !keyword::has_prefix(&prefix) {
            break;
        }
    }

    key
}

fn is_keyword_boundary(chars: &[char], start: usize, keyword: &Keyword) -> bool {
    if keyword.subword {
        return true;
    }
    let Some(next) = chars.get(start).copied() else {
        return true;
    };
    if is_word_boundary(next) {
        return true;
    }
    if keyword.prefix_for_number {
        return next.is_ascii_digit();
    }
    if keyword.prefix_for_other {
        return find_key(chars, start).is_some();
    }
    false
}
