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

/// Declares [`ElementKind`] and its string mapping from a single list, so
/// `as_str` and `FromStr` are always exact inverses — adding a variant here
/// updates both directions (and `Display`) at once, with no parallel tables to
/// drift. The strings are the snake_case names upstream and the fixture suites
/// key on; keep them in sync with upstream if it changes.
macro_rules! element_kinds {
    ($($variant:ident => $name:literal),+ $(,)?) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum ElementKind {
            $($variant),+
        }

        impl ElementKind {
            pub fn as_str(self) -> &'static str {
                match self {
                    $(ElementKind::$variant => $name),+
                }
            }
        }

        impl FromStr for ElementKind {
            type Err = ParseElementKindError;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                Ok(match s {
                    $($name => ElementKind::$variant,)+
                    _ => return Err(ParseElementKindError),
                })
            }
        }
    };
}

element_kinds! {
    AudioTerm => "audio_term",
    Device => "device",
    Episode => "episode",
    EpisodeTitle => "episode_title",
    FileChecksum => "file_checksum",
    FileExtension => "file_extension",
    Language => "language",
    Other => "other",
    Part => "part",
    ReleaseGroup => "release_group",
    ReleaseInformation => "release_information",
    ReleaseVersion => "release_version",
    Season => "season",
    Source => "source",
    Subtitles => "subtitles",
    Title => "title",
    Type => "type",
    VideoResolution => "video_resolution",
    VideoTerm => "video_term",
    Volume => "volume",
    Year => "year",
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Element {
    pub kind: ElementKind,
    pub value: String,
    /// Index (in UTF-32 codepoints, matching upstream) in the input string.
    pub position: usize,
}
