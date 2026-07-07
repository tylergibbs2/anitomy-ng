# anitomy-ng-c

The C ABI for [anitomy-ng](https://github.com/tylergibbs2/anitomy-ng), an anime
video filename parser. This crate is the foundation for non-Rust bindings (C#,
C/C++, Go, …): it exposes a small, stable `extern "C"` surface over the pure-Rust
core.

The core `anitomy-ng` crate is `#![forbid(unsafe_code)]`. **All FFI `unsafe`
is isolated in this crate**, at the boundary; the parser itself keeps its
no-unsafe guarantee.

## Building

```sh
cargo build -p anitomy-ng-c --release
```

Produces a shared library (`libanitomy.so` / `libanitomy.dylib` / `anitomy.dll`)
and a static library (`libanitomy.a`). The C header is generated to
`include/anitomy.h` by `build.rs` (via cbindgen).

## ABI at a glance

```c
AnitomyResult *anitomy_parse(const char *input_utf8, uint32_t options);
size_t         anitomy_result_len(const AnitomyResult *r);
uint32_t       anitomy_result_kind(const AnitomyResult *r, size_t i);
const char    *anitomy_result_value(const AnitomyResult *r, size_t i);
size_t         anitomy_result_position(const AnitomyResult *r, size_t i);
void           anitomy_result_free(AnitomyResult *r);

uint32_t       anitomy_options_default(void);
const char    *anitomy_kind_name(uint32_t kind);
const char    *anitomy_version(void);
```

Ownership: `anitomy_parse` returns an owning handle; read fields through the
accessors, then call `anitomy_result_free` exactly once. Strings from
`anitomy_result_value` are borrowed from the result and valid until it is freed.
Memory allocated by Rust is freed only by Rust. Options are a bitmask of the
`ANITOMY_OPTION_*` bits (or `anitomy_options_default()` for all-enabled).
