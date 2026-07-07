// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

namespace AnitomyNg;

/// <summary>
/// Which element kinds to extract. Every field defaults to <c>true</c>; set the
/// ones you want to skip to <c>false</c>. Mirrors <c>anitomy_ng::Options</c>.
/// </summary>
public sealed record Options
{
    /// <summary>Options with every category enabled (the default).</summary>
    public static Options Default { get; } = new();

    public bool ParseEpisode { get; init; } = true;
    public bool ParseEpisodeTitle { get; init; } = true;
    public bool ParseFileChecksum { get; init; } = true;
    public bool ParseFileExtension { get; init; } = true;
    public bool ParsePart { get; init; } = true;
    public bool ParseReleaseGroup { get; init; } = true;
    public bool ParseSeason { get; init; } = true;
    public bool ParseTitle { get; init; } = true;
    public bool ParseVideoResolution { get; init; } = true;
    public bool ParseYear { get; init; } = true;

    // Bit positions must match the ANITOMY_OPTION_* constants in anitomy-c.
    private const uint Episode = 1u << 0;
    private const uint EpisodeTitle = 1u << 1;
    private const uint FileChecksum = 1u << 2;
    private const uint FileExtension = 1u << 3;
    private const uint Part = 1u << 4;
    private const uint ReleaseGroup = 1u << 5;
    private const uint Season = 1u << 6;
    private const uint Title = 1u << 7;
    private const uint VideoResolution = 1u << 8;
    private const uint Year = 1u << 9;

    /// <summary>Encodes these options as the C ABI option bitmask.</summary>
    internal uint ToBitmask()
    {
        uint bits = 0;
        if (ParseEpisode) bits |= Episode;
        if (ParseEpisodeTitle) bits |= EpisodeTitle;
        if (ParseFileChecksum) bits |= FileChecksum;
        if (ParseFileExtension) bits |= FileExtension;
        if (ParsePart) bits |= Part;
        if (ParseReleaseGroup) bits |= ReleaseGroup;
        if (ParseSeason) bits |= Season;
        if (ParseTitle) bits |= Title;
        if (ParseVideoResolution) bits |= VideoResolution;
        if (ParseYear) bits |= Year;
        return bits;
    }
}
