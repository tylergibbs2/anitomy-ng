// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Rust port of [erengy/anitomy](https://github.com/erengy/anitomy), an
//! anime video filename parser.
//!
//! The public API (`ElementKind`, `Element`, `Options`, `parse`) matches
//! upstream; [`detail`] holds the implementation, one module per upstream
//! C++ header. See `anitomy/tests/conformance.rs` for the harness that
//! checks output against upstream's and anitopy's fixture suites.
//!
//! Pure, safe Rust: no `unsafe`, no C dependencies, so it cross-compiles
//! and builds wheels anywhere `rustc` runs.
//!
//! This crate parses untrusted, arbitrary filenames, so it must never panic
//! on any input; a panic here would be a denial-of-service bug. The lints
//! below turn the usual panic
//! sources (`.unwrap()`, `.expect()`, `panic!`, `unreachable!`, direct
//! `s[i]` indexing) into hard errors under `cargo clippy` (see
//! `tests/no_panic.rs` for a runtime check of the same property, which
//! catches what lints can't, e.g. integer overflow). Prefer `.get(i)`,
//! pattern matching, `?`, and `.unwrap_or(...)` instead.

#![forbid(unsafe_code)]
#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::unreachable,
    clippy::indexing_slicing
)]

mod detail;
mod element;
mod options;
mod together;

pub use element::{Element, ElementKind, ParseElementKindError};
pub use options::Options;
pub use together::parse_together;

/// Port of the free function `anitomy::parse` in `include/anitomy.hpp`.
///
/// `input` must be UTF-8 encoded and should be in composed form (NFC/NFKC).
/// Returns parsed elements ordered by their position in `input`; there may
/// be multiple elements of the same kind.
pub fn parse(input: &str, options: Options) -> Vec<Element> {
    let tokens = detail::tokenizer::tokenize(input, &options);
    detail::parser::parse(tokens, &options)
}
