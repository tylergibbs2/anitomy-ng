// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Port of `include/anitomy/detail/token.hpp`: token data types and their
//! predicates.

use super::delimiter::is_dash;
use super::keyword::Keyword;
use crate::element::ElementKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum TokenKind {
    OpenBracket,
    CloseBracket,
    Delimiter,
    Keyword,
    #[default]
    Text,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct Token {
    pub kind: TokenKind,
    pub value: String,
    pub keyword: Option<Keyword>,
    pub element_kind: Option<ElementKind>,
    /// Index (in codepoints) in the input string.
    pub position: usize,
    /// Token is enclosed in brackets.
    pub is_enclosed: bool,
    /// All characters in `value` are digits.
    pub is_number: bool,
}

#[allow(dead_code)]
pub(crate) fn is_identified_token(token: &Token) -> bool {
    token.element_kind.is_some()
}

#[allow(dead_code)]
pub(crate) fn is_free_token(token: &Token) -> bool {
    matches!(token.kind, TokenKind::Text | TokenKind::Keyword) && token.element_kind.is_none()
}

#[allow(dead_code)]
pub(crate) fn is_open_bracket_token(token: &Token) -> bool {
    token.kind == TokenKind::OpenBracket
}

#[allow(dead_code)]
pub(crate) fn is_close_bracket_token(token: &Token) -> bool {
    token.kind == TokenKind::CloseBracket
}

#[allow(dead_code)]
pub(crate) fn is_bracket_token(token: &Token) -> bool {
    is_open_bracket_token(token) || is_close_bracket_token(token)
}

#[allow(dead_code)]
pub(crate) fn is_dash_token(token: &Token) -> bool {
    token.kind == TokenKind::Delimiter && token.value.chars().next().is_some_and(is_dash)
}

#[allow(dead_code)]
pub(crate) fn is_delimiter_token(token: &Token) -> bool {
    token.kind == TokenKind::Delimiter
}

#[allow(dead_code)]
pub(crate) fn is_not_delimiter_token(token: &Token) -> bool {
    token.kind != TokenKind::Delimiter
}

#[allow(dead_code)]
pub(crate) fn is_keyword_token(token: &Token) -> bool {
    token.kind == TokenKind::Keyword
}

#[allow(dead_code)]
pub(crate) fn is_text_token(token: &Token) -> bool {
    token.kind == TokenKind::Text
}

#[allow(dead_code)]
pub(crate) fn is_enclosed_token(token: &Token) -> bool {
    token.is_enclosed
}

#[allow(dead_code)]
pub(crate) fn is_numeric_token(token: &Token) -> bool {
    token.is_number
}
