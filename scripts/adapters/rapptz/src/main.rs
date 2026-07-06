// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Benchmark adapter for Rapptz/anitomy-rs. Reads one filename per line on
//! stdin; writes one JSON object per line:
//!   {"input": "<line>", "output": {"<kind>": ["<value>", ...], ...}}
//! Kinds are mapped to current-schema snake_case names (schema=current). See
//! scripts/benchmark.py.

use std::collections::BTreeMap;
use std::io::{self, BufRead, Write};

use anitomy::ElementKind;

/// Map Rapptz's ElementKind to anitomy-ng's current snake_case kind, or `None`
/// to drop kinds with no current-schema equivalent.
fn kind_name(kind: ElementKind) -> Option<&'static str> {
    use ElementKind::*;
    Some(match kind {
        AudioTerm => "audio_term",
        DeviceCompatibility => "device",
        Episode => "episode",
        EpisodeTitle => "episode_title",
        EpisodeAlt => "episode",
        FileChecksum => "file_checksum",
        FileExtension => "file_extension",
        Language => "language",
        Other => "other",
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
        // `Date` (and any variant added upstream) has no current-schema kind.
        _ => return None,
    })
}

fn main() -> io::Result<()> {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut out = stdout.lock();

    for line in stdin.lock().lines() {
        let input = line?;
        let mut grouped: BTreeMap<&str, Vec<String>> = BTreeMap::new();
        for element in anitomy::parse(&input) {
            if let Some(name) = kind_name(element.kind()) {
                grouped.entry(name).or_default().push(element.value().to_string());
            }
        }
        let obj = serde_json::json!({ "input": input, "output": grouped });
        writeln!(out, "{obj}")?;
    }
    Ok(())
}
