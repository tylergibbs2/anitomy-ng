// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

using System.Text;
using AnitomyNg;
using Xunit;

namespace AnitomyNg.Tests;

/// <summary>
/// <see cref="ElementKind"/> and <see cref="Options"/>' bit positions are
/// hand-mirrored from anitomy-c. Nothing but these tests ties them to it, and
/// both drift silently: a stale enum yields wrongly-labelled elements, a stale
/// bit disables the wrong category.
/// </summary>
public class SyncTests
{
    /// One filename that produces all ten option-gated categories.
    private const string AllCategories =
        "[Grp] Show Name (2019) S02 (Part 2) - 05 - Episode Name [1080p][ABCD1234].mkv";

    private static string ToSnakeCase(string pascal)
    {
        var sb = new StringBuilder();
        foreach (char c in pascal)
        {
            if (char.IsUpper(c) && sb.Length > 0) sb.Append('_');
            sb.Append(char.ToLowerInvariant(c));
        }
        return sb.ToString();
    }

    [Fact]
    public void EveryKindMatchesItsNativeName()
    {
        foreach (var kind in Enum.GetValues<ElementKind>())
        {
            Assert.Equal(ToSnakeCase(kind.ToString()), Anitomy.KindName(kind));
        }
    }

    [Fact]
    public void KindDiscriminantsAreDenseAndComplete()
    {
        var values = Enum.GetValues<ElementKind>().Select(k => (uint)k).OrderBy(v => v).ToArray();
        Assert.Equal(Enumerable.Range(0, values.Length).Select(i => (uint)i), values);
        // One past the end must be unknown to the native side, so the managed
        // enum isn't missing a variant the native library can still emit.
        Assert.Equal(string.Empty, Anitomy.KindName((ElementKind)values.Length));
    }

    [Fact]
    public void DefaultOptionsMatchNativeBitmask()
    {
        Assert.Equal(NativeMethods.anitomy_options_default(), Options.Default.ToBitmask());
    }

    [Theory]
    [InlineData(nameof(Options.ParseEpisode), ElementKind.Episode)]
    [InlineData(nameof(Options.ParseEpisodeTitle), ElementKind.EpisodeTitle)]
    [InlineData(nameof(Options.ParseFileChecksum), ElementKind.FileChecksum)]
    [InlineData(nameof(Options.ParseFileExtension), ElementKind.FileExtension)]
    [InlineData(nameof(Options.ParsePart), ElementKind.Part)]
    [InlineData(nameof(Options.ParseReleaseGroup), ElementKind.ReleaseGroup)]
    [InlineData(nameof(Options.ParseSeason), ElementKind.Season)]
    [InlineData(nameof(Options.ParseTitle), ElementKind.Title)]
    [InlineData(nameof(Options.ParseVideoResolution), ElementKind.VideoResolution)]
    [InlineData(nameof(Options.ParseYear), ElementKind.Year)]
    public void DisablingAnOptionRemovesExactlyItsKind(string property, ElementKind kind)
    {
        Assert.Contains(Anitomy.Parse(AllCategories), e => e.Kind == kind);
        Assert.DoesNotContain(
            Anitomy.Parse(AllCategories, WithDisabled(property)), e => e.Kind == kind);
    }

    private static Options WithDisabled(string property) => property switch
    {
        nameof(Options.ParseEpisode) => new Options { ParseEpisode = false },
        nameof(Options.ParseEpisodeTitle) => new Options { ParseEpisodeTitle = false },
        nameof(Options.ParseFileChecksum) => new Options { ParseFileChecksum = false },
        nameof(Options.ParseFileExtension) => new Options { ParseFileExtension = false },
        nameof(Options.ParsePart) => new Options { ParsePart = false },
        nameof(Options.ParseReleaseGroup) => new Options { ParseReleaseGroup = false },
        nameof(Options.ParseSeason) => new Options { ParseSeason = false },
        nameof(Options.ParseTitle) => new Options { ParseTitle = false },
        nameof(Options.ParseVideoResolution) => new Options { ParseVideoResolution = false },
        nameof(Options.ParseYear) => new Options { ParseYear = false },
        _ => throw new ArgumentOutOfRangeException(nameof(property)),
    };
}
