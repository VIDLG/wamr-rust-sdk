//! Regenerates `crates/wamr-sys/src/bindings.rs` from WAMR's `wasm_export.h`.
//!
//! Run from the workspace root:
//!
//! ```sh
//! cargo run -p wamr-sys-bindgen
//! ```
//!
//! This requires `libclang` (set `LIBCLANG_PATH` if it is not found
//! automatically; the `pixi` environment provides it). The generated file is
//! committed, so downstream users do not depend on `bindgen` or `libclang`.
//!
//! Optional arguments override the default header/output paths:
//!
//! ```sh
//! cargo run -p wamr-sys-bindgen -- path/to/wasm_export.h path/to/bindings.rs
//! ```

use std::path::PathBuf;
use std::process::Command;

fn main() {
    #[cfg(target_os = "windows")]
    prepare_libclang();

    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut args = std::env::args().skip(1);

    let header = args.next().map(PathBuf::from).unwrap_or_else(|| {
        manifest_dir.join("../wamr-sys/wasm-micro-runtime/core/iwasm/include/wasm_export.h")
    });
    let out = args
        .next()
        .map(PathBuf::from)
        .unwrap_or_else(|| manifest_dir.join("../wamr-sys/src/bindings.rs"));

    assert!(
        header.exists(),
        "header not found at {}; is the wasm-micro-runtime submodule initialized?",
        header.display()
    );

    let bindings = bindgen::Builder::default()
        .ctypes_prefix("::core::ffi")
        .use_core()
        .header(header.to_string_lossy().into_owned())
        .derive_default(true)
        // The bindings must be portable across targets: size/offset assertions
        // are generated for the host and break when cross-compiling (e.g. to
        // 32-bit RISC-V).
        .layout_tests(false)
        // Emit the WAMR API only, not the host CRT headers it includes; that
        // keeps the committed file platform independent.
        .allowlist_file(".*wasm_export\\.h")
        .generate()
        .expect("failed to generate bindings");

    bindings
        .write_to_file(&out)
        .expect("failed to write bindings");

    match Command::new("rustfmt")
        .args(["--edition", "2021"])
        .arg(&out)
        .status()
    {
        Ok(status) if status.success() => {}
        _ => eprintln!("warning: could not run `rustfmt`; format the file manually"),
    }

    println!("wrote {}", out.display());
}

/// conda-forge ships `libclang-<major>.dll` on Windows, but `clang-sys` only
/// looks for `libclang.dll`. Create an alias next to it (hard link, falling
/// back to a copy).
#[cfg(target_os = "windows")]
fn prepare_libclang() {
    let Some(dir) = std::env::var_os("LIBCLANG_PATH").map(PathBuf::from) else {
        return;
    };
    if dir.is_file() || dir.join("libclang.dll").exists() {
        return;
    }

    let Ok(entries) = std::fs::read_dir(&dir) else {
        return;
    };

    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_ascii_lowercase();
        if !(name.starts_with("libclang-") && name.ends_with(".dll")) {
            continue;
        }

        let source = entry.path();
        let alias = dir.join("libclang.dll");
        if std::fs::hard_link(&source, &alias).is_err() && std::fs::copy(&source, &alias).is_err() {
            // The environment may be read-only: fall back to a hard link under
            // `target/` and point `LIBCLANG_PATH` at it.
            let alias_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/libclang");
            if std::fs::create_dir_all(&alias_dir).is_ok() {
                let alias = alias_dir.join("libclang.dll");
                let _ = std::fs::remove_file(&alias);
                if std::fs::hard_link(&source, &alias).is_ok()
                    || std::fs::copy(&source, &alias).is_ok()
                {
                    std::env::set_var("LIBCLANG_PATH", &alias_dir);
                }
            }
        }

        return;
    }
}
