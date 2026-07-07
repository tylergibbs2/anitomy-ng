# AnitomyNg

.NET bindings for [anitomy-ng](https://github.com/tylergibbs2/anitomy-ng), an
anime video filename parser. Prebuilt native binaries ship inside the package,
so there's no Rust toolchain or native build step for consumers.

```sh
dotnet add package AnitomyNg
```

```csharp
using AnitomyNg;

foreach (var element in Anitomy.Parse(
    "[TaigaSubs]_Toradora!_(2008)_-_01v2_-_Tiger_and_Dragon_[1280x720_H.264_FLAC][1234ABCD].mkv"))
{
    Console.WriteLine($"{element.Kind}: {element.Value}");
}
```

Disable specific categories with `Options`:

```csharp
var elements = Anitomy.Parse(filename, new Options { ParseTitle = false });
```

`Parse` returns an ordered `IReadOnlyList<Element>`; each `Element` has a
`Kind` (`ElementKind`), `Value`, and `Position`. `Position` is a Unicode
scalar (codepoint) index, not a UTF-16 `string` offset.

## Supported platforms

Prebuilt natives are included for `win-x64`, `linux-x64`, `linux-arm64`,
`osx-x64`, and `osx-arm64`. Requires .NET 8 or later.
