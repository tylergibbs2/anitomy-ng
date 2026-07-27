// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Port of `include/anitomy/options.hpp`.

/// Declares [`Options`] and its `Default` from one field list, so the fields
/// and their (all-`true`) defaults can't fall out of sync. The bindings
/// (`anitomy-py`, `anitomy-js`) mirror this same field list; keep them aligned.
macro_rules! options {
    ($($field:ident),+ $(,)?) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
        #[cfg_attr(feature = "serde", serde(default))]
        pub struct Options {
            $(pub $field: bool),+
        }

        impl Options {
            /// Field names, in declaration order — lets the bindings check
            /// their mirrored copies of this list against it.
            pub const FIELDS: &'static [&'static str] = &[$(stringify!($field)),+];
        }

        impl Default for Options {
            fn default() -> Self {
                Options {
                    $($field: true),+
                }
            }
        }
    };
}

options! {
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
