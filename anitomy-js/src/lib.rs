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

/// The kind of a parsed [`Element`], serialized as the snake_case strings used
/// across every anitomy-ng binding.
#[derive(Serialize, Tsify)]
#[tsify(into_wasm_abi)]
#[serde(rename_all = "snake_case")]
pub enum ElementKind {
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

impl From<anitomy_ng::ElementKind> for ElementKind {
    fn from(kind: anitomy_ng::ElementKind) -> Self {
        use anitomy_ng::ElementKind as K;
        match kind {
            K::AudioTerm => Self::AudioTerm,
            K::Device => Self::Device,
            K::Episode => Self::Episode,
            K::EpisodeTitle => Self::EpisodeTitle,
            K::FileChecksum => Self::FileChecksum,
            K::FileExtension => Self::FileExtension,
            K::Language => Self::Language,
            K::Other => Self::Other,
            K::Part => Self::Part,
            K::ReleaseGroup => Self::ReleaseGroup,
            K::ReleaseInformation => Self::ReleaseInformation,
            K::ReleaseVersion => Self::ReleaseVersion,
            K::Season => Self::Season,
            K::Source => Self::Source,
            K::Subtitles => Self::Subtitles,
            K::Title => Self::Title,
            K::Type => Self::Type,
            K::VideoResolution => Self::VideoResolution,
            K::VideoTerm => Self::VideoTerm,
            K::Volume => Self::Volume,
            K::Year => Self::Year,
        }
    }
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

/// Which element kinds to extract. Every field defaults to `true`; pass a
/// partial object to disable specific kinds, e.g. `{ parse_title: false }`.
#[derive(Deserialize, Tsify)]
#[tsify(from_wasm_abi)]
#[serde(default)]
pub struct Options {
    pub parse_episode: bool,
    pub parse_episode_title: bool,
    pub parse_file_checksum: bool,
    pub parse_file_extension: bool,
    pub parse_part: bool,
    pub parse_release_group: bool,
    pub parse_season: bool,
    pub parse_title: bool,
    pub parse_video_resolution: bool,
    pub parse_year: bool,
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
        Self {
            parse_episode: o.parse_episode,
            parse_episode_title: o.parse_episode_title,
            parse_file_checksum: o.parse_file_checksum,
            parse_file_extension: o.parse_file_extension,
            parse_part: o.parse_part,
            parse_release_group: o.parse_release_group,
            parse_season: o.parse_season,
            parse_title: o.parse_title,
            parse_video_resolution: o.parse_video_resolution,
            parse_year: o.parse_year,
        }
    }
}

impl From<Options> for CoreOptions {
    fn from(o: Options) -> Self {
        CoreOptions {
            parse_episode: o.parse_episode,
            parse_episode_title: o.parse_episode_title,
            parse_file_checksum: o.parse_file_checksum,
            parse_file_extension: o.parse_file_extension,
            parse_part: o.parse_part,
            parse_release_group: o.parse_release_group,
            parse_season: o.parse_season,
            parse_title: o.parse_title,
            parse_video_resolution: o.parse_video_resolution,
            parse_year: o.parse_year,
        }
    }
}

// Give the return value a precise TS type (`Element[]`) instead of `any`, while
// still serializing through serde-wasm-bindgen.
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "Element[]")]
    pub type Elements;
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
