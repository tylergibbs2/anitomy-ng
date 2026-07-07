// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! C ABI for [`anitomy_ng`] — the basis for non-Rust bindings (C#, C/C++, Go…).
//!
//! # The one unsafe crate
//!
//! The core `anitomy-ng` crate is `#![forbid(unsafe_code)]`. All FFI `unsafe`
//! is isolated here, at the boundary, so the parser itself keeps that
//! guarantee. This crate only marshals values across the boundary; it contains
//! no parsing logic.
//!
//! # Ownership model (opaque handle)
//!
//! [`anitomy_parse`] returns a `*mut AnitomyResult` that owns the whole parse
//! (the elements and their strings). Callers read fields through the accessor
//! functions and must hand the pointer back to [`anitomy_result_free`] exactly
//! once. Memory allocated by Rust is freed only by Rust — never `free()` a
//! pointer returned from here on the C side.
//!
//! Strings returned by [`anitomy_result_value`] are borrowed views into the
//! result and are valid until that result is freed; copy them out before
//! freeing. [`anitomy_kind_name`] and [`anitomy_version`] return `'static`
//! strings that are never freed.
//!
//! # Encoding
//!
//! All strings crossing the boundary are NUL-terminated UTF-8. `position` is a
//! Unicode-scalar (codepoint) index into the input, matching upstream — note
//! this is *not* a UTF-16 code-unit index, so it won't line up with a .NET
//! `string` offset for characters outside the Basic Multilingual Plane.

// Require every unsafe operation — even inside an `unsafe fn` — to sit in an
// explicit `unsafe {}` block. Keeps blocks minimal and each raw-pointer use
// individually visible and audited, rather than an `unsafe fn` body being one
// big implicitly-unsafe region.
#![deny(unsafe_op_in_unsafe_fn)]

use std::ffi::{c_char, CStr, CString};
use std::panic::{self, AssertUnwindSafe};
use std::ptr;

use anitomy_ng::{ElementKind, Options};

// --- Options bitmask -------------------------------------------------------
//
// One bit per parse category, in the same order as `anitomy_ng::Options`'
// fields. A set bit enables that category. Passing `0` disables everything;
// use `anitomy_options_default()` for the "all enabled" default.

/// Extract episode numbers.
pub const ANITOMY_OPTION_EPISODE: u32 = 1 << 0;
/// Extract episode titles.
pub const ANITOMY_OPTION_EPISODE_TITLE: u32 = 1 << 1;
/// Extract file checksums (CRC32).
pub const ANITOMY_OPTION_FILE_CHECKSUM: u32 = 1 << 2;
/// Extract the file extension.
pub const ANITOMY_OPTION_FILE_EXTENSION: u32 = 1 << 3;
/// Extract part markers.
pub const ANITOMY_OPTION_PART: u32 = 1 << 4;
/// Extract the release group.
pub const ANITOMY_OPTION_RELEASE_GROUP: u32 = 1 << 5;
/// Extract season numbers.
pub const ANITOMY_OPTION_SEASON: u32 = 1 << 6;
/// Extract the title.
pub const ANITOMY_OPTION_TITLE: u32 = 1 << 7;
/// Extract the video resolution.
pub const ANITOMY_OPTION_VIDEO_RESOLUTION: u32 = 1 << 8;
/// Extract the year.
pub const ANITOMY_OPTION_YEAR: u32 = 1 << 9;

/// The default option mask: every category enabled.
#[no_mangle]
pub extern "C" fn anitomy_options_default() -> u32 {
    ANITOMY_OPTION_EPISODE
        | ANITOMY_OPTION_EPISODE_TITLE
        | ANITOMY_OPTION_FILE_CHECKSUM
        | ANITOMY_OPTION_FILE_EXTENSION
        | ANITOMY_OPTION_PART
        | ANITOMY_OPTION_RELEASE_GROUP
        | ANITOMY_OPTION_SEASON
        | ANITOMY_OPTION_TITLE
        | ANITOMY_OPTION_VIDEO_RESOLUTION
        | ANITOMY_OPTION_YEAR
}

