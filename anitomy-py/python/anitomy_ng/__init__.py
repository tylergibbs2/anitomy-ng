# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.

"""Anime video filename parser (Python bindings for the anitomy-ng Rust crate).

The compiled extension (``_anitomy``) only deals in plain strings; this
module is what turns that into a strongly-typed surface: ``ElementKind`` is
a real ``enum.Enum``, ``Element`` a real frozen ``dataclass``, both fully
covered by the inline type annotations here (see ``py.typed``).
"""

from __future__ import annotations

from dataclasses import dataclass
from enum import Enum

from ._anitomy import Options, parse as _parse, parse_together as _parse_together

__all__ = ["ElementKind", "Element", "Options", "parse", "parse_together"]


class ElementKind(Enum):
    AUDIO_TERM = "audio_term"
    DEVICE = "device"
    EPISODE = "episode"
    EPISODE_TITLE = "episode_title"
    FILE_CHECKSUM = "file_checksum"
    FILE_EXTENSION = "file_extension"
    LANGUAGE = "language"
    OTHER = "other"
    PART = "part"
    RELEASE_GROUP = "release_group"
    RELEASE_INFORMATION = "release_information"
    RELEASE_VERSION = "release_version"
    SEASON = "season"
    SOURCE = "source"
    SUBTITLES = "subtitles"
    TITLE = "title"
    TYPE = "type"
    VIDEO_RESOLUTION = "video_resolution"
    VIDEO_TERM = "video_term"
    VOLUME = "volume"
    YEAR = "year"


@dataclass(frozen=True)
class Element:
    kind: ElementKind
    value: str
    position: int


def parse(filename: str, options: Options | None = None) -> list[Element]:
    """Parse an anime filename into its elements.

    Elements are ordered by their position in ``filename``; there may be
    multiple elements of the same kind (e.g. two ``ElementKind.EPISODE``
    for an episode range).
    """
    return [
        Element(kind=ElementKind(raw.kind), value=raw.value, position=raw.position)
        for raw in _parse(filename, options)
    ]


def parse_together(
    filenames: list[str], options: Options | None = None
) -> list[list[Element]]:
    """Parse a set of related anime filenames together.

    Returns one element list per input, in the same order (result ``i`` is for
    ``filenames[i]``). The shared context resolves ambiguities a single filename
    can't — e.g. a directory batch range vs. the real per-file episode, or a
    series title that lives only in a parent folder.
    """
    return [
        [
            Element(kind=ElementKind(raw.kind), value=raw.value, position=raw.position)
            for raw in sublist
        ]
        for sublist in _parse_together(list(filenames), options)
    ]
