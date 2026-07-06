// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Port of `include/anitomy/detail/parser.hpp`: runs the per-category
//! sub-parsers (in `detail::parser::*`, one module per
//! `include/anitomy/detail/parser/*.hpp`) in a fixed order — later parsers
//! rely on earlier ones having already claimed (and mutated the `kind`/
//! `element_kind` of) their tokens — then sorts the collected elements by
//! position.
//!
//! Each sub-parser is currently a stub returning nothing; fill them in one
//! at a time (each is a fairly self-contained file, matching upstream) and
//! watch `tests/conformance.rs`'s pass count climb.

mod episode;
mod episode_title;
mod file_checksum;
mod file_extension;
mod keywords;
mod part;
mod release_group;
mod season;
mod title;
mod video_resolution;
mod volume;
mod year;

use super::token::Token;
use crate::element::{Element, ElementKind};
use crate::options::Options;

pub(crate) fn parse(mut tokens: Vec<Token>, options: &Options) -> Vec<Element> {
    let mut elements: Vec<Element> = Vec::new();

    let contains =
        |elements: &[Element], kind: ElementKind| elements.iter().any(|e| e.kind == kind);

    if options.parse_file_extension {
        elements.extend(file_extension::parse_file_extension(&mut tokens));
    }

    elements.extend(keywords::parse_keywords(&mut tokens, options));

    if options.parse_file_checksum {
        elements.extend(file_checksum::parse_file_checksum(&mut tokens));
    }

    if options.parse_video_resolution {
        elements.extend(video_resolution::parse_video_resolution(&mut tokens));
    }

    if options.parse_year {
        elements.extend(year::parse_year(&mut tokens));
    }

    if options.parse_season {
        elements.extend(season::parse_season(&mut tokens));
    }

    if options.parse_part {
        elements.extend(part::parse_part(&mut tokens));
    }

    if options.parse_episode {
        elements.extend(volume::parse_volume(&mut tokens));
        elements.extend(episode::parse_episode(&mut tokens));
    }

    if options.parse_title {
        elements.extend(title::parse_title(&mut tokens));
    }

    if options.parse_release_group && !contains(&elements, ElementKind::ReleaseGroup) {
        elements.extend(release_group::parse_release_group(&mut tokens));
    }

    if options.parse_episode_title && contains(&elements, ElementKind::Episode) {
        elements.extend(episode_title::parse_episode_title(&mut tokens));
    }

    elements.sort_by_key(|e| e.position);
    elements
}
