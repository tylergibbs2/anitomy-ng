// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Port of `include/anitomy/detail/keyword.hpp`, including the full
//! keyword database (`include/anitomy/detail/keyword.hpp`'s
//! `make_base_keywords`/`make_keywords`).
//!
//! Lookups are ASCII case-insensitive (matching upstream's `KeywordHash`/
//! `KeywordEqual`, which only fold `A`-`Z`): keys are stored lowercased and
//! queries are lowercased before lookup.

use std::collections::HashMap;
use std::sync::OnceLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum KeywordKind {
    AudioChannels,
    AudioCodec,
    AudioLanguage,
    Device,
    Episode,
    EpisodeType,
    Language,
    Other,
    Part,
    ReleaseGroup,
    ReleaseInformation,
    ReleaseVersion,
    Season,
    Source,
    Subtitles,
    SubtitleLanguage,
    Type,
    VideoCodec,
    VideoColorDepth,
    VideoDynamicRange,
    VideoFormat,
    VideoFrameRate,
    VideoProfile,
    VideoQuality,
    VideoResolution,
    Volume,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Keyword {
    pub kind: KeywordKind,
    pub ambiguous: bool,
    pub subword: bool,
    pub prefix_for_number: bool,
    pub prefix_for_other: bool,
}

const AMBIGUOUS: u8 = 1 << 0;
const SUBWORD: u8 = 1 << 1;
const PREFIX_FOR_NUMBER: u8 = 1 << 2;
const PREFIX_FOR_OTHER: u8 = 1 << 3;

fn make_keyword(kind: KeywordKind, flags: u8) -> Keyword {
    Keyword {
        kind,
        ambiguous: flags & AMBIGUOUS != 0,
        subword: flags & SUBWORD != 0,
        prefix_for_number: flags & PREFIX_FOR_NUMBER != 0,
        prefix_for_other: flags & PREFIX_FOR_OTHER != 0,
    }
}

#[rustfmt::skip]
fn base_keywords() -> &'static [(KeywordKind, &'static [(&'static str, u8)])] {
    use KeywordKind::*;
    &[
        // Audio
        (AudioChannels, &[
            ("2.0", AMBIGUOUS), // e.g. "Evangelion 2.0"
            ("2.0ch", 0),
            ("2ch", 0),
            ("5.1", 0),
            ("5.1ch", 0),
            ("7.1", 0),
            ("7.1ch", 0),
        ]),
        (AudioCodec, &[
            ("AAC", PREFIX_FOR_OTHER),
            ("AACx2", 0),
            ("AACx3", 0),
            ("AACx4", 0),
            ("AC3", 0),
            ("EAC3", 0),
            ("E-AC-3", 0),
            ("E-AC3", 0),
            ("Atmos", 0),
            ("Dolby Atmos", 0),
            ("DD", PREFIX_FOR_OTHER),
            ("DDP", PREFIX_FOR_NUMBER),
            ("Dolby TrueHD", 0),
            ("TrueHD", PREFIX_FOR_NUMBER),
            ("DTS", PREFIX_FOR_NUMBER),
            ("DTS-ES", 0),
            ("FLAC", PREFIX_FOR_NUMBER),
            ("FLACx2", 0),
            ("FLACx3", 0),
            ("FLACx4", 0),
            ("Lossless", 0),
            ("MP3", 0),
            ("Opus", AMBIGUOUS), // e.g. "Opus.COLORs"
            ("OGG", 0),
            ("Vorbis", 0),
        ]),
        (AudioLanguage, &[
            ("DualAudio", 0),
            ("Dual Audio", 0),
            ("MultiAudio", 0),
            ("Multi Audio", 0),
            ("Dub", 0),
            ("Dubbed", 0),
            ("Dubs", 0),
            ("ChiDub", 0),
            ("Chinese Dub", 0),
            ("EngDub", 0),
            ("English Dub", 0),
            ("GerDub", 0),
            ("German Dub", 0),
            ("JapDub", 0),
            ("Japanese Dub", 0),
            ("Korean Dub", 0),
        ]),

        // Device
        (Device, &[
            ("Android", AMBIGUOUS), // e.g. "Dragon Ball Z: Super Android 13"
            ("iPad", PREFIX_FOR_NUMBER),
            ("iPhone", PREFIX_FOR_NUMBER),
            ("iPod", 0),
            ("PS", PREFIX_FOR_NUMBER),
            ("Xbox", PREFIX_FOR_NUMBER),
        ]),

        // Episode
        (Episode, &[
            ("Ep", PREFIX_FOR_NUMBER),
            ("Eps", PREFIX_FOR_NUMBER),
            ("Episode", PREFIX_FOR_NUMBER),
            ("Episodes", PREFIX_FOR_NUMBER),
            ("Episodio", PREFIX_FOR_NUMBER),
            ("Epis\u{f3}dio", PREFIX_FOR_NUMBER),
            ("Capitulo", PREFIX_FOR_NUMBER),
            ("Folge", PREFIX_FOR_NUMBER),
        ]),

        // Episode type
        (EpisodeType, &[
            ("OP", AMBIGUOUS | PREFIX_FOR_NUMBER), // e.g. "takt op.Destiny", "My Unique Skill Makes Me OP even at Level 1"
            ("Opening", AMBIGUOUS),                // e.g. "Pool Opening"
            ("NCOP", PREFIX_FOR_NUMBER),
            ("ED", AMBIGUOUS | PREFIX_FOR_NUMBER), // e.g. "s.CRY.ed"
            ("Ending", AMBIGUOUS),                 // e.g. "Happy Ending", "True Ending"
            ("NCED", PREFIX_FOR_NUMBER),
            ("Preview", AMBIGUOUS),
            ("PV", AMBIGUOUS | PREFIX_FOR_NUMBER),
        ]),

        // Language
        (Language, &[
            ("CHS", 0), // Chinese Simplified
            ("CHT", 0), // Chinese Traditional
            ("ENG", 0),
            ("English", 0),
            ("ESP", AMBIGUOUS), // e.g. "Tokyo ESP"
            ("Espanol", 0),
            ("Spanish", 0),
            ("ITA", AMBIGUOUS), // e.g. "Bokura ga Ita"
            ("JAP", 0),
            ("JPN", 0),
            ("PT-BR", 0),
            ("VOSTFR", 0),
        ]),

        // Other
        (Other, &[
            ("Remaster", 0),
            ("Remastered", 0),
            ("Uncensored", 0),
            ("Uncut", 0),
            ("TS", 0),
            ("VFR", 0),
            ("Widescreen", 0),
            ("WS", 0),
        ]),

        // Part
        (Part, &[
            ("Cour", PREFIX_FOR_NUMBER),
            ("Part", AMBIGUOUS | PREFIX_FOR_NUMBER), // e.g. "Extra Part", "Part-Timer"
            ("Parte", PREFIX_FOR_NUMBER),
        ]),

        // Release group
        (ReleaseGroup, &[
            ("0x539", 0), // to avoid parsing as season 0 episode 539
            ("THORA", 0), // usually placed at the end
            ("VARYG", 0), // placed at the end
        ]),

        // Release information
        (ReleaseInformation, &[
            ("Batch", 0),
            ("Complete", 0),
            ("End", AMBIGUOUS),   // e.g. "The End of Evangelion"
            ("Final", AMBIGUOUS), // e.g. "Final Approach"
            ("Patch", 0),
            ("Remux", 0),
            ("Repack", 0),
        ]),

        // Release version
        (ReleaseVersion, &[
            ("v0", 0),
            ("v1", 0),
            ("v2", 0),
            ("v3", 0),
            ("v4", 0),
        ]),

        // Season
        // Usually preceded or followed by a number (e.g. `2nd Season` or `Season 2`).
        (Season, &[
            ("Season", AMBIGUOUS),
            ("Saison", AMBIGUOUS),
        ]),

        // Source
        (Source, &[
            ("BD", PREFIX_FOR_OTHER),
            ("BDRip", 0),
            ("BluRay", 0),
            ("Blu ray", 0),
            ("DVD", PREFIX_FOR_NUMBER),
            ("DVD5", 0),
            ("DVD9", 0),
            ("DVDISO", 0),
            ("DVDRip", 0),
            ("DVD Rip", 0),
            ("R2DVD", 0),
            ("R2J", 0),
            ("R2JDVD", 0),
            ("R2JDVDRip", 0),
            ("HDTV", 0),
            ("HDTVRip", 0),
            ("TVRip", 0),
            ("TV Rip", 0),
            ("Web", AMBIGUOUS),
            ("Webcast", 0),
            ("WebDL", 0),
            ("Web DL", 0),
            ("WebRip", 0),
            ("ADN", 0),         // Animation Digital Network
            ("AMZN", 0),        // Amazon Prime
            ("BILI", 0),        // Bilibili
            ("Bilibili", 0),
            ("CR", 0),          // Crunchyroll
            ("Crunchyroll", 0),
            ("DSNP", 0),        // Disney+
            ("Funi", 0),        // Funimation
            ("Funimation", 0),
            ("HIDI", 0),        // Hidive
            ("Hidive", 0),
            ("Hulu", 0),
            ("Netflix", 0),
            ("NF", 0),          // Netflix
            ("VRV", 0),
            ("YouTube", 0),
        ]),

        // Subtitles
        (Subtitles, &[
            ("ASS", 0),
            ("BIG5", 0),
            ("Hardsub", 0),
            ("Hardsubs", 0),
            ("RAW", 0),
            ("Softsub", 0),
            ("Softsubs", 0),
            ("Sub", 0),
            ("Subbed", 0),
            ("Subtitled", 0),
            ("Multisub", 0),
            ("Multi Sub", 0),
            ("Multi Subs", 0),
            ("Multiple Subtitle", 0),
        ]),
        (SubtitleLanguage, &[
            ("EngSub", 0),
            ("EngSubs", 0),
            ("GerSub", 0),
        ]),

        // Type
        (Type, &[
            ("TV", AMBIGUOUS),
            ("Movie", AMBIGUOUS),
            ("Gekijouban", AMBIGUOUS),
            ("OAD", AMBIGUOUS | PREFIX_FOR_NUMBER),
            ("OAV", AMBIGUOUS | PREFIX_FOR_NUMBER),
            ("OVA", AMBIGUOUS | PREFIX_FOR_NUMBER),
            ("ONA", AMBIGUOUS | PREFIX_FOR_NUMBER),
            ("SP", AMBIGUOUS | PREFIX_FOR_NUMBER), // e.g. "Yumeiro Patissiere SP Professional"
            ("Special", AMBIGUOUS),                // e.g. "Special A"
            ("Specials", AMBIGUOUS),
        ]),

        // Video
        (VideoColorDepth, &[
            ("8bit", 0),
            ("8bits", 0),
            ("8 bit", 0),
            ("8 bits", 0),
            // Beyond-upstream: `PREFIX_FOR_OTHER` lets the tokenizer split
            // `10bit` from a directly-glued following keyword with no
            // separator, e.g. `10bitH.264` (`H.264` comes from the `"H 264"`
            // entry's generated delimiter variants below; see `build_map`).
            // Upstream produces no video_term at all for the glued run.
            ("10bit", PREFIX_FOR_OTHER),
            ("10bits", 0),
            ("10 bit", 0),
            ("10 bits", 0),
        ]),
        (VideoCodec, &[
            ("AV1", 0),
            ("DivX", PREFIX_FOR_NUMBER),
            ("AVC", 0),
            ("H 264", 0),
            ("H264", 0),
            ("X 264", 0),
            ("X264", 0),
            ("H 265", 0),
            ("H265", 0),
            ("HEVC", PREFIX_FOR_NUMBER),
            ("X 265", 0),
            ("X265", 0),
            ("Xvid", 0),
        ]),
        (VideoDynamicRange, &[
            ("HDR", 0),
            ("HDR10", 0),
            ("Dolby Vision", 0),
            ("DV", 0),
        ]),
        (VideoFormat, &[
            ("AVI", 0),
            ("MP4", 0),
            ("RMVB", 0),
            ("WMV", 0),
            ("WMV3", 0),
            ("WMV9", 0),
        ]),
        (VideoFrameRate, &[
            ("23.976FPS", 0),
            ("24FPS", 0),
            ("29.97FPS", 0),
            ("30FPS", 0),
            ("60FPS", 0),
            ("120FPS", 0),
        ]),
        (VideoProfile, &[
            ("Hi10", 0),
            ("Hi10p", 0),
            ("Hi444", 0),
            ("Hi444P", 0),
            ("Hi444PP", 0),
        ]),
        (VideoQuality, &[
            ("HD", 0),
            ("SD", 0),
            ("HQ", 0),
            ("LQ", 0),
        ]),
        (VideoResolution, &[
            ("1080p", SUBWORD),
            ("1440p", SUBWORD),
            ("2160p", SUBWORD),
            ("4K", 0),
        ]),

        // Volume
        (Volume, &[
            ("Vol", PREFIX_FOR_NUMBER),
            ("Volume", PREFIX_FOR_NUMBER),
        ]),
    ]
}

fn build_map() -> HashMap<String, Keyword> {
    let mut map = HashMap::new();
    for (kind, entries) in base_keywords() {
        for (value, flags) in *entries {
            let keyword = make_keyword(*kind, *flags);
            map.insert(value.to_ascii_lowercase(), keyword);
            if value.contains(' ') {
                for delimiter in ['_', '.', '-'] {
                    map.insert(
                        value
                            .replace(' ', &delimiter.to_string())
                            .to_ascii_lowercase(),
                        keyword,
                    );
                }
            }
        }
    }
    map
}

fn keywords() -> &'static HashMap<String, Keyword> {
    static MAP: OnceLock<HashMap<String, Keyword>> = OnceLock::new();
    MAP.get_or_init(build_map)
}

/// Exact, case-insensitive lookup.
pub(crate) fn get(word: &str) -> Option<Keyword> {
    keywords().get(&word.to_ascii_lowercase()).copied()
}

/// Whether any known keyword starts with `prefix` (case-insensitive). Used
/// by the tokenizer to find the longest valid keyword at a given position
/// without scanning the whole table character-by-character from scratch.
pub(crate) fn has_prefix(prefix: &str) -> bool {
    let prefix = prefix.to_ascii_lowercase();
    keywords().keys().any(|k| k.starts_with(&prefix))
}
