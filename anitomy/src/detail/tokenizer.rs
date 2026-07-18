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

pub(crate) fn tokenize<'a>(input: &'a str, _options: &Options) -> Vec<Token<'a>> {
    let chars: Vec<char> = input.chars().collect();
    // Byte offset of each char, plus a sentinel of `input.len()`, so a token
    // spanning char range `start..end` slices as `input[offsets[start]..offsets[end]]`.
    let mut offsets: Vec<usize> = input.char_indices().map(|(i, _)| i).collect();
    offsets.push(input.len());

    let mut pos = 0usize;
    let mut tokens = Vec::new();

    while let Some(token) = next_token(input, &chars, &offsets, &mut pos) {
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

fn next_token<'a>(
    input: &'a str,
    chars: &[char],
    offsets: &[usize],
    pos: &mut usize,
) -> Option<Token<'a>> {
    let ch = chars.get(*pos).copied()?;

    if is_open_bracket(ch) {
        return Some(Token {
            kind: TokenKind::OpenBracket,
            value: take(input, offsets, pos, 1),
            ..Token::default()
        });
    }
    if is_close_bracket(ch) {
        return Some(Token {
            kind: TokenKind::CloseBracket,
            value: take(input, offsets, pos, 1),
            ..Token::default()
        });
    }
    if is_delimiter(ch) {
        return Some(Token {
            kind: TokenKind::Delimiter,
            value: take(input, offsets, pos, 1),
            ..Token::default()
        });
    }
    if let Some((value, keyword)) = take_keyword(input, chars, offsets, pos) {
        return Some(Token {
            kind: TokenKind::Keyword,
            value,
            keyword: Some(keyword),
            ..Token::default()
        });
    }
    if let Some((value, keyword)) = take_composite_language(input, chars, offsets, pos) {
        return Some(Token {
            kind: TokenKind::Keyword,
            value,
            keyword: Some(keyword),
            ..Token::default()
        });
    }

    Some(Token {
        kind: TokenKind::Text,
        value: take_text(input, chars, offsets, pos),
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

/// Consumes the next `n` chars from `pos` onward (fewer if there aren't `n`
/// left) and returns them as a slice of `input` — never panics.
fn take<'a>(input: &'a str, offsets: &[usize], pos: &mut usize, n: usize) -> &'a str {
    let start = *pos;
    let end = start.saturating_add(n).min(offsets.len().saturating_sub(1));
    *pos = end;
    match (offsets.get(start), offsets.get(end)) {
        (Some(&b0), Some(&b1)) => input.get(b0..b1).unwrap_or_default(),
        _ => "",
    }
}

fn take_text<'a>(input: &'a str, chars: &[char], offsets: &[usize], pos: &mut usize) -> &'a str {
    let n = chars
        .iter()
        .skip(*pos)
        .take_while(|&&c| is_text_char(c))
        .count();
    take(input, offsets, pos, n)
}

fn take_keyword<'a>(
    input: &'a str,
    chars: &[char],
    offsets: &[usize],
    pos: &mut usize,
) -> Option<(&'a str, Keyword)> {
    let (n, keyword) = find_key(chars, *pos)?;

    if !is_keyword_boundary(chars, pos.saturating_add(n), &keyword) {
        return None;
    }

    Some((take(input, offsets, pos, n), keyword))
}

/// Matches a composite `<lang-code>+Sub/Dub` tag (e.g. `GerJapDub`) against the
/// maximal text run at `pos` — the same span `take_text` would take — so it
/// only matches a whole word. Runs after `take_keyword`, so real keywords win.
fn take_composite_language<'a>(
    input: &'a str,
    chars: &[char],
    offsets: &[usize],
    pos: &mut usize,
) -> Option<(&'a str, Keyword)> {
    let n = chars
        .iter()
        .skip(*pos)
        .take_while(|&&c| is_text_char(c))
        .count();
    // Every composite tag ends in `sub(s)`/`dub(s)`; skip the lookup otherwise.
    if !matches!(
        chars.get(*pos + n.checked_sub(1)?)?.to_ascii_lowercase(),
        'b' | 's'
    ) {
        return None;
    }
    let end = *pos + n;
    let run = input.get(*offsets.get(*pos)?..*offsets.get(end)?)?;
    let keyword = keyword::get_composite(run)?;
    *pos = end;
    Some((run, keyword))
}

/// Longest keyword match at `start`, as `(char length, keyword)`. Grows a
/// reused lowercased buffer one char at a time, bailing once no known keyword
/// shares the current prefix.
fn find_key(chars: &[char], start: usize) -> Option<(usize, Keyword)> {
    let mut best = None;
    let mut lower = String::new();

    for (i, &ch) in chars.iter().enumerate().skip(start) {
        lower.push(ch.to_ascii_lowercase());
        if let Some(keyword) = keyword::get_lower(&lower) {
            best = Some((i - start + 1, keyword));
        }
        if !keyword::has_prefix_lower(&lower) {
            break;
        }
    }

    best
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
