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

/// Re-parse the filename component as authoritative, borrowing a missing title
/// or season from the directory components; inputs with no directory prefix are
/// unchanged.
pub(crate) fn parse_one(input: &str, options: Options) -> Vec<Element> {
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

    borrow_from_ancestors(&mut elements, &chars, dir_end.saturating_sub(1), options);

    elements
}

/// Fill a missing title/season from the directory components, nearest first.
/// They can come from different levels (`Show/Season 2/…`), so a component
/// yielding neither is skipped rather than ending the walk. Each is parsed
/// alone, so a UNC or drive-letter root can't glue itself into a title.
fn borrow_from_ancestors(
    elements: &mut Vec<Element>,
    chars: &[char],
    dir_last: usize,
    options: Options,
) {
    let mut want_title = !elements.iter().any(|e| e.kind == ElementKind::Title);
    let mut want_season =
        options.parse_season && !elements.iter().any(|e| e.kind == ElementKind::Season);

    let mut borrowed = false;
    for (start, end) in ancestor_components(chars, dir_last) {
        if !want_title && !want_season {
            break;
        }
        let Some(component) = chars.get(start..end) else {
            continue;
        };
        // Skips a parse per ancestor for the common season-less filename.
        if !want_title && !is_season_folder(component) {
            continue;
        }
        let component_input: String = component.iter().collect();
        for element in crate::parse(&component_input, options) {
            match element.kind {
                ElementKind::Title if want_title => want_title = false,
                ElementKind::Season if want_season => want_season = false,
                _ => continue,
            }
            elements.push(Element {
                position: element.position.saturating_add(start),
                ..element
            });
            borrowed = true;
        }
    }

    if borrowed {
        elements.sort_by_key(|e| e.position);
    }
}

/// Could this be a bare season folder (`Season 2`)? A digit is necessary; a
/// bracket means release tags, i.e. a show or batch folder.
fn is_season_folder(component: &[char]) -> bool {
    use crate::detail::bracket::{is_close_bracket, is_open_bracket};

    component.iter().any(char::is_ascii_digit)
        && !component
            .iter()
            .any(|&c| is_open_bracket(c) || is_close_bracket(c))
}

/// The separator-delimited components of `chars[..end]`, nearest first, as
/// `(start, end)` char ranges.
fn ancestor_components(chars: &[char], mut end: usize) -> Vec<(usize, usize)> {
    let mut bounds = Vec::new();
    loop {
        let start = chars
            .get(..end)
            .and_then(|s| s.iter().rposition(|&c| is_path_separator(c)))
            .map_or(0, |i| i.saturating_add(1));
        if start < end {
            bounds.push((start, end));
        }
        if start == 0 {
            return bounds;
        }
        end = start.saturating_sub(1);
    }
}

/// End (exclusive) of a real directory prefix, or `None`.
///
/// An absolute-path prefix (`C:\`, `\\server\`) splits at its last separator.
/// Otherwise the boundary is the rightmost separator whose trailing component
/// parses as a real filename (see [`looks_like_filename`]). Re-parsing the
/// candidate segment, rather than trusting the greedy title span, is what keeps
/// a `\` in `AC\DC` intact — the greedy parse absorbs a real separator and an
/// in-title one alike. [`continues_a_title`] is where that span still gets a say.
fn directory_boundary(chars: &[char], options: Options) -> Option<usize> {
    if has_absolute_windows_prefix(chars) {
        return chars
            .iter()
            .rposition(|&c| is_path_separator(c))
            .map(|i| i.saturating_add(1));
    }

    // Most inputs are a bare filename; leave before paying for the whole parse.
    if !chars.iter().any(|&c| is_path_separator(c)) {
        return None;
    }

    for i in (0..chars.len()).rev() {
        if !chars.get(i).is_some_and(|&c| is_path_separator(c)) {
            continue;
        }
        let prefix = chars.get(..i).unwrap_or_default();
        let tail = chars.get(i.saturating_add(1)..).unwrap_or_default();
        let tail_input: String = tail.iter().collect();
        let tail_elements = crate::parse(&tail_input, options);

        // The veto is only consulted for a separator that would otherwise be
        // taken, so most paths never pay for it.
        if looks_like_filename(&tail_elements, prefix, options)
            && !continues_a_title(&tail_elements, chars, i, prefix, options)
        {
            return Some(i.saturating_add(1));
        }
    }
    None
}

