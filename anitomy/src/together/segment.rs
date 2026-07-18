// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Per-file path segmentation: strip a real directory prefix so a record
//! describes the file, not the folder.
//!
//! Recognizes both `/` and `\` on every platform; `std::path` is avoided since
//! the input may be a path from a different OS than the host.

use crate::element::{Element, ElementKind};
use crate::options::Options;

/// Re-parse the filename component as authoritative, borrowing only a missing
/// title from the parent folder; inputs with no directory prefix are unchanged.
pub(super) fn parse_one(input: &str, options: Options) -> Vec<Element> {
    let full = crate::parse(input, options);
    let chars: Vec<char> = input.chars().collect();

    let Some(dir_end) = directory_boundary(&chars, &full) else {
        return full;
    };
    let Some(tail) = chars.get(dir_end..) else {
        return full;
    };
    let filename: String = tail.iter().collect();

    // Shift filename positions back into the original string's coordinates.
    let mut elements = crate::parse(&filename, options);
    for element in &mut elements {
        element.position = element.position.saturating_add(dir_end);
    }

    // Borrow a missing title from the immediate parent component only, so a UNC
    // or drive-letter prefix can't glue itself into it.
    if !elements.iter().any(|e| e.kind == ElementKind::Title) {
        let parent_start = parent_component_start(&chars, dir_end);
        if let Some(parent) = chars.get(parent_start..dir_end.saturating_sub(1)) {
            let parent_input: String = parent.iter().collect();
            if let Some(title) = crate::parse(&parent_input, options)
                .into_iter()
                .find(|e| e.kind == ElementKind::Title)
            {
                elements.push(Element {
                    position: title.position.saturating_add(parent_start),
                    ..title
                });
                elements.sort_by_key(|e| e.position);
            }
        }
    }

    elements
}

/// End (exclusive) of a real directory prefix, or `None`.
///
/// An absolute-path prefix (`C:\`, `\\server\`) splits at its last separator;
/// otherwise the boundary is the last separator no element spans, which keeps a
/// `/` in `Fate/stay night` or a `\` in `AC\DC` from counting.
fn directory_boundary(chars: &[char], elements: &[Element]) -> Option<usize> {
    if has_absolute_windows_prefix(chars) {
        return chars
            .iter()
            .rposition(|&c| is_path_separator(c))
            .map(|i| i.saturating_add(1));
    }

    let mut boundary = None;
    for (i, &c) in chars.iter().enumerate() {
        if is_path_separator(c) && !spanned(elements, i) {
            boundary = Some(i.saturating_add(1));
        }
    }
    boundary
}

/// Start of the component before the boundary separator at `dir_end - 1`.
fn parent_component_start(chars: &[char], dir_end: usize) -> usize {
    let boundary_sep = dir_end.saturating_sub(1);
    let mut start = 0;
    for i in 0..boundary_sep {
        if chars.get(i).is_some_and(|&c| is_path_separator(c)) {
            start = i.saturating_add(1);
        }
    }
    start
}

fn is_path_separator(c: char) -> bool {
    matches!(c, '/' | '\\')
}

/// Does the input begin with a drive letter (`C:\`) or UNC root (`\\`)?
fn has_absolute_windows_prefix(chars: &[char]) -> bool {
    let unc = matches!((chars.first(), chars.get(1)), (Some('\\'), Some('\\')));
    let drive = matches!(
        (chars.first(), chars.get(1), chars.get(2)),
        (Some(c), Some(':'), Some(sep)) if c.is_ascii_alphabetic() && is_path_separator(*sep)
    );
    unc || drive
}

/// Does any element cover char index `i`?
fn spanned(elements: &[Element], i: usize) -> bool {
    elements.iter().any(|e| {
        let end = e.position.saturating_add(e.value.chars().count());
        e.position <= i && i < end
    })
}
