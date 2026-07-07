// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Generates `include/anitomy.h` from the crate's `extern "C"` definitions via
//! cbindgen, so the C header can't drift from the ABI.
//!
//! Best-effort: if cbindgen fails for any reason it emits a warning but does
//! not fail the build — the shared library itself doesn't depend on the header.

use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=src/lib.rs");
    println!("cargo:rerun-if-changed=cbindgen.toml");

    let crate_dir = match std::env::var("CARGO_MANIFEST_DIR") {
        Ok(dir) => PathBuf::from(dir),
        Err(_) => return,
    };
    let include_dir = crate_dir.join("include");
    let header = include_dir.join("anitomy.h");

    if let Err(err) = std::fs::create_dir_all(&include_dir) {
        println!("cargo:warning=anitomy-c: could not create include/: {err}");
        return;
    }

    match cbindgen::generate(&crate_dir) {
        Ok(bindings) => {
            bindings.write_to_file(&header);
        }
        Err(err) => {
            println!("cargo:warning=anitomy-c: cbindgen header generation failed: {err}");
        }
    }
}
