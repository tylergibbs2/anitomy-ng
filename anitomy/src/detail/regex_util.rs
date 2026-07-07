// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! The one sanctioned home for regex `expect`s.
//!
//! Every sub-parser compiles fixed, literal patterns (never derived from
//! input) into a `OnceLock`. A compile failure is therefore a development-time
//! bug in a string constant, not something reachable from user input — but the
//! crate denies `clippy::expect_used` everywhere to guarantee it never panics
//! on a filename. Routing every pattern through [`compile`] keeps that single
//! justified `expect` in one audited place instead of scattering
//! `#[allow(clippy::expect_used)]` across a dozen call sites.

use regex::Regex;

/// Compiles a fixed, literal regex pattern. Panics only if `pattern` is not a
/// valid regex, which — since every caller passes a string constant — can only
/// happen at development time and is caught immediately by `tests/no_panic.rs`
/// (and every conformance test), never on real input.
#[allow(clippy::expect_used)]
pub(crate) fn compile(pattern: &str) -> Regex {
    Regex::new(pattern).expect("static regex pattern must be valid")
}
