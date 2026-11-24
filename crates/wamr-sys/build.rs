/*
 * Copyright (C) 2023 Liquid Reply GmbH. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0 WITH LLVM-exception
 */

extern crate cmake;

use cmake::Config;
use std::env;
#[cfg(feature = "wamrc")]
use std::path::Path;
use std::path::PathBuf;

const LLVM_LIBRARIES: &[&str] = &[
    // keep alphabet order
    "LLVMOrcJIT",
    "LLVMOrcShared",
    "LLVMOrcTargetProcess",
    "LLVMPasses",
    "LLVMProfileData",
    "LLVMRuntimeDyld",
    "LLVMScalarOpts",
    "LLVMSelectionDAG",
    "LLVMSymbolize",
    "LLVMTarget",
    "LLVMTextAPI",
    "LLVMTransformUtils",
    "LLVMVectorize",
    "LLVMX86AsmParser",
    "LLVMX86CodeGen",
    "LLVMX86Desc",
    "LLVMX86Disassembler",
    "LLVMX86Info",
    "LLVMXRay",
    "LLVMipo",
];

fn get_feature_flags() -> (String, String, String, String, String, String) {
    let enable_custom_section = if cfg!(feature = "custom-section") {
        "1"
    } else {
        "0"
    };
    let enable_dump_call_stack = if cfg!(feature = "dump-call-stack") {
        "1"
    } else {
        "0"
    };
    let enable_llvm_jit = if cfg!(feature = "llvmjit") { "1" } else { "0" };
    let enable_multi_module = if cfg!(feature = "multi-module") {
        "1"
    } else {
        "0"
    };
    let enable_name_section = if cfg!(feature = "name-section") {
        "1"
    } else {
        "0"
    };
    let disable_hw_bound_check = if cfg!(feature = "hw-bound-check") {
        "0"
    } else {
        "1"
    };

    (
        enable_custom_section.to_string(),
        enable_dump_call_stack.to_string(),
        enable_llvm_jit.to_string(),
        enable_multi_module.to_string(),
        enable_name_section.to_string(),
        disable_hw_bound_check.to_string(),
    )
}

fn link_llvm_libraries(llvm_cfg_path: &String, enable_llvm_jit: &String) {
    if enable_llvm_jit == "0" {
        return;
    }

    let llvm_cfg_path = PathBuf::from(llvm_cfg_path);
    assert!(llvm_cfg_path.exists());

    let llvm_lib_path = llvm_cfg_path.join("../../../lib").canonicalize().unwrap();
    assert!(llvm_lib_path.exists());

    println!("cargo:rustc-link-lib=dylib=dl");
    println!("cargo:rustc-link-lib=dylib=m");
    println!("cargo:rustc-link-lib=dylib=rt");
    println!("cargo:rustc-link-lib=dylib=stdc++");
    println!("cargo:rustc-link-lib=dylib=z");
    println!("cargo:libdir={}", llvm_lib_path.display());
    println!("cargo:rustc-link-search=native={}", llvm_lib_path.display());

    for &llvm_lib in LLVM_LIBRARIES {
        println!("cargo:rustc-link-lib=static={}", llvm_lib);
    }
}

