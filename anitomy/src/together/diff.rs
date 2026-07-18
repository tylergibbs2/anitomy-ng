// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Cross-file differential: correct each parse against what varies across the
//! set, working in `char`s so positions match those the parser emits.

use crate::element::{Element, ElementKind};

/// Reconcile parses against their differential in place: recover the episode
/// from the sole numeric varying span, or leave them untouched if it's absent
/// or ambiguous.
pub(super) fn reconcile(results: &mut [Vec<Element>], chars: &[Vec<char>]) {
    let candidates: Vec<Region> = variable_regions(chars)
        .into_iter()
        .filter(|region| is_episode_candidate(region, results, chars))
        .collect();

    let region = match candidates.as_slice() {
        [only] => only,
        _ => return,
    };

    let variants: Vec<String> = chars.iter().map(|cs| region.slice(cs)).collect();
    for (result, variant) in results.iter_mut().zip(&variants) {
        apply_episode(result, region.prefix, variant);
    }
    fill_missing_titles(results);
}

/// A span is the episode only if every member's substring there is numeric and
/// no member already typed it as something else (a `720` inside `720p`, etc.).
fn is_episode_candidate(region: &Region, results: &[Vec<Element>], chars: &[Vec<char>]) -> bool {
    let numeric = chars
        .iter()
        .map(|cs| region.slice(cs))
        .all(|v| !v.is_empty() && v.chars().all(|c| c.is_ascii_digit()));
    if !numeric {
        return false;
    }

    let start = region.prefix;
    let claimed = results.iter().zip(chars).any(|(result, cs)| {
        let end = cs.len().saturating_sub(region.suffix);
        result
            .iter()
            .any(|e| e.kind != ElementKind::Episode && overlaps(e, start, end))
    });
    !claimed
}

/// Does `element` occupy any char in `[start, end)`?
fn overlaps(element: &Element, start: usize, end: usize) -> bool {
    let e_start = element.position;
    let e_end = e_start.saturating_add(element.value.chars().count());
    e_start < end && start < e_end
}

/// A span in which the members differ; each member's substring is
/// `chars[prefix .. len - suffix]`.
struct Region {
    prefix: usize,
    suffix: usize,
}

impl Region {
    fn slice(&self, cs: &[char]) -> String {
        let end = cs.len().saturating_sub(self.suffix);
        cs.get(self.prefix..end)
            .map(|s| s.iter().collect())
            .unwrap_or_default()
    }
}

/// Every span in which the members differ: a column-wise diff for equal-length
/// members (separating e.g. episode from checksum), else the single outer span;
/// each widened over shared edge digits so `05`/`04` isn't cut to `5`/`4`.
fn variable_regions(chars: &[Vec<char>]) -> Vec<Region> {
    let equal_len = chars
        .first()
        .map(|first| chars.iter().all(|cs| cs.len() == first.len()));

    let raw: Vec<Region> = match equal_len {
        Some(true) => column_regions(chars),
        _ => single_region(chars).into_iter().collect(),
    };

    raw.into_iter()
        .map(|r| widen_over_digits(chars, r))
        .collect()
}

/// Each maximal run of columns at which not all equal-length members agree.
fn column_regions(chars: &[Vec<char>]) -> Vec<Region> {
    let len = chars.first().map(Vec::len).unwrap_or(0);
    let mut regions = Vec::new();
    let mut col = 0usize;
    while col < len {
        if !column_varies(chars, col) {
            col += 1;
            continue;
        }
        let start = col;
        while col < len && column_varies(chars, col) {
            col += 1;
        }
        regions.push(Region {
            prefix: start,
            suffix: len.saturating_sub(col),
        });
    }
    regions
}

fn column_varies(chars: &[Vec<char>], col: usize) -> bool {
    let head = chars.first().and_then(|cs| cs.get(col));
    chars.iter().any(|cs| cs.get(col) != head)
}

/// The span between the longest common prefix and suffix; `None` if identical.
fn single_region(chars: &[Vec<char>]) -> Option<Region> {
    let min_len = chars.iter().map(|c| c.len()).min()?;

    let mut prefix = 0usize;
    while prefix < min_len {
        let head = *chars.first()?.get(prefix)?;
        if chars.iter().any(|cs| cs.get(prefix) != Some(&head)) {
            break;
        }
        prefix += 1;
    }

    if chars.iter().all(|cs| cs.len() == prefix) {
        return None;
    }

    let mut suffix = 0usize;
    while prefix.saturating_add(suffix) < min_len {
        let Some(tail) = tail_char(chars.first()?, suffix) else {
            break;
        };
        if chars.iter().any(|cs| tail_char(cs, suffix) != Some(tail)) {
            break;
        }
        suffix += 1;
    }

    Some(Region { prefix, suffix })
}

/// Grow a region over digits shared by all members at its edges.
fn widen_over_digits(chars: &[Vec<char>], mut region: Region) -> Region {
    let Some(first) = chars.first() else {
        return region;
    };

    while region.prefix > 0
        && first
            .get(region.prefix.saturating_sub(1))
            .is_some_and(is_digit)
    {
        region.prefix -= 1;
    }
    while region.suffix > 0 {
        let idx = first.len().saturating_sub(region.suffix);
        if first.get(idx).is_some_and(is_digit) {
            region.suffix -= 1;
        } else {
            break;
        }
    }
    region
}

/// The `n`-th char from the end of `cs` (`n == 0` is the last).
fn tail_char(cs: &[char], n: usize) -> Option<char> {
    let idx = cs.len().checked_sub(1)?.checked_sub(n)?;
    cs.get(idx).copied()
}

fn is_digit(c: &char) -> bool {
    c.is_ascii_digit()
}

/// Set this member's episode to the varying span, dropping any other `Episode`
/// (a directory range the single-file parse mistook for one).
fn apply_episode(result: &mut Vec<Element>, start: usize, variant: &str) {
    let end = start.saturating_add(variant.chars().count());
    result.retain(|e| e.kind != ElementKind::Episode || (e.position >= start && e.position < end));

    if !result
        .iter()
        .any(|e| e.kind == ElementKind::Episode && e.position == start)
    {
        result.push(Element {
            kind: ElementKind::Episode,
            value: variant.to_string(),
            position: start,
        });
    }

    result.sort_by_key(|e| e.position);
}

/// Give every title-less member the batch-consensus title.
fn fill_missing_titles(results: &mut [Vec<Element>]) {
    let Some((value, position)) = consensus_title(results) else {
        return;
    };
    for result in results.iter_mut() {
        if !result.iter().any(|e| e.kind == ElementKind::Title) {
            result.push(Element {
                kind: ElementKind::Title,
                value: value.clone(),
                position,
            });
            result.sort_by_key(|e| e.position);
        }
    }
}

/// The most common title value across members, with a representative position.
fn consensus_title(results: &[Vec<Element>]) -> Option<(String, usize)> {
    let mut counts: Vec<(String, usize, usize)> = Vec::new();
    for result in results {
        if let Some(e) = result.iter().find(|e| e.kind == ElementKind::Title) {
            match counts.iter_mut().find(|(v, _, _)| *v == e.value) {
                Some(entry) => entry.1 += 1,
                None => counts.push((e.value.clone(), 1, e.position)),
            }
        }
    }
    counts
        .into_iter()
        .max_by_key(|(_, count, _)| *count)
        .map(|(value, _, position)| (value, position))
}
