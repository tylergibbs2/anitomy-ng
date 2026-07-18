// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Port of `include/anitomy/detail/util.hpp`, limited to the helpers the
//! parser actually uses. Thin wrappers over `std` (`is_alpha`, `is_digit`,
//! `is_xdigit`, `to_float`, `to_lower`) are omitted in favour of calling the
//! standard library directly; `read_file` is CLI-only (not part of the
//! library) and is intentionally omitted; and `find_all_if` is a
//! generic-iterator algorithm better expressed as `Iterator` combinators at
//! each call site in Rust.

use std::collections::{HashMap, HashSet};
use std::hash::{BuildHasherDefault, Hasher};

/// FxHash: a fast non-cryptographic hasher for short ASCII keys, where the
/// default SipHash's per-lookup overhead would dominate (e.g. the tokenizer's
/// keyword scan, which hashes a prefix on every character).
#[derive(Default)]
pub(crate) struct FxHasher(u64);

impl Hasher for FxHasher {
    fn write(&mut self, bytes: &[u8]) {
        const SEED: u64 = 0x51_7c_c1_b7_27_22_0a_95;
        for &b in bytes {
            self.0 = (self.0.rotate_left(5) ^ u64::from(b)).wrapping_mul(SEED);
        }
    }
    fn finish(&self) -> u64 {
        self.0
    }
}

pub(crate) type FxMap<K, V> = HashMap<K, V, BuildHasherDefault<FxHasher>>;
pub(crate) type FxSet<K> = HashSet<K, BuildHasherDefault<FxHasher>>;

pub(crate) fn from_ordinal_number(input: &str) -> Option<&'static str> {
    Some(match input {
        "1st" | "First" => "1",
        "2nd" | "Second" => "2",
        "3rd" | "Third" => "3",
        "4th" | "Fourth" => "4",
        "5th" | "Fifth" => "5",
        "6th" | "Sixth" => "6",
        "7th" | "Seventh" => "7",
        "8th" | "Eighth" => "8",
        "9th" | "Ninth" => "9",
        _ => return None,
    })
}

pub(crate) fn from_roman_number(input: &str) -> Option<&'static str> {
    Some(match input {
        "II" => "2",
        "III" => "3",
        "IV" => "4",
        _ => return None,
    })
}

pub(crate) fn to_int(str: &str) -> i32 {
    str.parse().unwrap_or(0)
}

/// ASCII case-insensitive comparison (matches upstream `equal`, which only
/// folds `A`-`Z`, not full Unicode case folding).
pub(crate) fn equal_ignore_ascii_case(a: &str, b: &str) -> bool {
    a.eq_ignore_ascii_case(b)
}

/// Converts a byte offset (as `regex::Match::start()`/`end()` return) into
/// a codepoint count, matching `Token::position`'s units (upstream counts
/// positions in UTF-32 codepoints, not UTF-8 bytes). Cheap here since
/// matched strings are always short (a single token's value).
pub(crate) fn byte_to_char_offset(s: &str, byte_offset: usize) -> usize {
    s.get(..byte_offset)
        .map_or_else(|| s.chars().count(), |prefix| prefix.chars().count())
}
