// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Port of `include/anitomy/detail/parser/title.hpp`.

use crate::detail::container::{find_from, find_prev_token, mark};
use crate::detail::element::{build_element_value, underscore_is_separator};
use crate::detail::token::{
    is_close_bracket_token, is_enclosed_token, is_free_token, is_identified_token,
    is_not_delimiter_token, is_open_bracket_token, Token,
};
use crate::element::{Element, ElementKind};

/// Scans forward from `first` for the end of the "second enclosed range"
/// fallback (`[Group][Title][Info]`), stopping at the first bracket token
/// that isn't part of a balanced, fully-contained `(...)` aside.
///
/// Skips over balanced `(...)` pairs so a parenthetical inside the title,
/// e.g. `[Group][Title (Aside) More][Info]`, doesn't truncate it early at
/// the aside's own `(`. Upstream stops at the first bracket of any kind and
/// has this bug.
fn find_title_fallback_end(tokens: &[Token], first: usize) -> usize {
    let len = tokens.len();
    let mut paren_depth = 0usize;

    for (offset, token) in tokens.iter().enumerate().skip(first) {
        if is_close_bracket_token(token) {
            if paren_depth > 0 {
                paren_depth -= 1;
                continue;
            }
            return offset;
        }
        if is_open_bracket_token(token) {
            if token.value == "(" {
                paren_depth += 1;
                continue;
            }
            if paren_depth == 0 {
                return offset;
            }
        }
    }

    len
}

/// Whether any token in `tokens[first..last)` contains an ASCII letter.
fn region_has_latin(tokens: &[Token], first: usize, last: usize) -> bool {
    tokens.get(first..last).is_some_and(|s| {
        s.iter()
            .any(|t| t.value.chars().any(|c| c.is_ascii_alphabetic()))
    })
}

fn find_title(tokens: &[Token]) -> (usize, usize) {
    let len = tokens.len();

    // Ignore filenames starting with episode number.
    if tokens
        .first()
        .is_some_and(|t| t.element_kind == Some(ElementKind::Episode))
    {
        return (len, len);
    }

    // A leading corner-bracket group `「…」` is a stylized title (Japanese
    // quotation marks), not a metadata bracket — take it whole, brackets
    // included (e.g. `「K」 Image …` -> title `「K」`). Returning early keeps the
    // closing `」` from being stripped as a trailing bracket below.
    if tokens
        .first()
        .is_some_and(|t| is_open_bracket_token(t) && t.value == "\u{300C}")
    {
        let close = find_from(tokens, 1, |t| {
            is_close_bracket_token(t) && t.value == "\u{300D}"
        });
        if close < len {
            return (0, close + 1);
        }
    }

    // Find the first free unenclosed range, e.g. `[Group] Title - Episode [Info]`.
    let mut first = find_from(tokens, 0, |t| is_free_token(t) && !is_enclosed_token(t));
    let mut last = find_from(tokens, first, is_identified_token);

    // Leading-junk prefix: a lone token before the release-group bracket
    // (`37 [Ruberia] Death Note`, `EvoBot.[Watakushi] Akuma no Riddle`) is a
    // stray uploader tag or episode number, not the title. When the first free
    // token is immediately followed by an enclosed, unidentified (group-like)
    // bracket and then more free text, skip past both so the title is the run
    // after the bracket — which also frees release_group to claim the bracket.
    if first < len {
        let bracket = find_from(tokens, first + 1, is_not_delimiter_token);
        if tokens.get(bracket).is_some_and(is_open_bracket_token) {
            let close = find_from(tokens, bracket, is_close_bracket_token);
            let group_like = close < len
                && tokens.get(bracket + 1..close).is_some_and(|s| {
                    !s.is_empty() && s.iter().all(|t| !is_identified_token(t))
                });
            if group_like {
                let title_start =
                    find_from(tokens, close + 1, |t| is_free_token(t) && !is_enclosed_token(t));
                if title_start < len {
                    first = title_start;
                    last = find_from(tokens, first, is_identified_token);
                }
            }
        }
    }

    // Fall back to the second enclosed range (assuming the first is the release group),
    // e.g. `[Group][Title][Info]`.
    if first == len {
        first = find_from(tokens, 0, is_close_bracket_token);
        first = find_from(tokens, first, is_free_token);
        last = find_title_fallback_end(tokens, first);

        // Common Chinese-fansub layout `[group][中文名][English Name][ep]`: the
        // first title candidate after the group is a CJK name with no Latin
        // letters, immediately followed by the romanized/English title in its
        // own bracket. Prefer the first later candidate that has Latin letters.
        if first != len && !region_has_latin(tokens, first, last) {
            let mut probe = last;
            loop {
                probe = find_from(tokens, probe, is_free_token);
                if probe == len {
                    break;
                }
                let probe_end = find_title_fallback_end(tokens, probe);
                if region_has_latin(tokens, probe, probe_end) {
                    first = probe;
                    last = probe_end;
                    break;
                }
                probe = probe_end;
            }
        }
    }

    // Allow filenames without a title.
    if first == len {
        return (len, len);
    }

    // Prevent titles with mismatched brackets, e.g. `Title (` -> `Title `, `Title [Info ` -> `Title `.
    if let Some(region) = tokens.get(first..last) {
        let open_brackets: Vec<usize> = region
            .iter()
            .enumerate()
            .filter(|(_, t)| is_open_bracket_token(t))
            .map(|(i, _)| first + i)
            .collect();
        if !open_brackets.is_empty() {
            let close_count = region.iter().filter(|t| is_close_bracket_token(t)).count();
            if close_count != open_brackets.len() {
                last = open_brackets.last().copied().unwrap_or(last);
            }
        }
    }

    // Prevent titles ending with brackets (except parentheses),
    // e.g. `Title [Group]` -> `Title `; `Title (TV)` is unchanged.
    if let Some(prev) = find_prev_token(tokens, last, is_not_delimiter_token) {
        let ends_in_non_paren_bracket = tokens
            .get(prev)
            .is_some_and(|t| is_close_bracket_token(t) && t.value != ")");
        if ends_in_non_paren_bracket {
            if let Some(open) = find_prev_token(tokens, prev, is_open_bracket_token) {
                last = open;
            }
        }
    }

    (first, last)
}

pub(super) fn parse_title(tokens: &mut [Token]) -> Option<Element> {
    let (first, last) = find_title(tokens);
    if first >= last {
        return None;
    }

    let value = build_element_value(
        tokens.get(first..last)?,
        false,
        underscore_is_separator(tokens),
    );
    if value.is_empty() {
        return None;
    }

    let position = tokens.get(first)?.position;
    for i in first..last {
        mark(tokens, i, ElementKind::Title);
    }

    Some(Element {
        kind: ElementKind::Title,
        value,
        position,
    })
}
