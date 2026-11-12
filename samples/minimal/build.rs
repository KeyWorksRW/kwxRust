//! Build script for the minimal sample port
//!
//! Embeds the Windows manifest for Common Controls 6.0 support.

use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    // Build scripts run for the host, so the target must come from the environment.
    if env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }

    let manifest_dir = match env::var("CARGO_MANIFEST_DIR") {
        Ok(dir) => PathBuf::from(dir),
        Err(_) => return,
    };
    let manifest_path = manifest_dir.join("minimal.manifest");
    if !manifest_path.exists() {
        return;
    }

    println!("cargo:rerun-if-changed={}", manifest_path.display());

    if env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("msvc") {
        // Tell the MSVC linker to embed the manifest.
        println!("cargo:rustc-link-arg=/MANIFEST:EMBED");
        println!(
            "cargo:rustc-link-arg=/MANIFESTINPUT:{}",
            manifest_path.display()
        );
        return;
    }

    // GNU ABI (gnullvm): link the manifest as a windres resource object.
    let out_dir = match env::var("OUT_DIR") {
        Ok(dir) => PathBuf::from(dir),
        Err(_) => return,
    };
    let rc_path = out_dir.join("minimal.rc");
    let obj_path = out_dir.join("minimal_manifest.o");
    let manifest = manifest_path.display().to_string().replace('\\', "/");

    if fs::write(&rc_path, format!("1 24 \"{manifest}\"\n")).is_err() {
        println!("cargo:warning=could not write {}", rc_path.display());
        return;
    }

    let windres = env::var("WINDRES").unwrap_or_else(|_| "llvm-windres".to_string());
    let status = Command::new(&windres)
        .arg("-i")
        .arg(&rc_path)
        .arg("-o")
        .arg(&obj_path)
        .status();

    match status {
        Ok(status) if status.success() => {
            println!("cargo:rustc-link-arg={}", obj_path.display());
        }
        _ => println!(
            "cargo:warning=could not compile the Windows manifest with {windres}; \
             building without it"
        ),
    }
}
