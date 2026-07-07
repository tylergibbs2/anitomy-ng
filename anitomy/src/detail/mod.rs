// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Mirrors the `include/anitomy/detail/` layout upstream, one module per
//! header.

pub(crate) mod bracket;
pub(crate) mod container;
pub(crate) mod delimiter;
pub(crate) mod element;
pub(crate) mod keyword;
pub(crate) mod parser;
pub(crate) mod regex_util;
pub(crate) mod token;
pub(crate) mod tokenizer;
pub(crate) mod util;
