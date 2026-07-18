// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! WebAssembly / npm bindings for anitomy-ng.
//!
//! Mirrors the typed surface of the Python bindings: a single
//! `parse(filename, options?) -> Element[]`, where `ElementKind` is a
//! snake_case string union and `Options` is a partial object of booleans. The
//! TypeScript types are derived from these structs via `tsify`.

use anitomy_ng::Options as CoreOptions;
use serde::{Deserialize, Serialize};
use tsify_next::Tsify;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

/// Declares the wasm-facing [`ElementKind`] mirror and its `From` conversion
/// off the core enum from a single variant list. `#[serde(rename_all)]` handles
/// the snake_case strings, so the variants only need naming once; keep the list
/// aligned with `anitomy_ng::ElementKind`.
macro_rules! element_kind_bridge {
    ($($variant:ident),+ $(,)?) => {
        /// The kind of a parsed [`Element`], serialized as the snake_case
        /// strings used across every anitomy-ng binding.
        #[derive(Serialize, Tsify)]
        #[tsify(into_wasm_abi)]
        #[serde(rename_all = "snake_case")]
        pub enum ElementKind {
            $($variant),+
        }

        impl From<anitomy_ng::ElementKind> for ElementKind {
            fn from(kind: anitomy_ng::ElementKind) -> Self {
                use anitomy_ng::ElementKind as K;
                match kind {
                    $(K::$variant => Self::$variant),+
                }
            }
        }
    };
}

element_kind_bridge! {
    AudioTerm,
    Device,
    Episode,
    EpisodeTitle,
    FileChecksum,
    FileExtension,
    Language,
    Other,
    Part,
    ReleaseGroup,
    ReleaseInformation,
    ReleaseVersion,
    Season,
    Source,
    Subtitles,
    Title,
    Type,
    VideoResolution,
    VideoTerm,
    Volume,
    Year,
}

/// One parsed element: its [`ElementKind`], the matched substring, and the
/// element's codepoint position in the input.
#[derive(Serialize, Tsify)]
#[tsify(into_wasm_abi)]
pub struct Element {
    pub kind: ElementKind,
    pub value: String,
    pub position: usize,
}

/// Declares the wasm-facing [`Options`] mirror and its round-trip conversions
/// with `CoreOptions` from a single field list, so the struct, `Default`, and
/// both `From` impls can't drift apart. Keep the list aligned with
/// `anitomy_ng::Options`.
macro_rules! options_bridge {
    ($($field:ident),+ $(,)?) => {
        /// Which element kinds to extract. Every field defaults to `true`; pass
        /// a partial object to disable specific kinds, e.g. `{ parse_title: false }`.
        #[derive(Deserialize, Tsify)]
        #[tsify(from_wasm_abi)]
        #[serde(default)]
        pub struct Options {
            $(pub $field: bool),+
        }

        impl Default for Options {
            fn default() -> Self {
                // Same defaults as the core crate (all enabled). `#[serde(default)]`
                // above uses this to fill any fields omitted by the caller.
                CoreOptions::default().into()
            }
        }

        impl From<CoreOptions> for Options {
            fn from(o: CoreOptions) -> Self {
                Self { $($field: o.$field),+ }
            }
        }

        impl From<Options> for CoreOptions {
            fn from(o: Options) -> Self {
                CoreOptions { $($field: o.$field),+ }
            }
        }
    };
}

options_bridge! {
    parse_episode,
    parse_episode_title,
    parse_file_checksum,
    parse_file_extension,
    parse_part,
    parse_release_group,
    parse_season,
    parse_title,
    parse_video_resolution,
    parse_year,
}

// Give the return values precise TS types (`Element[]`, `Element[][]`) instead
// of `any`, while still serializing through serde-wasm-bindgen.
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "Element[]")]
    pub type Elements;

    #[wasm_bindgen(typescript_type = "Element[][]")]
    pub type ElementsTogether;
}

/// Parse an anime video filename into its elements, ordered by position.
///
/// `options` is optional; omit it (or pass a partial object) to use the
/// defaults. There may be multiple elements of the same kind (e.g. an episode
/// range yields two `episode` elements).
#[wasm_bindgen]
pub fn parse(filename: &str, options: Option<Options>) -> Result<Elements, JsValue> {
    let opts = options.map(CoreOptions::from).unwrap_or_default();
    let elements: Vec<Element> = anitomy_ng::parse(filename, opts)
        .into_iter()
        .map(|el| Element {
            kind: el.kind.into(),
            value: el.value,
            position: el.position,
        })
        .collect();
    let value = serde_wasm_bindgen::to_value(&elements)?;
    Ok(value.unchecked_into())
}

/// Parse a set of related filenames together, returning one `Element[]` per
/// input in the same order (result `i` is for `filenames[i]`).
///
/// The shared context resolves ambiguities a single filename can't — e.g. a
/// directory batch range vs. the real per-file episode, or a series title that
/// lives only in a parent folder. `options` is optional, as in `parse`.
#[wasm_bindgen]
pub fn parse_together(
    filenames: Vec<String>,
    options: Option<Options>,
) -> Result<ElementsTogether, JsValue> {
    let opts = options.map(CoreOptions::from).unwrap_or_default();
    let refs: Vec<&str> = filenames.iter().map(String::as_str).collect();
    let batch: Vec<Vec<Element>> = anitomy_ng::parse_together(&refs, opts)
        .into_iter()
        .map(|elements| {
            elements
                .into_iter()
                .map(|el| Element {
                    kind: el.kind.into(),
                    value: el.value,
                    position: el.position,
                })
                .collect()
        })
        .collect();
    let value = serde_wasm_bindgen::to_value(&batch)?;
    Ok(value.unchecked_into())
}
