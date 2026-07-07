// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

namespace AnitomyNg;

/// <summary>One parsed element of a filename.</summary>
/// <param name="Kind">Which kind of element this is.</param>
/// <param name="Value">The matched (and cleaned) substring.</param>
/// <param name="Position">
/// The element's position in the input, counted in Unicode scalar values
/// (codepoints), matching the upstream parser. Note this is <b>not</b> a
/// <see cref="string"/> (UTF-16 code-unit) index, so it will differ from a
/// .NET string offset for characters outside the Basic Multilingual Plane.
/// </param>
public readonly record struct Element(ElementKind Kind, string Value, int Position);
