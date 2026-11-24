# Project commands.
#
# This crate embeds a C runtime, so the build needs extra tooling (CMake,
# Ninja, LLVM). Run the recipes through pixi so it is on `PATH`:
#
#     pixi run just <recipe>
#
# The pixi tasks (`pixi run build`, `pixi run test`, ...) call these recipes.

# `wamr-sys` reads the LLVM CMake config from `LLVM_LIB_CFG_PATH`, and the
# bindings regenerator locates libclang through `LIBCLANG_PATH`. When running
# through pixi, derive both from the active environment so nothing needs to be
# set by hand.
conda_prefix := env_var_or_default("CONDA_PREFIX", "")

llvm_cfg := if conda_prefix != "" {
    if os() == "windows" {
        conda_prefix + "/Library/lib/cmake/llvm"
    } else {
        conda_prefix + "/lib/cmake/llvm"
    }
} else {
    env_var_or_default("LLVM_LIB_CFG_PATH", "")
}

libclang := if conda_prefix != "" {
    if os() == "windows" {
        conda_prefix + "/Library/bin"
    } else {
        conda_prefix + "/lib"
    }
} else {
    env_var_or_default("LIBCLANG_PATH", "")
}

export LLVM_LIB_CFG_PATH := llvm_cfg
export LIBCLANG_PATH := libclang

# List the available recipes.
default:
    @just --list

# Build the C runtime and the Rust SDK.
build:
    cargo build

# Type-check the workspace with the default (`std`) features.
check:
    cargo check --workspace

# Type-check the SDK in `no_std` mode (core + alloc only).
check-no-std:
    cargo check -p wamr-rust-sdk --no-default-features

# Run unit tests (wamrc-dependent tests are excluded).
test:
    cargo test --lib

# Run the AOT tests too; builds `wamrc` and therefore needs LLVM.
test-aot:
    cargo test --lib --features wamrc

# Run unit tests including the `#[ignore]`d ones (requires a built `wamrc`).
test-all:
    cargo test --lib --features wamrc -- --ignored --test-threads 1

# Regenerate `crates/wamr-sys/src/bindings.rs` (requires libclang).
bindings:
    cargo run -p wamr-sys-bindgen

# Format all code.
fmt:
    cargo fmt --all

# Run clippy over all targets.
clippy:
    cargo clippy --workspace --all-targets
