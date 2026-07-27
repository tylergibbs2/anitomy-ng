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
            /// Every variant, in declaration order.
            pub const ALL: &'static [ElementKind] = &[$(ElementKind::$variant),+];

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
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Element {
    pub kind: ElementKind,
    pub value: String,
    /// Index (in UTF-32 codepoints, matching upstream) in the input string.
    pub position: usize,
}

/// Serialized as the same snake_case name `as_str`/`FromStr` use, so the wire
/// format can't drift from the fixture suites' keys.
#[cfg(feature = "serde")]
impl serde::Serialize for ElementKind {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(self.as_str())
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for ElementKind {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = <&str as serde::Deserialize>::deserialize(d)?;
        s.parse()
            .map_err(|_| serde::de::Error::unknown_variant(s, &[]))
    }
}

#[cfg(all(test, feature = "serde"))]
#[allow(clippy::unwrap_used)]
mod serde_tests {
    use crate::{Element, ElementKind, Options};

    #[test]
    fn element_kind_wire_name_matches_as_str() {
        for &kind in ElementKind::ALL {
            let json = serde_json::to_string(&kind).unwrap();
            assert_eq!(json, format!("\"{}\"", kind.as_str()));
            assert_eq!(serde_json::from_str::<ElementKind>(&json).unwrap(), kind);
        }
    }

    #[test]
    fn element_kind_rejects_unknown() {
        assert!(serde_json::from_str::<ElementKind>("\"nope\"").is_err());
    }

    #[test]
    fn element_and_options_round_trip() {
        let elements = crate::parse("[Grp] Show - 05 [1080p].mkv", Options::default());
        let json = serde_json::to_string(&elements).unwrap();
        assert_eq!(
            serde_json::from_str::<Vec<Element>>(&json).unwrap(),
            elements
        );

        let opts = Options {
            parse_title: false,
            ..Default::default()
        };
        let back: Options = serde_json::from_str(&serde_json::to_string(&opts).unwrap()).unwrap();
        assert_eq!(back, opts);
        // serde(default) means a partial object fills the rest from Default.
        let partial: Options = serde_json::from_str(r#"{"parse_title":false}"#).unwrap();
        assert_eq!(partial, opts);
    }
}