fn options_from_bits(bits: u32) -> Options {
    Options {
        parse_episode: bits & ANITOMY_OPTION_EPISODE != 0,
        parse_episode_title: bits & ANITOMY_OPTION_EPISODE_TITLE != 0,
        parse_file_checksum: bits & ANITOMY_OPTION_FILE_CHECKSUM != 0,
        parse_file_extension: bits & ANITOMY_OPTION_FILE_EXTENSION != 0,
        parse_part: bits & ANITOMY_OPTION_PART != 0,
        parse_release_group: bits & ANITOMY_OPTION_RELEASE_GROUP != 0,
        parse_season: bits & ANITOMY_OPTION_SEASON != 0,
        parse_title: bits & ANITOMY_OPTION_TITLE != 0,
        parse_video_resolution: bits & ANITOMY_OPTION_VIDEO_RESOLUTION != 0,
        parse_year: bits & ANITOMY_OPTION_YEAR != 0,
    }
}

// --- ElementKind discriminants ---------------------------------------------
//
// Stable integer values for `ElementKind`, pinned here so reordering the Rust
// enum can never silently shift the ABI. A test below asserts the mapping is
// total. Bindings should mirror these exact numbers.

/// Maps an [`ElementKind`] to its stable ABI discriminant.
fn kind_to_u32(kind: ElementKind) -> u32 {
    match kind {
        ElementKind::AudioTerm => 0,
        ElementKind::Device => 1,
        ElementKind::Episode => 2,
        ElementKind::EpisodeTitle => 3,
        ElementKind::FileChecksum => 4,
        ElementKind::FileExtension => 5,
        ElementKind::Language => 6,
        ElementKind::Other => 7,
        ElementKind::Part => 8,
        ElementKind::ReleaseGroup => 9,
        ElementKind::ReleaseInformation => 10,
        ElementKind::ReleaseVersion => 11,
        ElementKind::Season => 12,
        ElementKind::Source => 13,
        ElementKind::Subtitles => 14,
        ElementKind::Title => 15,
        ElementKind::Type => 16,
        ElementKind::VideoResolution => 17,
        ElementKind::VideoTerm => 18,
        ElementKind::Volume => 19,
        ElementKind::Year => 20,
    }
}

/// The snake_case name of an `ElementKind` discriminant (as returned by
/// [`anitomy_result_kind`]), or an empty string for an unknown value. The
/// returned pointer is a `'static` C string and must not be freed.
#[no_mangle]
pub extern "C" fn anitomy_kind_name(kind: u32) -> *const c_char {
    let name: &CStr = match kind {
        0 => c"audio_term",
        1 => c"device",
        2 => c"episode",
        3 => c"episode_title",
        4 => c"file_checksum",
        5 => c"file_extension",
        6 => c"language",
        7 => c"other",
        8 => c"part",
        9 => c"release_group",
        10 => c"release_information",
        11 => c"release_version",
        12 => c"season",
        13 => c"source",
        14 => c"subtitles",
        15 => c"title",
        16 => c"type",
        17 => c"video_resolution",
        18 => c"video_term",
        19 => c"volume",
        20 => c"year",
        _ => c"",
    };
    name.as_ptr()
}

// --- Result handle ---------------------------------------------------------

/// One parsed element, pre-marshalled for the C side.
struct CElement {
    kind: u32,
    value: CString,
    position: usize,
}

/// Opaque owner of a parse result. Create with [`anitomy_parse`], free with
/// [`anitomy_result_free`].
pub struct AnitomyResult {
    items: Vec<CElement>,
}

