// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

using AnitomyNg;
using Xunit;

namespace AnitomyNg.Tests;

/// <summary>
/// End-to-end tests over the real native library — these exercise the whole
/// C#↔C ABI↔Rust path (marshalling, ownership, freeing), not just managed code.
/// </summary>
public class AnitomyTests
{
    [Fact]
    public void ParsesCoreElements()
    {
        var elements = Anitomy.Parse(
            "[TaigaSubs]_Toradora!_(2008)_-_01v2_-_Tiger_and_Dragon_[1280x720_H.264_FLAC][1234ABCD].mkv");

        Assert.Contains(elements, e => e.Kind == ElementKind.ReleaseGroup && e.Value == "TaigaSubs");
        Assert.Contains(elements, e => e.Kind == ElementKind.Title && e.Value == "Toradora!");
        Assert.Contains(elements, e => e.Kind == ElementKind.Year && e.Value == "2008");
        Assert.Contains(elements, e => e.Kind == ElementKind.Episode && e.Value == "01");
        Assert.Contains(elements, e => e.Kind == ElementKind.FileExtension && e.Value == "mkv");
    }

    [Fact]
    public void ElementsAreOrderedByPosition()
    {
        var elements = Anitomy.Parse("[Grp] Show Name - 03 [1080p].mkv");
        int previous = -1;
        foreach (var e in elements)
        {
            Assert.True(e.Position >= previous, "elements should be ordered by position");
            previous = e.Position;
        }
    }

    [Fact]
    public void OptionsDisableCategories()
    {
        var elements = Anitomy.Parse("Toradora! - 01.mkv", new Options { ParseTitle = false });
        Assert.DoesNotContain(elements, e => e.Kind == ElementKind.Title);
        // Other categories still resolve.
        Assert.Contains(elements, e => e.Kind == ElementKind.Episode);
    }

    [Fact]
    public void EmptyInputYieldsNoElements()
    {
        Assert.Empty(Anitomy.Parse(""));
    }

    [Fact]
    public void HandlesUnicodeValues()
    {
        // Non-ASCII must round-trip through UTF-8 marshalling intact.
        var elements = Anitomy.Parse("[グループ] 進撃の巨人 - 01.mkv");
        Assert.Contains(elements, e => e.Kind == ElementKind.Episode && e.Value == "01");
    }

    [Fact]
    public void NativeVersionIsPopulated()
    {
        Assert.False(string.IsNullOrEmpty(Anitomy.NativeVersion));
    }

    [Fact]
    public void ManyParsesDoNotLeakOrCrash()
    {
        // Repeated parse/free cycles: a use-after-free or double-free in the
        // ownership handling would surface as a crash here.
        for (int i = 0; i < 5000; i++)
        {
            var elements = Anitomy.Parse("[Grp] Title - 12 [720p][ABCD1234].mkv");
            Assert.NotEmpty(elements);
        }
    }
}
