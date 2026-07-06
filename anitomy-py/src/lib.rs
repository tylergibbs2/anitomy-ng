// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Native `anitomy_ng._anitomy` extension module.
//!
//! Kept thin: it exposes typed data (`RawElement`, `Options`) and one
//! function (`parse`) across the FFI boundary. The ergonomic public API
//! (`Element` as a `dataclass`, `ElementKind` as an `enum.Enum`) lives in
//! `python/anitomy_ng/__init__.py`, which is easier to keep strongly typed
//! (mypy/pyright-checkable, no macro magic) as plain Python than to build
//! the same thing out of pyo3 classes.
//!
//! No `unsafe` in this crate; pyo3's macros are specifically designed to
//! expand under `#![forbid(unsafe_code)]` in the user's own crate.

#![forbid(unsafe_code)]
#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::unreachable
)]

use pyo3::prelude::*;

/// Mirrors `anitomy_ng::Options`. Re-exported directly as
/// `anitomy_ng.Options`, since it's a flat struct of bools.
#[pyclass(get_all, set_all)]
#[derive(Debug, Clone, Copy)]
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

#[pymethods]
impl Options {
    #[new]
    #[pyo3(signature = (
        *,
        parse_episode = true,
        parse_episode_title = true,
        parse_file_checksum = true,
        parse_file_extension = true,
        parse_part = true,
        parse_release_group = true,
        parse_season = true,
        parse_title = true,
        parse_video_resolution = true,
        parse_year = true,
    ))]
    #[allow(clippy::too_many_arguments)]
    fn new(
        parse_episode: bool,
        parse_episode_title: bool,
        parse_file_checksum: bool,
        parse_file_extension: bool,
        parse_part: bool,
        parse_release_group: bool,
        parse_season: bool,
        parse_title: bool,
        parse_video_resolution: bool,
        parse_year: bool,
    ) -> Self {
        Options {
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
    }

    fn __repr__(&self) -> String {
        format!("{self:?}")
    }
}

impl From<Options> for anitomy_ng::Options {
    fn from(o: Options) -> Self {
        anitomy_ng::Options {
            parse_episode: o.parse_episode,
            parse_episode_title: o.parse_episode_title,
            parse_file_checksum: o.parse_file_checksum,
            parse_file_extension: o.parse_file_extension,
            parse_part: o.parse_part,
            parse_release_group: o.parse_release_group,
            parse_season: o.parse_season,
            parse_title: o.parse_title,
            parse_video_resolution: o.parse_video_resolution,
            parse_year: o.parse_year,
        }
    }
}

impl Default for Options {
    fn default() -> Self {
        anitomy_ng::Options::default().into()
    }
}

impl From<anitomy_ng::Options> for Options {
    fn from(o: anitomy_ng::Options) -> Self {
        Options {
            parse_episode: o.parse_episode,
            parse_episode_title: o.parse_episode_title,
            parse_file_checksum: o.parse_file_checksum,
            parse_file_extension: o.parse_file_extension,
            parse_part: o.parse_part,
            parse_release_group: o.parse_release_group,
            parse_season: o.parse_season,
            parse_title: o.parse_title,
            parse_video_resolution: o.parse_video_resolution,
            parse_year: o.parse_year,
        }
    }
}

/// One parsed element, still in raw/untyped form (`kind` is the snake_case
/// `ElementKind` name, e.g. `"release_group"`). `anitomy_ng.parse` in
/// `__init__.py` converts these into `anitomy_ng.Element` (`kind` becomes a
/// real `anitomy_ng.ElementKind` member).
#[pyclass(get_all, frozen)]
pub struct RawElement {
    pub kind: String,
    pub value: String,
    pub position: usize,
}

#[pyfunction]
#[pyo3(signature = (filename, options=None))]
fn parse(filename: &str, options: Option<Options>) -> Vec<RawElement> {
    let opts: anitomy_ng::Options = options.unwrap_or_default().into();
    anitomy_ng::parse(filename, opts)
        .into_iter()
        .map(|e| RawElement {
            kind: e.kind.as_str().to_string(),
            value: e.value,
            position: e.position,
        })
        .collect()
}

#[pymodule]
fn _anitomy(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Options>()?;
    m.add_class::<RawElement>()?;
    m.add_function(wrap_pyfunction!(parse, m)?)?;
    Ok(())
}
