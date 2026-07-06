// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Port of `include/anitomy/element.hpp` and the `ElementKind` half of
//! `include/anitomy/detail/format.hpp` (the `to_string`/`to_element_kind`
//! tables). Keep the `as_str` mapping in sync with upstream if it changes —
//! the conformance fixture suites (`tests/fixtures/*.json`) key on these
//! exact strings.

use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

impl ElementKind {
    pub fn as_str(self) -> &'static str {
        match self {
            ElementKind::AudioTerm => "audio_term",
            ElementKind::Device => "device",
            ElementKind::Episode => "episode",
            ElementKind::EpisodeTitle => "episode_title",
            ElementKind::FileChecksum => "file_checksum",
            ElementKind::FileExtension => "file_extension",
            ElementKind::Language => "language",
            ElementKind::Other => "other",
            ElementKind::Part => "part",
            ElementKind::ReleaseGroup => "release_group",
            ElementKind::ReleaseInformation => "release_information",
            ElementKind::ReleaseVersion => "release_version",
            ElementKind::Season => "season",
            ElementKind::Source => "source",
            ElementKind::Subtitles => "subtitles",
            ElementKind::Title => "title",
            ElementKind::Type => "type",
            ElementKind::VideoResolution => "video_resolution",
            ElementKind::VideoTerm => "video_term",
            ElementKind::Volume => "volume",
            ElementKind::Year => "year",
        }
    }
}

impl fmt::Display for ElementKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// No matching `ElementKind` for the given string.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParseElementKindError;

impl fmt::Display for ParseElementKindError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("not a valid ElementKind")
    }
}

impl std::error::Error for ParseElementKindError {}

impl FromStr for ElementKind {
    type Err = ParseElementKindError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "audio_term" => ElementKind::AudioTerm,
            "device" => ElementKind::Device,
            "episode" => ElementKind::Episode,
            "episode_title" => ElementKind::EpisodeTitle,
            "file_checksum" => ElementKind::FileChecksum,
            "file_extension" => ElementKind::FileExtension,
            "language" => ElementKind::Language,
            "other" => ElementKind::Other,
            "part" => ElementKind::Part,
            "release_group" => ElementKind::ReleaseGroup,
            "release_information" => ElementKind::ReleaseInformation,
            "release_version" => ElementKind::ReleaseVersion,
            "season" => ElementKind::Season,
            "source" => ElementKind::Source,
            "subtitles" => ElementKind::Subtitles,
            "title" => ElementKind::Title,
            "type" => ElementKind::Type,
            "video_resolution" => ElementKind::VideoResolution,
            "video_term" => ElementKind::VideoTerm,
            "volume" => ElementKind::Volume,
            "year" => ElementKind::Year,
            _ => return Err(ParseElementKindError),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Element {
    pub kind: ElementKind,
    pub value: String,
    /// Index (in UTF-32 codepoints, matching upstream) in the input string.
    pub position: usize,
}
