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

use std::collections::HashSet;
use std::sync::OnceLock;

use super::util::{FxMap, FxSet};

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
            // `<lang-code>Dub` tags (`GerDub`, `GerJapDub`, ...): see `get_composite`.
            // Only the spelled-out forms remain here.
            ("Chinese Dub", 0),
            ("English Dub", 0),
            ("German Dub", 0),
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
            // ISO-639-style three-letter codes common in multi-language
            // release tags (e.g. `[FRE][GER][SPA]`). AMBIGUOUS -> recognized
            // only when bracketed, so they never misfire on title words.
            ("ARA", AMBIGUOUS),
            ("CHI", AMBIGUOUS),
            ("DEU", AMBIGUOUS),
            ("FRA", AMBIGUOUS),
            ("FRE", AMBIGUOUS),
            ("GER", AMBIGUOUS),
            ("KOR", AMBIGUOUS),
            ("POR", AMBIGUOUS),
            ("RUS", AMBIGUOUS),
            ("SPA", AMBIGUOUS),
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
            // Abbreviation `pt3`/`pt.2`. Ambiguous (like `Part`), so it only
            // counts enclosed — e.g. `(Season 04 pt3)` — never as a bare word.
            // The longer `pt-BR` language keyword still wins by longest-match,
            // so it isn't mistaken for a part.
            ("Pt", AMBIGUOUS | PREFIX_FOR_NUMBER),
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
        // `<lang-code>+Sub` tags (`GerSub`, `GerEngSub`, ...): see `get_composite`.

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

fn build_map() -> FxMap<String, Keyword> {
    let mut map = FxMap::default();
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

fn keywords() -> &'static FxMap<String, Keyword> {
    static MAP: OnceLock<FxMap<String, Keyword>> = OnceLock::new();
    MAP.get_or_init(build_map)
}

/// Exact keyword lookup; `lower` must already be ASCII-lowercased.
pub(crate) fn get_lower(lower: &str) -> Option<Keyword> {
    keywords().get(lower).copied()
}

/// Whether any known keyword starts with `lower` (which must already be
/// ASCII-lowercased). Answers in one hash lookup; the tokenizer calls it once
/// per character while growing a candidate keyword.
pub(crate) fn has_prefix_lower(lower: &str) -> bool {
    prefixes().contains(lower)
}

/// The three-letter `Language` keywords, lowercased — the single source of
/// truth for language codes. [`get_composite`] draws its vocabulary from here,
/// so adding a code to the `Language` table (e.g. `FRE`) also teaches the
/// composite recognizer `FreSub`/`FreDub`, with no second list to keep in sync.
fn language_codes() -> &'static HashSet<String> {
    static CODES: OnceLock<HashSet<String>> = OnceLock::new();
    CODES.get_or_init(|| {
        let mut set = HashSet::new();
        for (kind, entries) in base_keywords() {
            if *kind != KeywordKind::Language {
                continue;
            }
            for (value, _) in *entries {
                if value.len() == 3 && value.bytes().all(|b| b.is_ascii_alphabetic()) {
                    set.insert(value.to_ascii_lowercase());
                }
            }
        }
        set
    })
}

/// Matches `(<code>)+ Sub|Subs|Dub|Dubs` generically instead of enumerating
/// every combination as a keyword. Returns a synthetic `SubtitleLanguage`
/// (`Sub`) or `AudioLanguage` (`Dub`) keyword when `word` is one or more
/// three-letter [`language_codes`] plus such a suffix (e.g. `GerSub`, `GerJapDub`).
pub(crate) fn get_composite(word: &str) -> Option<Keyword> {
    let lower = word.to_ascii_lowercase();
    let (codes, kind) = if let Some(rest) = strip_suffix_any(&lower, &["subs", "sub"]) {
        (rest, KeywordKind::SubtitleLanguage)
    } else if let Some(rest) = strip_suffix_any(&lower, &["dubs", "dub"]) {
        (rest, KeywordKind::AudioLanguage)
    } else {
        return None;
    };
    // Prefix must be a non-empty run of whole three-letter codes.
    if codes.is_empty() || codes.len() % 3 != 0 {
        return None;
    }
    let known = language_codes();
    let all_codes = codes
        .as_bytes()
        .chunks(3)
        .all(|c| std::str::from_utf8(c).is_ok_and(|s| known.contains(s)));
    all_codes.then(|| make_keyword(kind, 0))
}

/// Returns `word` without the first matching suffix in `suffixes` (tried in
/// order, so list longer suffixes first).
fn strip_suffix_any<'a>(word: &'a str, suffixes: &[&str]) -> Option<&'a str> {
    suffixes.iter().find_map(|s| word.strip_suffix(s))
}

/// Every (lowercased) prefix of every known keyword, including the empty
/// string and each full key. Backs [`has_prefix_lower`].
fn prefixes() -> &'static FxSet<String> {
    static PREFIXES: OnceLock<FxSet<String>> = OnceLock::new();
    PREFIXES.get_or_init(|| {
        let mut set = FxSet::default();
        for key in keywords().keys() {
            // Keys are already lowercased (see `build_map`); accumulate chars
            // so multi-byte prefixes land on char boundaries.
            let mut prefix = String::new();
            set.insert(prefix.clone());
            for ch in key.chars() {
                prefix.push(ch);
                set.insert(prefix.clone());
            }
        }
        set
    })
}
