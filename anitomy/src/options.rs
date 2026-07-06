// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Port of `include/anitomy/options.hpp`.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
        Options {
            parse_episode: true,
            parse_episode_title: true,
            parse_file_checksum: true,
            parse_file_extension: true,
            parse_part: true,
            parse_release_group: true,
            parse_season: true,
            parse_title: true,
            parse_video_resolution: true,
            parse_year: true,
        }
    }
}
