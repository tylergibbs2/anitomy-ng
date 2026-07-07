// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

// Benchmark adapter for tabratton/AnitomySharp (C#/.NET). Reads the corpus from
// stdin (one filename per line); emits JSONL — one {input, output} object per
// line — with old-schema anitomy category keys (schema=old), plus a final
// {"__per_file_ns__": N} line with the median per-file parse time in this .NET
// process (parse only; startup and I/O excluded).
using System.Diagnostics;
using System.Text;
using System.Text.Json;

// "ElementAnimeTitle" -> "anime_title": strip the "Element" prefix, then
// PascalCase -> snake_case. Matches scripts/benchmark.py's OLD_KEY_MAP keys.
static string ToSnake(string category)
{
    var s = category.StartsWith("Element") ? category["Element".Length..] : category;
    var sb = new StringBuilder(s.Length + 8);
    for (var i = 0; i < s.Length; i++)
    {
        if (char.IsUpper(s[i]) && i > 0) sb.Append('_');
        sb.Append(char.ToLowerInvariant(s[i]));
    }
    return sb.ToString();
}

var inputs = new List<string>();
string? line;
while ((line = Console.In.ReadLine()) is not null)
    if (line.Length > 0) inputs.Add(line);

var outBuf = new StringBuilder();
foreach (var input in inputs)
{
    var grouped = new Dictionary<string, List<string>>();
    try
    {
        foreach (var e in AnitomySharp.AnitomySharp.Parse(input))
        {
            var key = ToSnake(e.Category.ToString());
            if (!grouped.TryGetValue(key, out var vals)) grouped[key] = vals = new List<string>();
            vals.Add(e.Value);
        }
    }
    catch
    {
        grouped.Clear();
    }
    outBuf.AppendLine(JsonSerializer.Serialize(new { input, output = grouped }));
}

const int timedPasses = 200;
static void ParseAll(List<string> xs)
{
    foreach (var x in xs)
        foreach (var _ in AnitomySharp.AnitomySharp.Parse(x)) { }
}

for (var w = 0; w < 5; w++) ParseAll(inputs); // warmup
var passNs = new List<double>(timedPasses);
for (var p = 0; p < timedPasses; p++)
{
    var sw = Stopwatch.StartNew();
    ParseAll(inputs);
    sw.Stop();
    passNs.Add(sw.Elapsed.TotalNanoseconds);
}
passNs.Sort();
var perFileNs = passNs[passNs.Count / 2] / inputs.Count;
outBuf.AppendLine(JsonSerializer.Serialize(new Dictionary<string, double> { ["__per_file_ns__"] = perFileNs }));

Console.Out.Write(outBuf.ToString());
