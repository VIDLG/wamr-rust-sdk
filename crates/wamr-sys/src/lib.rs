/*
 * Copyright (C) 2023 Liquid Reply GmbH. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0 WITH LLVM-exception
 */

// Suppress the flurry of warnings caused by using "C" naming conventions
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![cfg_attr(not(feature = "std"), no_std)]

// Pre-generated from `wasm_export.h` by the `wamr-sys-bindgen` crate.
// Regenerate with `cargo run -p wamr-sys-bindgen`.
include!("bindings.rs");
