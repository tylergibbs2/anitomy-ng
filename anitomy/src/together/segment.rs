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
    let chars: Vec<char> = input.chars().collect();

    let Some(dir_end) = directory_boundary(&chars, options) else {
        return crate::parse(input, options);
    };
    let Some(tail) = chars.get(dir_end..) else {
        return crate::parse(input, options);
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
/// An absolute-path prefix (`C:\`, `\\server\`) splits at its last separator.
/// Otherwise the boundary is the rightmost separator whose trailing component
/// parses as a real filename (see [`looks_like_filename`]). Deciding by
/// re-parsing the candidate segment — rather than trusting the greedy
/// whole-string parse's title span — is what lets a real folder be split off
/// while a `/` in `Fate/stay night` or a `\` in `AC\DC` is left alone: the
/// whole-string parse absorbs both a real separator and an in-title one into a
/// single title element, so its spans can't tell them apart.
fn directory_boundary(chars: &[char], options: Options) -> Option<usize> {
    if has_absolute_windows_prefix(chars) {
        return chars
            .iter()
            .rposition(|&c| is_path_separator(c))
            .map(|i| i.saturating_add(1));
    }

    for i in (0..chars.len()).rev() {
        if !chars.get(i).is_some_and(|&c| is_path_separator(c)) {
            continue;
        }
        let prefix = chars.get(..i).unwrap_or_default();
        let tail = chars.get(i.saturating_add(1)..).unwrap_or_default();
        if looks_like_filename(prefix, tail, options) {
            return Some(i.saturating_add(1));
        }
    }
    None
}

/// Does the component after a separator parse as a real filename, rather than as
/// the tail of a title that merely contains a slash (`Fate/stay night`)? A real
/// filename shows one of three signals; an in-title slash's tail shows none:
///
///   (a) it carries its own release metadata — a group, resolution, checksum,
///       season, … — that a lone title fragment would not;
///   (b) it has no title of its own (an episode-led name whose series title
///       lives in the parent folder, e.g. `05 - Episode.mkv`); or
///   (c) its title echoes the parent component (the folder restates the show,
///       e.g. `My Show/My Show - 01.mkv`).
fn looks_like_filename(prefix: &[char], tail: &[char], options: Options) -> bool {
    let tail_input: String = tail.iter().collect();
    let elements = crate::parse(&tail_input, options);

    // (a) own release metadata.
    if elements.iter().any(|e| is_release_descriptor(e.kind)) {
        return true;
    }

    // (b) no title of its own.
    let Some(tail_title) = elements.iter().find(|e| e.kind == ElementKind::Title) else {
        return true;
    };

    // (c) title echoes the parent component.
    let parent_input: String = prefix.iter().collect();
    crate::parse(&parent_input, options)
        .iter()
        .any(|e| e.kind == ElementKind::Title && e.value == tail_title.value)
}

/// A kind that marks a self-contained release — anything a lone title fragment
/// wouldn't carry. Title / episode / episode-title / extension are excluded
/// because an in-title slash's tail (`stay night - 01.mkv`) has exactly those.
fn is_release_descriptor(kind: ElementKind) -> bool {
    !matches!(
        kind,
        ElementKind::Title
            | ElementKind::Episode
            | ElementKind::EpisodeTitle
            | ElementKind::FileExtension
    )
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