/// Does the greedy parse account for this separator as part of one title?
///
/// `Fate/Zero - 05 [720p].mkv` and `Anime/Show - 05 [720p].mkv` are the same
/// shape, so nothing local separates a franchise slash from a one-level path.
/// The tie-breaker is whether the greedy title is *exactly* the component
/// before the separator, the separator, and the tail's own title. That fails
/// once the tail carries anything the folder wouldn't (a group tag in
/// `Random Folder/[SubsPlease] Frieren`) or the path runs deeper.
///
/// Comparing parsed titles rather than raw text keeps `[Doki] Fate/stay night`
/// working. Equal titles either side are the folder-echo case, a real
/// directory, so they never veto.
fn continues_a_title(
    tail_elements: &[Element],
    chars: &[char],
    sep: usize,
    prefix: &[char],
    options: Options,
) -> bool {
    let Some(tail_title) = tail_elements.iter().find(|e| e.kind == ElementKind::Title) else {
        return false;
    };
    // For the veto to fire the title must run straight on from the separator,
    // so anything ahead of it in the tail (a group tag, most commonly) rules it
    // out — checked before parsing anything, since this is the common case.
    if tail_title.position != 0 {
        return false;
    }
    let Some(separator) = chars.get(sep) else {
        return false;
    };

    let component_start = prefix
        .iter()
        .rposition(|&c| is_path_separator(c))
        .map_or(0, |i| i.saturating_add(1));
    let component: &[char] = prefix.get(component_start..).unwrap_or_default();

    // Judged on the whole path, not just the two components either side: an
    // outer directory belongs in the greedy title too, and dropping it makes
    // the join match on nested paths it shouldn't (`Anime/Anime/Show - 05`).
    let whole: String = chars.iter().collect();
    let whole_elements = crate::parse(&whole, options);
    if !whole_elements
        .iter()
        .any(|e| e.kind == ElementKind::Episode)
    {
        return false;
    }
    let Some(whole_title) = whole_elements.iter().find(|e| e.kind == ElementKind::Title) else {
        return false;
    };

    let Some(expected) = whole_title
        .value
        .strip_suffix(&format!("{separator}{}", tail_title.value))
    else {
        return false;
    };

    // A folder restating the file's title is a real directory.
    if expected == tail_title.value {
        return false;
    }

    let component_input: String = component.iter().collect();
    crate::parse(&component_input, options)
        .iter()
        .any(|e| e.kind == ElementKind::Title && e.value == expected)
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
///
/// Signal (a) holds for nearly every real filename, so [`continues_a_title`]
/// runs first to veto a slash-title that merely carries metadata.
fn looks_like_filename(elements: &[Element], prefix: &[char], options: Options) -> bool {
    // (a) own release metadata.
    if elements.iter().any(|e| is_release_descriptor(e.kind)) {
        return true;
    }

    // (b) no title of its own.
    let Some(tail_title) = elements.iter().find(|e| e.kind == ElementKind::Title) else {
        return true;
    };

    // (c) title echoes one of the directory components.
    echoes_an_ancestor(&tail_title.value, prefix, options)
}

/// Does any directory component's own title restate `title`? Per component, not
/// the prefix as one string: `Attack on Titan/Season 2` parses as the single
/// title `Attack on Titan/Season` plus episode `2`, which echoes nothing.
fn echoes_an_ancestor(title: &str, prefix: &[char], options: Options) -> bool {
    ancestor_components(prefix, prefix.len())
        .into_iter()
        .filter_map(|(start, end)| prefix.get(start..end))
        .any(|component| {
            let component_input: String = component.iter().collect();
            crate::parse(&component_input, options)
                .iter()
                .any(|e| e.kind == ElementKind::Title && same_title(&e.value, title))
        })
}

/// Same show, ignoring delimiter normalization? The fold depends on which
/// delimiters the whole run uses, so `Ex-Arm` alone becomes `Ex Arm` while
/// `Ex-Arm 06.mkv` keeps the dash.
fn same_title(a: &str, b: &str) -> bool {
    let words = |s: &str| -> Vec<String> {
        s.split(|c: char| !c.is_alphanumeric())
            .filter(|w| !w.is_empty())
            .map(str::to_lowercase)
            .collect()
    };
    words(a) == words(b)
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
