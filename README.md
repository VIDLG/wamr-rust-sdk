# wamr-rust-sdk

## WAMR Rust SDK

### Overview

WAMR Rust SDK provides Rust language bindings for WAMR. It is the wrapper
of [*wasm_export.h*](../../../core/iwasm/include/wasm_export.h) but with Rust style.
It is more convenient to use WAMR in Rust with this crate.

This crate contains API used to interact with Wasm modules. You can compile
modules, instantiate modules, call their export functions, etc.
Plus, as an embedded of Wasm, you can provide Wasm module functionality by
creating host-defined functions.

WAMR Rust SDK includes a [*wamr-sys*](../crates/wamr-sys) crate. It searches for
the WAMR runtime source in the path *../..* and compiles it with CMake. The
Rust bindings are pre-generated and committed, so building does not require
`bindgen` or `libclang`.

This crate has similar concepts to the
[WebAssembly specification](https://webassembly.github.io/spec/core/).

#### Core concepts

- *Runtime*. It is the environment that hosts all the wasm modules. Each process has one runtime instance.
- *Module*. It is the compiled .wasm or .aot. It can be loaded into runtime and instantiated into instance.
- *Instance*. It is the running instance of a module. It can be used to call export functions.
- *Function*. It is the exported function.

#### WASI concepts

- *WASIArgs*. It is used to configure the WASI environment.
  - *pre-open*. All files and directories in the list will be opened before the .wasm or .aot loaded.
  - *allowed address*. All ip addresses in the *allowed address* list will be allowed to connect with a socket.
  - *allowed DNS*.

#### WAMR private concepts

- *loading linking* instead of *instantiation linking*. *instantiation linking* is
used in Wasm JS API and Wasm C API. It means that every instance has its own, maybe
variant, imports. But *loading linking* means that all instances share the same *imports*.

- *RuntimeArg*. Control runtime behavior.
  - *running mode*.
  - *allocator*.

- *NativeFunction*.

- *WasmValues*.

### Examples

#### Example: to run a wasm32-wasip1 .wasm

*wasm32-wasip1* is a most common target for Wasm. It means that the .wasm is compiled with
`cargo build --target wasm32-wasip1` or `wasi-sdk/bin/clang --target wasm32-wasip1`.

Say there is a gcd_wasm32_wasi.wasm which includes a function named *gcd*. It returns the GCD
of two parameters.

The rust code to call the function would be:

```rust
use wamr_rust_sdk::{
    runtime::Runtime, module::Module, instance::Instance, function::Function,
    value::WasmValue, RuntimeError
};
use std::path::PathBuf;

fn main() -> Result<(), RuntimeError> {
    let runtime = Runtime::new()?;

    let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    d.push("resources/test");
    d.push("gcd_wasm32_wasi.wasm");

    let module = Module::from_file(&runtime, d.as_path())?;

    let instance = Instance::new(&runtime, &module, 1024 * 64)?;

    let function = Function::find_export_func(&instance, "gcd")?;

    let params: Vec<WasmValue> = vec![WasmValue::I32(9), WasmValue::I32(27)];
    let result = function.call(&instance, &params)?;
    assert_eq!(result, WasmValue::I32(9));

    Ok(())
}
```

#### Example: more configuration for runtime

With more configuration, runtime is capable to run .wasm with variant features, like

- Wasm without WASI requirement. Usually, it means that the .wasm is compiled with `-nostdlib`
  or `--target wasm32-unknown-unknown`
- Configure runtime.
- Provides host-defined functions to meet import requirements.

Say there is an add_extra_wasm32_wasi.wasm. Its exported function, `add()`,
requires an imported function, `extra()`, during the execution. The `add()`
adds two parameters and the result of `extra()` . It is like `a + b + extra()`.

The rust code to call the *add* function is like this:

```rust
use wamr_rust_sdk::{
    runtime::Runtime, module::Module, instance::Instance, function::Function,
    value::WasmValue, RuntimeError
};
use std::path::PathBuf;
use std::ffi::c_void;

extern "C" fn extra() -> i32 {
    100
}

fn main() -> Result<(), RuntimeError> {
    let runtime = Runtime::builder()
        .use_system_allocator()
        .register_host_function("extra", extra as *mut c_void)
        .build()?;

    let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    d.push("resources/test");
    d.push("add_extra_wasm32_wasi.wasm");
    let module = Module::from_file(&runtime, d.as_path())?;

    let instance = Instance::new(&runtime, &module, 1024 * 64)?;

    let function = Function::find_export_func(&instance, "add")?;

    let params: Vec<WasmValue> = vec![WasmValue::I32(9), WasmValue::I32(27)];
    let result = function.call(&instance, &params)?;
    assert_eq!(result, WasmValue::I32(136));

    Ok(())
}
```

### `no_std` support

The crate links against `std` by default. For embedded targets, disable the
default features so it only depends on `core` and `alloc`:

```toml
wamr-rust-sdk = { version = "1.0.0", default-features = false }
```

A global allocator is still required, and the file based helpers such as
`Module::from_file` are not available in this configuration. Use
`Module::from_vec` with your own loader instead.

### Build Instructions

#### Development environment with pixi

`wamr-sys` compiles the WAMR C runtime with CMake and builds the `wamrc` AOT
compiler against LLVM. A [pixi](https://pixi.sh) environment provides CMake,
Ninja, LLVM and `just`:

```sh
pixi install
pixi run just check         # type-check (std)
pixi run just check-no-std  # type-check (no_std)
pixi run just test          # run unit tests
```

Every recipe is also available as a pixi task, e.g. `pixi run check`.

#### Build requirements

- Always: CMake and a C compiler. The CMake build is cached under `target/`,
  so only the first build is slow.
- Only with the `wamrc` feature: LLVM 18, used to build the `wamrc` AOT
  compiler. `wamrc` is never needed to *use* the SDK, only to precompile
  `.wasm` into `.aot` (e.g. for tests).

No prebuilt C library is shipped: WAMR has to be compiled against a platform
port that matches your OS/chip (see the bare-metal section below). The Rust
bindings in `crates/wamr-sys/src/bindings.rs` are pre-generated and committed,
so downstream builds do **not** need `bindgen` or `libclang`.

Maintainers can regenerate them (this does need `libclang`) with:

```sh
pixi run just bindings   # uses the pixi-provided libclang
```

#### Building `wamr-sys` and `wamr-rust-sdk`

To build the `wamr-sys` and `wamr-rust-sdk` crates, follow these steps:

1. Ensure you have the Rust toolchain installed.
2. Clone the repository:

   ```sh
   git clone https://github.com/bytecodealliance/wamr-rust-sdk.git
   cd wamr-rust-sdk
   ```

3. Build the `wamr-sys` crate:

   ```sh
   cargo build -p wamr-sys
   ```

4. Build the `wamr-rust-sdk` crate:

   ```sh
   cargo build
   ```

#### Preparing a Development and Building Environment

##### Native targets

1. Prepare the Rust toolchain.

2. If targeting a non-linux platform, set `WAMR_BUILD_TARGET` and `WAMR_BUILD_PLATFORM` in `.cargo/config.toml`:

   ```toml
   [env]
   WAMR_BUILD_PLATFORM = "OS name"
   WAMR_BUILD_TARGET = "CPU architecture"
   ```

3. If targeting a platform not supplied by WAMR, refer to the [WAMR porting guide](https://github.com/bytecodealliance/wasm-micro-runtime/blob/main/doc/port_wamr.md#wamr-porting-guide) and set `WAMR_BUILD_TARGET`, `WAMR_BUILD_PLATFORM` and `WAMR_SHARED_PLATFORM_CONFIG` in `.cargo/config.toml` properly.

##### Bare-metal targets (e.g. `esp-hal`)

This crate does not use esp-idf and does not target Xtensa. The supported
bare-metal, `no_std` use case is the RISC-V based ESP32-C series with
[`esp-hal`](https://github.com/esp-rs/esp-hal). Disable the default features:

```toml
wamr-rust-sdk = { git = "https://github.com/bytecodealliance/wamr-rust-sdk", default-features = false }
```

WAMR still needs a platform port for the target. Point the build at one via
`.cargo/config.toml`:

```toml
[env]
# the ESP32-C series is RISC-V
WAMR_BUILD_TARGET = "RISCV32"
# a custom platform implementation; see the WAMR porting guide
WAMR_SHARED_PLATFORM_CONFIG = "path/to/platform.cmake"
```

A global allocator is required in this configuration.

#### Precompiling wasm to AOT

For constrained targets, ship precompiled AOT modules instead of `.wasm`.
Build `wamrc` once and run it ahead of time:

```sh
cargo build --features wamrc
# wamrc ends up at target/debug/build/wamr-sys-*/out/wamrcbuild/bin/wamrc
wamrc -o app.aot app.wasm
```

Then load the bytes at runtime (e.g. read from flash) with `Module::from_vec`
(or `Module::from_file` when `std` is enabled):

```rust
let module = Module::from_vec(&runtime, aot_bytes, "app")?;
```

The device build then needs no LLVM, and the runtime does not spend time
compiling wasm.

When you only ever load precompiled `.aot` modules, disable the interpreter to
shrink the runtime:

```toml
wamr-rust-sdk = { git = "https://github.com/bytecodealliance/wamr-rust-sdk", default-features = false, features = ["aot-only"] }
```

With `aot-only`, loading a `.wasm` fails; only `.aot` modules are accepted.

#### BKMs

- To regenerate the bindings, `LIBCLANG_PATH` must point at the directory
  containing libclang.
- The `pixi` environment already provides CMake, Ninja, LLVM and `just`.
