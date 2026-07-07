// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Port of `include/anitomy/detail/parser/release_group.hpp`.
//!
//! The upstream recursion (`find_release_group` calling itself on a
//! shrinking suffix) is tail recursion, so it's a loop here instead;
//! indices stay absolute into the original `tokens`. But upstream's
//! `find_prev_token(tokens, first, pred)` call inside the recursive
//! function *is* bounded to the current recursive call's span — C++
//! `std::span::begin()` shifts with each recursive slice, and
//! `find_prev_token` explicitly refuses to look before `container.begin()`
//! — so the "is this range preceded by something other than its own open
//! bracket" check must only look back as far as `search_start`, not to the
//! start of the whole file. (The *fallback* branch's plain `std::prev`,
//! unlike `find_prev_token`, isn't bounded this way — it's raw pointer
//! arithmetic on the shared backing storage — so that one stays unbounded.)

use crate::detail::container::{find_from, find_prev_token, mark};
use crate::detail::element::{build_element_value, underscore_is_separator};
use crate::detail::token::{
    is_close_bracket_token, is_dash_token, is_delimiter_token, is_free_token, is_identified_token,
    is_not_delimiter_token, is_open_bracket_token, Token,
};
use crate::element::{Element, ElementKind};

/// Returns (first, last) token indices (last exclusive), or `(len, len)` if
/// no release group span was found.
fn find_release_group(tokens: &[Token]) -> (usize, usize) {
    let len = tokens.len();
    let mut search_start = 0usize;
    // All valid enclosed candidates, in order. Upstream returns the first; we
    // collect them so the trailing-bracket convention below can pick the last.
    let mut candidates: Vec<(usize, usize)> = Vec::new();

    loop {
        // Find the first enclosed unidentified range, e.g. `[Group] Title - Episode [Info]`.
        let first = find_from(tokens, search_start, |t| {
            t.is_enclosed && !is_identified_token(t)
        });
        let last = if first == len {
            len
        } else {
            find_from(tokens, first, |t| {
                is_close_bracket_token(t) || is_identified_token(t)
            })
        };

        if first != len {
            // Skip if the range contains other tokens. Bounded to [search_start, first) —
            // see module docs.
            let blocked_by_prev = tokens
                .get(search_start..first)
                .and_then(|s| s.iter().rposition(is_not_delimiter_token))
                .is_some_and(|rel| {
                    !tokens
                        .get(search_start + rel)
                        .is_some_and(is_open_bracket_token)
                });
            if blocked_by_prev {
                search_start = last;
                continue;
            }
            if last != len && !tokens.get(last).is_some_and(is_close_bracket_token) {
                search_start = last;
                continue;
            }
            candidates.push((first, last));
            search_start = last;
            continue;
        }

        // Enclosed candidates exhausted. When the title is unenclosed and sits
        // before the first candidate bracket (`Title [x] [y] [Group]`, e.g.
        // `Noein_[01_of_24]_[ru_jp]_[bodlerov_&_torrents_ru]`), the trailing
        // brackets are metadata and the group is the last one — not the first
        // unidentified bracket upstream would greedily take. In the standard
        // `[Group] Title` layout the group precedes the title, so this leaves
        // it as the first candidate.
        if let Some(&(cf, cl)) = candidates.first() {
            let title_precedes = tokens.get(cf).is_some_and(|c| {
                tokens.iter().any(|t| {
                    t.element_kind == Some(ElementKind::Title)
                        && !t.is_enclosed
                        && t.position < c.position
                })
            });
            return if title_precedes {
                candidates.last().copied().unwrap_or((cf, cl))
            } else {
                (cf, cl)
            };
        }

        // Fall back to the last token before file extension, e.g. `Title.Episode.Info-Group.mkv`.
        let region = tokens.get(search_start..).unwrap_or(&[]);
        let fallback = find_prev_token(region, region.len(), |t| {
            t.element_kind != Some(ElementKind::FileExtension) && is_not_delimiter_token(t)
        });
        if let Some(local_idx) = fallback {
            let idx = search_start + local_idx;
            if idx > 0 && tokens.get(idx).is_some_and(is_free_token) {
                let prev_idx = idx - 1;
                if tokens
                    .get(prev_idx)
                    .is_some_and(|t| is_delimiter_token(t) && is_dash_token(t))
                {
                    return (idx, idx + 1);
                }
            }
        }

        // Backward peel (forward×backward ensemble): the primary fallback above
        // stops at the last non-extension token, which is the checksum bracket
        // in `…x264-CTR.[CRC32].mkv`, so it never reaches the group. Peel past
        // all trailing metadata — extension, checksum, bracketed tags are
        // identified or enclosed — to the last *free, unenclosed* token; if a
        // dash precedes it, that is the scene-style `-GROUP` tail.
        let peeled = find_prev_token(region, region.len(), |t| {
            is_free_token(t) && !t.is_enclosed
        });
        if let Some(local_idx) = peeled {
            let idx = search_start + local_idx;
            if idx > 0
                && tokens
                    .get(idx - 1)
                    .is_some_and(|t| is_delimiter_token(t) && is_dash_token(t))
            {
                return (idx, idx + 1);
            }
        }
        return (len, len);
    }
}

pub(super) fn parse_release_group(tokens: &mut [Token]) -> Option<Element> {
    let (first, last) = find_release_group(tokens);
    if first >= last {
        return None;
    }

    let value = build_element_value(
        tokens.get(first..last)?,
        true,
        underscore_is_separator(tokens),
    );
    if value.is_empty() {
        return None;
    }

    let position = tokens.get(first)?.position;
    for i in first..last {
        mark(tokens, i, ElementKind::ReleaseGroup);
    }

    Some(Element {
        kind: ElementKind::ReleaseGroup,
        value,
        position,
    })
}
