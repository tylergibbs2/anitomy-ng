// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Parse a set of related filenames together, using what is invariant across
//! the set to disambiguate what a single filename cannot.
//!
//! Each input is parsed on its own, then two passes run: [`segment`] suppresses
//! per-file directory noise, and [`diff`] reconciles each parse against what
//! varies across the set.

mod diff;
mod segment;

use crate::element::Element;
use crate::options::Options;

/// Parse a single filename that may carry a directory prefix, so the result
/// describes the file rather than the folder.
///
/// [`parse`](crate::parse) treats its input as one flat string, matching
/// upstream, so a path leaves separators and duplicated folder text in the
/// title. This strips a real directory prefix first and recovers a title that
/// lives only in the parent folder. Both `/` and `\` work on every platform,
/// including UNC and drive-letter roots.
///
/// Without a directory prefix this is exactly [`parse`](crate::parse), and a
/// separator inside a title (`Fate/stay night`) is left alone.
///
/// ```
/// let elements = anitomy_ng::parse_path(
///     "Frieren (01-12) [Batch]/Frieren - 05 [1080p].mkv",
///     anitomy_ng::Options::default(),
/// );
/// let title = elements.iter().find(|e| e.kind == anitomy_ng::ElementKind::Title);
/// assert_eq!(title.map(|e| e.value.as_str()), Some("Frieren"));
/// ```
///
/// For a set of related files, prefer [`parse_together`], which additionally
/// uses what varies across the set.
pub fn parse_path(input: &str, options: Options) -> Vec<Element> {
    segment::parse_one(input, options)
}

/// Parse related filenames together, order-preserving (result `i` is for
/// `inputs[i]`); an unrelated or single-item list is left as its per-file parse.
pub fn parse_together(inputs: &[&str], options: Options) -> Vec<Vec<Element>> {
    let mut results: Vec<Vec<Element>> = inputs
        .iter()
        .map(|s| segment::parse_one(s, options))
        .collect();

    // The cross-file differential needs at least two members to have any signal.
    if inputs.len() < 2 {
        return results;
    }

    // `char`s so positions line up with the codepoint-based ones the parser emits.
    let chars: Vec<Vec<char>> = inputs.iter().map(|s| s.chars().collect()).collect();
    diff::reconcile(&mut results, &chars);

    results
}