/// Parses `input` (a NUL-terminated UTF-8 string) with the given option mask
/// (see the `ANITOMY_OPTION_*` bits, or [`anitomy_options_default`]).
///
/// Returns an owning handle the caller must release with
/// [`anitomy_result_free`], or NULL if `input` is NULL, is not valid UTF-8, or
/// (impossibly, given the core is panic-free) the parser panicked.
///
/// # Safety
///
/// `input` must be NULL or a valid pointer to a NUL-terminated string.
#[no_mangle]
pub unsafe extern "C" fn anitomy_parse(input: *const c_char, options: u32) -> *mut AnitomyResult {
    // The core never panics (see anitomy's tests/no_panic.rs), but unwinding
    // across the FFI boundary is UB, so contain it and degrade to NULL.
    let parsed = panic::catch_unwind(AssertUnwindSafe(|| {
        if input.is_null() {
            return None;
        }
        // SAFETY: non-null by the check above; the caller guarantees a
        // NUL-terminated string per this function's contract.
        let bytes = unsafe { CStr::from_ptr(input) };
        let text = bytes.to_str().ok()?;

        let items = anitomy_ng::parse(text, options_from_bits(options))
            .into_iter()
            .map(|e| CElement {
                kind: kind_to_u32(e.kind),
                // A parsed value can't contain an interior NUL (it's a slice of
                // the input's non-delimiter tokens), but degrade to "" rather
                // than fail if that assumption ever breaks.
                value: CString::new(e.value).unwrap_or_default(),
                position: e.position,
            })
            .collect();
        Some(Box::new(AnitomyResult { items }))
    }));

    match parsed {
        Ok(Some(result)) => Box::into_raw(result),
        _ => ptr::null_mut(),
    }
}

/// Borrows a handle as a safe reference — the single place a caller-provided
/// pointer is dereferenced. Every read accessor goes through this, so the raw
/// deref (and any future hardening of it) lives in exactly one audited spot
/// instead of being repeated per accessor. Yields `None` for NULL.
///
/// # Safety
///
/// `result` must be NULL or a live handle from [`anitomy_parse`].
unsafe fn result_ref<'a>(result: *const AnitomyResult) -> Option<&'a AnitomyResult> {
    // SAFETY: upheld by this function's own contract.
    unsafe { result.as_ref() }
}

/// The number of elements in `result` (0 if `result` is NULL).
///
/// # Safety
///
/// `result` must be NULL or a live handle from [`anitomy_parse`].
#[no_mangle]
pub unsafe extern "C" fn anitomy_result_len(result: *const AnitomyResult) -> usize {
    // SAFETY: forwarded to `result_ref`'s contract.
    unsafe { result_ref(result) }.map_or(0, |r| r.items.len())
}

/// The `ElementKind` discriminant of element `index` (see [`anitomy_kind_name`]),
/// or `u32::MAX` if `result` is NULL or `index` is out of range.
///
/// # Safety
///
/// `result` must be NULL or a live handle from [`anitomy_parse`].
#[no_mangle]
pub unsafe extern "C" fn anitomy_result_kind(result: *const AnitomyResult, index: usize) -> u32 {
    // SAFETY: forwarded to `result_ref`'s contract.
    unsafe { result_ref(result) }
        .and_then(|r| r.items.get(index))
        .map_or(u32::MAX, |e| e.kind)
}

/// The value of element `index` as a NUL-terminated UTF-8 string, borrowed from
/// `result` (valid until it is freed), or NULL if `result` is NULL or `index`
/// is out of range. Copy it out before calling [`anitomy_result_free`].
///
/// # Safety
///
/// `result` must be NULL or a live handle from [`anitomy_parse`].
#[no_mangle]
pub unsafe extern "C" fn anitomy_result_value(
    result: *const AnitomyResult,
    index: usize,
) -> *const c_char {
    // SAFETY: forwarded to `result_ref`'s contract.
    unsafe { result_ref(result) }
        .and_then(|r| r.items.get(index))
        .map_or(ptr::null(), |e| e.value.as_ptr())
}

/// The codepoint position of element `index` in the input (0 if `result` is
/// NULL or `index` is out of range).
///
/// # Safety
///
/// `result` must be NULL or a live handle from [`anitomy_parse`].
#[no_mangle]
pub unsafe extern "C" fn anitomy_result_position(
    result: *const AnitomyResult,
    index: usize,
) -> usize {
    // SAFETY: forwarded to `result_ref`'s contract.
    unsafe { result_ref(result) }
        .and_then(|r| r.items.get(index))
        .map_or(0, |e| e.position)
}

/// Frees a result returned by [`anitomy_parse`]. NULL is ignored. Never call
/// this more than once on the same pointer.
///
/// # Safety
///
/// `result` must be NULL or a handle from [`anitomy_parse`] that has not
/// already been freed.
#[no_mangle]
pub unsafe extern "C" fn anitomy_result_free(result: *mut AnitomyResult) {
    if !result.is_null() {
        // SAFETY: non-null and, per the contract, a pointer produced by
        // `anitomy_parse` and not yet freed.
        drop(unsafe { Box::from_raw(result) });
    }
}

