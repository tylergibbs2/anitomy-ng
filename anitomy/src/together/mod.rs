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
