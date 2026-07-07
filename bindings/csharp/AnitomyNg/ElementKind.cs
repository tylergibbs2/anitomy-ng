// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

namespace AnitomyNg;

/// <summary>
/// The kind of a parsed <see cref="Element"/>. The integer values are the
/// stable ABI discriminants defined in anitomy-c (kind_to_u32) and must not be
/// reordered independently of it.
/// </summary>
public enum ElementKind
{
    AudioTerm = 0,
    Device = 1,
    Episode = 2,
    EpisodeTitle = 3,
    FileChecksum = 4,
    FileExtension = 5,
    Language = 6,
    Other = 7,
    Part = 8,
    ReleaseGroup = 9,
    ReleaseInformation = 10,
    ReleaseVersion = 11,
    Season = 12,
    Source = 13,
    Subtitles = 14,
    Title = 15,
    Type = 16,
    VideoResolution = 17,
    VideoTerm = 18,
    Volume = 19,
    Year = 20,
}
