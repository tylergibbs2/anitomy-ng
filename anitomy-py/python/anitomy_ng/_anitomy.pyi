# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.

"""Type stub for the compiled native extension. Internal — import from
`anitomy_ng` (the package `__init__.py`), not from here directly.
"""

class Options:
    parse_episode: bool
    parse_episode_title: bool
    parse_file_checksum: bool
    parse_file_extension: bool
    parse_part: bool
    parse_release_group: bool
    parse_season: bool
    parse_title: bool
    parse_video_resolution: bool
    parse_year: bool

    def __init__(
        self,
        *,
        parse_episode: bool = True,
        parse_episode_title: bool = True,
        parse_file_checksum: bool = True,
        parse_file_extension: bool = True,
        parse_part: bool = True,
        parse_release_group: bool = True,
        parse_season: bool = True,
        parse_title: bool = True,
        parse_video_resolution: bool = True,
        parse_year: bool = True,
    ) -> None: ...

class RawElement:
    kind: str
    value: str
    position: int

def parse(filename: str, options: Options | None = None) -> list[RawElement]: ...
def parse_together(
    filenames: list[str], options: Options | None = None
) -> list[list[RawElement]]: ...
