// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

using System.Runtime.InteropServices;

namespace AnitomyNg;

/// <summary>
/// Parses anime video filenames into their elements. Wraps the native
/// anitomy-ng parser; all interop and native memory management is contained
/// here, so callers only ever see managed types.
/// </summary>
public static class Anitomy
{
    /// <summary>
    /// Parses <paramref name="filename"/> into an ordered list of elements
    /// (by position). There may be multiple elements of the same kind (e.g. an
    /// episode range yields two <see cref="ElementKind.Episode"/> values).
    /// </summary>
    /// <param name="filename">The filename to parse.</param>
    /// <param name="options">
    /// Which categories to extract; defaults to <see cref="Options.Default"/>
    /// (all enabled).
    /// </param>
    /// <returns>The parsed elements, or an empty list if nothing was found.</returns>
    public static IReadOnlyList<Element> Parse(string filename, Options? options = null)
    {
        ArgumentNullException.ThrowIfNull(filename);

        uint mask = (options ?? Options.Default).ToBitmask();
        nint result = NativeMethods.anitomy_parse(filename, mask);
        if (result == nint.Zero)
        {
            return Array.Empty<Element>();
        }

        // try/finally guarantees the native handle is freed even if reading a
        // value throws — the pointer never escapes this method.
        try
        {
            return ReadElements(result);
        }
        finally
        {
            NativeMethods.anitomy_result_free(result);
        }
    }

    /// <summary>
    /// Parses a set of related filenames together, returning one element list
    /// per input in the same order (result <c>i</c> is for
    /// <paramref name="filenames"/><c>[i]</c>). The shared context resolves
    /// ambiguities a single filename can't — e.g. a directory batch range vs.
    /// the real per-file episode, or a series title that lives only in a parent
    /// folder.
    /// </summary>
    /// <param name="filenames">The filenames to parse together.</param>
    /// <param name="options">
    /// Which categories to extract; defaults to <see cref="Options.Default"/>
    /// (all enabled).
    /// </param>
    /// <returns>One element list per input, in input order.</returns>
    public static IReadOnlyList<IReadOnlyList<Element>> ParseTogether(
        IReadOnlyList<string> filenames, Options? options = null)
    {
        ArgumentNullException.ThrowIfNull(filenames);

        int count = filenames.Count;
        uint mask = (options ?? Options.Default).ToBitmask();

        // Marshal each filename to a native UTF-8 string and gather the pointers
        // into a blittable array we can pin as the native char**.
        nint[] pointers = new nint[count];
        for (int i = 0; i < count; i++)
        {
            pointers[i] = Marshal.StringToCoTaskMemUTF8(filenames[i] ?? string.Empty);
        }

        GCHandle pinned = GCHandle.Alloc(pointers, GCHandleType.Pinned);
        try
        {
            nint batch = NativeMethods.anitomy_parse_together(
                pinned.AddrOfPinnedObject(), (nuint)count, mask);
            if (batch == nint.Zero)
            {
                return Array.Empty<IReadOnlyList<Element>>();
            }

            try
            {
                nuint len = NativeMethods.anitomy_results_len(batch);
                var results = new List<IReadOnlyList<Element>>((int)len);
                for (nuint i = 0; i < len; i++)
                {
                    // Borrowed sub-result, owned by the batch — never freed here.
                    results.Add(ReadElements(NativeMethods.anitomy_results_get(batch, i)));
                }
                return results;
            }
            finally
            {
                NativeMethods.anitomy_results_free(batch);
            }
        }
        finally
        {
            pinned.Free();
            for (int i = 0; i < count; i++)
            {
                Marshal.FreeCoTaskMem(pointers[i]);
            }
        }
    }

    /// <summary>
    /// Reads every element out of a (borrowed or owned) native result handle.
    /// Does not free the handle — ownership stays with the caller.
    /// </summary>
    private static IReadOnlyList<Element> ReadElements(nint result)
    {
        if (result == nint.Zero)
        {
            return Array.Empty<Element>();
        }

        nuint len = NativeMethods.anitomy_result_len(result);
        var elements = new List<Element>((int)len);
        for (nuint i = 0; i < len; i++)
        {
            var kind = (ElementKind)NativeMethods.anitomy_result_kind(result, i);
            nint valuePtr = NativeMethods.anitomy_result_value(result, i);
            string value = Marshal.PtrToStringUTF8(valuePtr) ?? string.Empty;
            int position = (int)NativeMethods.anitomy_result_position(result, i);
            elements.Add(new Element(kind, value, position));
        }
        return elements;
    }

    /// <summary>The version of the underlying native anitomy-ng library.</summary>
    public static string NativeVersion =>
        Marshal.PtrToStringUTF8(NativeMethods.anitomy_version()) ?? string.Empty;
}