fn setup_config(
    wamr_root: &PathBuf,
    feature_flags: (String, String, String, String, String, String),
) -> Config {
    let (
        enable_custom_section,
        enable_dump_call_stack,
        enable_llvm_jit,
        enable_multi_module,
        enable_name_section,
        disalbe_hw_bound_check,
    ) = feature_flags;

    let enable_interp = if cfg!(feature = "aot-only") { "0" } else { "1" };

    let mut cfg = Config::new(wamr_root);
    // WAMR uses C11 `_Alignof`/`_Static_assert`; MSVC only enables them with
    // `/std:c11`.
    if std::env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("msvc") {
        cfg.cflag("/std:c11");
    }
    cfg.define("WAMR_BUILD_AOT", "1")
        .define("WAMR_BUILD_INTERP", enable_interp)
        .define("WAMR_BUILD_FAST_INTERP", enable_interp)
        .define("WAMR_BUILD_JIT", &enable_llvm_jit)
        .define("WAMR_BUILD_BULK_MEMORY", "1")
        .define("WAMR_BUILD_REF_TYPES", "1")
        .define("WAMR_BUILD_SIMD", "1")
        .define("WAMR_BUILD_LIBC_WASI", "1")
        .define("WAMR_BUILD_LIBC_BUILTIN", "0")
        .define("WAMR_DISABLE_HW_BOUND_CHECK", &disalbe_hw_bound_check)
        .define("WAMR_BUILD_MULTI_MODULE", &enable_multi_module)
        .define("WAMR_BUILD_DUMP_CALL_STACK", &enable_dump_call_stack)
        .define("WAMR_BUILD_CUSTOM_NAME_SECTION", &enable_name_section)
        .define("WAMR_BUILD_LOAD_CUSTOM_SECTION", &enable_custom_section);

    // always assume non-empty strings for these environment variables

    if let Ok(platform_name) = env::var("WAMR_BUILD_PLATFORM") {
        cfg.define("WAMR_BUILD_PLATFORM", &platform_name);
    }

    if let Ok(target_name) = env::var("WAMR_BUILD_TARGET") {
        cfg.define("WAMR_BUILD_TARGET", &target_name);
    }

    if let Ok(platform_config) = env::var("WAMR_SHARED_PLATFORM_CONFIG") {
        cfg.define("SHARED_PLATFORM_CONFIG", &platform_config);
    }

    if let Ok(llvm_cfg_path) = env::var("LLVM_LIB_CFG_PATH") {
        link_llvm_libraries(&llvm_cfg_path, &enable_llvm_jit);
        cfg.define("LLVM_DIR", &llvm_cfg_path);
    }

    // STDIN/STDOUT/STDERR redirect
    if let Ok(bh_vprintf) = env::var("WAMR_BH_VPRINTF") {
        cfg.define("WAMR_BH_VPRINTF", &bh_vprintf);
    }

    cfg
}

fn build_wamr_libraries(wamr_root: &PathBuf) {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let vmbuild_path = out_dir.join("vmbuild");

    let feature_flags = get_feature_flags();
    let mut cfg = setup_config(wamr_root, feature_flags);
    let dst = cfg.out_dir(vmbuild_path).build_target("vmlib").build();

    println!("cargo:rustc-link-search=native={}/build", dst.display());
    // Multi-config generators (Visual Studio, Xcode) place the archive in a
    // per-config subdirectory.
    println!(
        "cargo:rustc-link-search=native={}/build/Debug",
        dst.display()
    );
    println!(
        "cargo:rustc-link-search=native={}/build/Release",
        dst.display()
    );
    println!("cargo:rustc-link-lib=static=iwasm");
}

#[cfg(feature = "wamrc")]
fn build_wamrc(wamr_root: &Path) {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let wamrc_build_path = out_dir.join("wamrcbuild");

    let wamr_compiler_path = wamr_root.join("wamr-compiler");
    assert!(wamr_compiler_path.exists());

    // Keep `wamrc` consistent with the runtime build. This also avoids the
    // Zydis `FetchContent` on Windows/x86_64, which requires network access.
    let disable_hw_bound_check = if cfg!(feature = "hw-bound-check") {
        "0"
    } else {
        "1"
    };

    Config::new(&wamr_compiler_path)
        .out_dir(wamrc_build_path)
        .define("WAMR_BUILD_WITH_CUSTOM_LLVM", "1")
        .define("WAMR_DISABLE_HW_BOUND_CHECK", disable_hw_bound_check)
        .define(
            "LLVM_DIR",
            env::var("LLVM_LIB_CFG_PATH")
                .expect("LLVM_LIB_CFG_PATH isn't specified in config.toml"),
        )
        .build();
}

fn main() {
    println!("cargo:rerun-if-env-changed=WAMR_BUILD_PLATFORM");
    println!("cargo:rerun-if-env-changed=WAMR_SHARED_PLATFORM_CONFIG");
    println!("cargo:rerun-if-env-changed=LLVM_LIB_CFG_PATH");
    println!("cargo:rerun-if-env-changed=WAMR_BH_VPRINTF");

    let wamr_root = env::current_dir().unwrap();
    let wamr_root = wamr_root.join("wasm-micro-runtime");
    assert!(wamr_root.exists());

    build_wamr_libraries(&wamr_root);
    #[cfg(feature = "wamrc")]
    build_wamrc(&wamr_root);
}