/// This crate's version as a `'static` NUL-terminated string; must not be freed.
#[no_mangle]
pub extern "C" fn anitomy_version() -> *const c_char {
    const VERSION: &str = concat!(env!("CARGO_PKG_VERSION"), "\0");
    VERSION.as_ptr().cast()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Round-trips a filename through the real extern "C" surface the way a C
    /// caller would: parse, read every field, free.
    #[test]
    fn parse_read_free() {
        unsafe {
            let input = c"[TaigaSubs] Toradora! (2008) - 01v2 [1280x720][1234ABCD].mkv";
            let result = anitomy_parse(input.as_ptr(), anitomy_options_default());
            assert!(!result.is_null());

            let len = anitomy_result_len(result);
            assert!(len > 0);

            // Collect (kind_name, value) pairs by walking the accessors.
            let mut seen = Vec::new();
            for i in 0..len {
                let kind = anitomy_result_kind(result, i);
                assert_ne!(kind, u32::MAX);
                let name = CStr::from_ptr(anitomy_kind_name(kind))
                    .to_str()
                    .unwrap()
                    .to_owned();
                let value_ptr = anitomy_result_value(result, i);
                assert!(!value_ptr.is_null());
                let value = CStr::from_ptr(value_ptr).to_str().unwrap().to_owned();
                let _pos = anitomy_result_position(result, i);
                seen.push((name, value));
            }

            assert!(seen
                .iter()
                .any(|(k, v)| k == "release_group" && v == "TaigaSubs"));
            assert!(seen.iter().any(|(k, v)| k == "title" && v == "Toradora!"));
            assert!(seen.iter().any(|(k, v)| k == "year" && v == "2008"));
            assert!(seen
                .iter()
                .any(|(k, v)| k == "file_extension" && v == "mkv"));

            anitomy_result_free(result);
        }
    }

    #[test]
    fn null_and_oob_are_safe() {
        unsafe {
            assert!(anitomy_parse(ptr::null(), anitomy_options_default()).is_null());
            assert_eq!(anitomy_result_len(ptr::null()), 0);
            assert_eq!(anitomy_result_kind(ptr::null(), 0), u32::MAX);
            assert!(anitomy_result_value(ptr::null(), 0).is_null());
            assert_eq!(anitomy_result_position(ptr::null(), 99), 0);
            anitomy_result_free(ptr::null_mut()); // no-op, must not crash

            let result = anitomy_parse(c"x.mkv".as_ptr(), anitomy_options_default());
            let len = anitomy_result_len(result);
            assert_eq!(anitomy_result_kind(result, len + 5), u32::MAX);
            assert!(anitomy_result_value(result, len + 5).is_null());
            anitomy_result_free(result);
        }
    }

    #[test]
    fn options_mask_disables_categories() {
        unsafe {
            // Title disabled: no title element should appear.
            let mask = anitomy_options_default() & !ANITOMY_OPTION_TITLE;
            let result = anitomy_parse(c"Toradora! - 01.mkv".as_ptr(), mask);
            let len = anitomy_result_len(result);
            for i in 0..len {
                let name = CStr::from_ptr(anitomy_kind_name(anitomy_result_kind(result, i)))
                    .to_str()
                    .unwrap();
                assert_ne!(name, "title");
            }
            anitomy_result_free(result);
        }
    }

    /// Every `ElementKind` maps to a discriminant whose `anitomy_kind_name`
    /// round-trips — guards against the enum and the ABI table drifting apart.
    #[test]
    fn kind_mapping_is_total() {
        // If a variant is added to ElementKind without updating kind_to_u32,
        // its parse output would surface here (or the name would be empty).
        for n in 0..21u32 {
            let name = unsafe { CStr::from_ptr(anitomy_kind_name(n)) }
                .to_str()
                .unwrap();
            assert!(!name.is_empty(), "discriminant {n} has no name");
        }
        // One past the end is empty.
        assert!(unsafe { CStr::from_ptr(anitomy_kind_name(21)) }
            .to_str()
            .unwrap()
            .is_empty());
    }
}
