/*
 * Copyright (C) 2019 Intel Corporation. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0 WITH LLVM-exception
 */

//! # WAMR Rust SDK
//!
//! ## Overview
//!
//! WAMR Rust SDK provides Rust language bindings for WAMR. It is the wrapper
//! of [*wasm_export.h*](../../../core/iwasm/include/wasm_export.h) but with Rust style.
//! It is more convenient to use WAMR in Rust with this crate.
//!
//! This crate contains API used to interact with Wasm modules. You can compile
//! modules, instantiate modules, call their export functions, etc.
//! Plus, as an embedded of Wasm, you can provide Wasm module functionality by
//! creating host-defined functions.
//!
//! WAMR Rust SDK includes a [*wamr-sys*](../crates/wamr-sys) crate. It searches for
//! the WAMR runtime source in the path *../..* and compiles it with CMake. The
//! Rust bindings are pre-generated and committed, so building does not require
//! `bindgen` or `libclang`.
//!
//! This crate has similar concepts to the
//! [WebAssembly specification](https://webassembly.github.io/spec/core/).
//!
//! ### Core concepts
//!
//! - *Runtime*. It is the environment that hosts all the wasm modules. Each process has one runtime instance.
//! - *Module*. It is the compiled .wasm or .aot. It can be loaded into runtime and instantiated into instance.
//! - *Instance*. It is the running instance of a module. It can be used to call export functions.
//! - *Function*. It is the exported function.
//!
//! ### WASI concepts
//!
//! - *WASIArgs*. It is used to configure the WASI environment.
//!   - *pre-open*. All files and directories in the list will be opened before the .wasm or .aot loaded.
//!   - *allowed address*. All ip addresses in the *allowed address* list will be allowed to connect with a socket.
//!   - *allowed DNS*.
//!
//! ### WAMR private concepts
//!
//! - *loading linking* instead of *instantiation linking*. *instantiation linking* is
//!   used in Wasm JS API and Wasm C API. It means that every instance has its own, maybe
//!   variant, imports. But *loading linking* means that all instances share the same *imports*.
//!
//! - *RuntimeArg*. Control runtime behavior.
//!   - *running mode*.
//!   - *allocator*.
//!
//! - *NativeFunction*.
//!
//! - *WasmValues*.
//!
//! ## Examples
//!
//! ### Example: to run a wasm32-wasip1 .wasm
//!
//! *wasm32-wasip1* is a most common target for Wasm. It means that the .wasm is compiled with
//! `cargo build --target wasm32-wasip1` or `wasi-sdk/bin/clang --target wasm32-wasip1`.
//!
//! Say there is a gcd_wasm32_wasi.wasm which includes a function named *gcd*. It returns the GCD
//! of two parameters.
//!
//! The rust code to call the function would be:
//!
//! ```
//! use wamr_rust_sdk::{
//!     runtime::Runtime, module::Module, instance::Instance, function::Function,
//!     value::WasmValue, RuntimeError
//! };
//!
//! fn main() -> Result<(), RuntimeError> {
//!     let runtime = Runtime::new()?;
//!
//!     let binary = include_bytes!("../resources/test/gcd_wasm32_wasi.wasm").to_vec();
//!     let module = Module::from_vec(&runtime, binary, "gcd_wasm32_wasi.wasm")?;
//!
//!     let instance = Instance::new(&runtime, &module, 1024 * 64)?;
//!
//!     let function = Function::find_export_func(&instance, "gcd")?;
//!
//!     let params: Vec<WasmValue> = vec![WasmValue::I32(9), WasmValue::I32(27)];
//!     let result = function.call(&instance, &params)?;
//!     assert_eq!(result, vec![WasmValue::I32(9)]);
//!
//!     Ok(())
//! }
//! ```
//!
//! ### Example: more configuration for runtime.
//!
//! With more configuration, runtime is capable to run .wasm with variant features, like
//! - Wasm without WASI requirement. Usually, it means that the .wasm is compiled with `-nostdlib`
//!   or `--target wasm32-unknown-unknown`
//! - Configure runtime.
//! - Provides host-defined functions to meet import requirements.
//!
//! Say there is an add_extra_wasm32_wasi.wasm. Its exported function, `add()`,
//! requires an imported function, `extra()`, during the execution. The `add()`
//! adds two parameters and the result of `extra()` . It is like `a + b + extra()`.
//!
//! The rust code to call the *add* function is like this:
//!
//! ```
//! use wamr_rust_sdk::{
//!     runtime::Runtime, module::Module, instance::Instance, function::Function,
//!     value::WasmValue, RuntimeError
//! };
//! use core::ffi::c_void;
//!
//! extern "C" fn extra() -> i32 {
//!     100
//! }
//!
//! fn main() -> Result<(), RuntimeError> {
//!     let runtime = Runtime::builder()
//!         .use_system_allocator()
//!         .register_host_function("extra", extra as *mut c_void)
//!         .build()?;
//!
//!     let binary = include_bytes!("../resources/test/add_extra_wasm32_wasi.wasm").to_vec();
//!     let module = Module::from_vec(&runtime, binary, "add_extra_wasm32_wasi.wasm")?;
//!
//!     let instance = Instance::new(&runtime, &module, 1024 * 64)?;
//!
//!     let function = Function::find_export_func(&instance, "add")?;
//!
//!     let params: Vec<WasmValue> = vec![WasmValue::I32(9), WasmValue::I32(27)];
//!     let result = function.call(&instance, &params)?;
//!     assert_eq!(result, vec![WasmValue::I32(136)]);
//!
//!     Ok(())
//! }
//! ```
//!

//! ## `no_std` support
//!
//! By default this crate links against `std`. To use it in a `no_std`
//! environment (e.g. embedded targets) disable the default features:
//!
//! ```toml
//! wamr-rust-sdk = { version = "1.0.0", default-features = false }
//! ```
//!
//! With `std` disabled the crate only depends on `core` and `alloc`, so a
//! global allocator is still required. File based helpers such as
//! `Module::from_file` are not available in that configuration; use
//! `Module::from_vec` together with your own loader instead.
//!

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use alloc::string::String;

pub use wamr_sys as sys;

pub mod function;
mod helper;
pub mod host_function;
pub mod instance;
pub mod module;
pub mod runtime;
pub mod value;
pub mod wasi_context;

/// Error raised when calling a wasm function fails.
#[derive(Debug, thiserror::Error)]
#[error("{message} and {exit_code}")]
pub struct ExecError {
    pub message: String,
    pub exit_code: u32,
}

/// all kinds of exceptions raised by WAMR
#[derive(Debug, thiserror::Error)]
pub enum RuntimeError {
    #[error("Not implemented")]
    NotImplemented,
    /// Runtime initialization error.
    #[error("Runtime initialization failure")]
    InitializationFailure,
    /// file operation error. usually while loading(compilation) a .wasm
    #[cfg(feature = "std")]
    #[error("Wasm file operation error: {0}")]
    WasmFileFSError(#[from] std::io::Error),
    /// A compilation error. usually means that the .wasm file is invalid
    #[error("Wasm compilation error: {0}")]
    CompilationError(String),
    /// instantiation failure
    #[error("Wasm instantiation failure: {0}")]
    InstantiationFailure(String),
    /// Error during execute wasm functions
    #[error("Wasm execution error: {0}")]
    ExecutionError(#[from] ExecError),
    /// usually returns by `find_export_func()`
    #[error("Function not found")]
    FunctionNotFound,
}
