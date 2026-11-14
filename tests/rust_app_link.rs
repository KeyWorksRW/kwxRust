// Native link script shared by the kwxRust test packages (tests/quick and
// tests/quick_probe) through `include!` in each package's build.rs.
//
// The `wx` crate is a generated, link-only binding crate: it declares the
// `kwxFFI` symbols but owns no build script, so a package that depends on it
// must supply the native link line itself. This is the same line the
// `samples/minimal` build script emits:
//
// 1. Compile the repository's `src/kwxApp.cpp` GUI entry shim and archive it,
//    because that file is not part of the `kwxFFI` target but defines the
//    `kwxApp_*` symbols every GUI runner links against.
// 2. Emit the native link arguments: the shim archive, `libkwxFFI.a`, the
//    wxWidgets static modules CMake produced, the C++ runtime and the Windows
//    system libraries. They are link arguments, not `rustc-link-lib`, so the
//    archives are scanned after the Rust rlibs.
// 3. Embed the package's Common Controls 6.0 manifest (`<package>.manifest`).

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

// Locates the repository root (the folder holding `kwxgen.config.json`).
fn repo_root(start: &Path) -> Option<PathBuf> {
    let mut dir = Some(start);
    while let Some(current) = dir {
        if current.join("kwxgen.config.json").is_file() {
            return Some(current.to_path_buf());
        }
        dir = current.parent();
    }
    None
}

// Compiles `src/kwxApp.cpp` and archives it as `libkwxapp_entry.a`.
fn compile_entry_shim(repo: &Path, debug: bool) -> Option<PathBuf> {
    let source = repo.join("src").join("kwxApp.cpp");
    if !source.is_file() {
        println!(
            "cargo:warning={} not found; the kwxApp_* entry points stay unresolved",
            source.display()
        );
        return None;
    }
    println!("cargo:rerun-if-changed={}", source.display());

    let out_dir = PathBuf::from(env::var("OUT_DIR").ok()?);
    let object = out_dir.join("kwxApp.o");
    let archive = out_dir.join("libkwxapp_entry.a");

    let compiler = env::var("CXX").unwrap_or_else(|_| "x86_64-w64-mingw32-clang++".to_string());
    let wx_include = repo
        .join("build")
        .join("_deps")
        .join("wxwidgets-src")
        .join("include");
    let setup_dir = if debug { "mswud" } else { "mswu" };
    let wx_setup = repo
        .join("build")
        .join("_deps")
        .join("wxwidgets-build")
        .join("lib")
        .join("clang_x64_lib")
        .join(setup_dir);
    let kwxffi_include = repo
        .parent()
        .map(|p| p.join("kwxFFI").join("include"))
        .unwrap_or_else(|| PathBuf::from("../kwxFFI/include"));

    let status = Command::new(&compiler)
        .arg("-std=c++23")
        .arg(if debug { "-O0" } else { "-O2" })
        .arg("-c")
        .arg(&source)
        .arg("-o")
        .arg(&object)
        .arg(format!("-I{}", kwxffi_include.display()))
        .arg(format!("-I{}", wx_include.display()))
        .arg(format!("-I{}", wx_setup.display()))
        .status();
    match status {
        Ok(s) if s.success() => {}
        _ => {
            println!(
                "cargo:warning=could not compile {} with {compiler}",
                source.display()
            );
            return None;
        }
    }

    let archiver = env::var("AR").unwrap_or_else(|_| "llvm-ar".to_string());
    let status = Command::new(&archiver)
        .arg("crs")
        .arg(&archive)
        .arg(&object)
        .status();
    match status {
        Ok(s) if s.success() => Some(archive),
        _ => {
            println!(
                "cargo:warning=could not archive {} with {archiver}",
                object.display()
            );
            None
        }
    }
}

// Embeds the package manifest (`<package>.manifest`) as a resource object.
fn embed_manifest(manifest_dir: &Path, package: &str) {
    if env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    let manifest_path = manifest_dir.join(format!("{package}.manifest"));
    if !manifest_path.exists() {
        return;
    }
    println!("cargo:rerun-if-changed={}", manifest_path.display());

    if env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("msvc") {
        println!("cargo:rustc-link-arg=/MANIFEST:EMBED");
        println!(
            "cargo:rustc-link-arg=/MANIFESTINPUT:{}",
            manifest_path.display()
        );
        return;
    }

    // GNU ABI (gnullvm): link the manifest as a windres resource object.
    let Ok(out_dir) = env::var("OUT_DIR") else {
        return;
    };
    let out_dir = PathBuf::from(out_dir);
    let rc_path = out_dir.join(format!("{package}.rc"));
    let obj_path = out_dir.join(format!("{package}_manifest.o"));
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

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    let package = env::var("CARGO_PKG_NAME").unwrap_or_else(|_| String::from("app"));
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=../rust_app_link.rs");

    let Some(repo) = repo_root(&manifest_dir) else {
        println!("cargo:warning=kwxgen.config.json not found; skipping native link directives");
        return;
    };

    // Debug cargo profile -> lib/Debug and the `d`-suffixed wx archives.
    let debug = env::var("PROFILE").as_deref() != Ok("release");
    let kwx_config = if debug { "Debug" } else { "Release" };

    let kwx_lib_dir = repo.join("lib").join(kwx_config);
    let wx_lib_dir = repo
        .join("build")
        .join("_deps")
        .join("wxwidgets-build")
        .join("lib")
        .join("clang_x64_lib");
    println!("cargo:rustc-link-search=native={}", kwx_lib_dir.display());
    println!("cargo:rustc-link-search=native={}", wx_lib_dir.display());

    // ---- link line --------------------------------------------------------
    let entry_archive = compile_entry_shim(&repo, debug);
    if let Some(archive) = &entry_archive {
        println!(
            "cargo:rustc-link-search=native={}",
            archive.parent().unwrap().display()
        );
        println!("cargo:rustc-link-arg=-lkwxapp_entry");
    }

    // The archives are passed by path: the search path also holds stale
    // `lib`-prefixed copies that would otherwise win over the current output.
    println!(
        "cargo:rustc-link-arg={}",
        kwx_lib_dir.join("libkwxFFI.a").display()
    );

    // wxWidgets modules the generated bindings reference, in dependency order.
    // CMake inserts a `d` before the module name in the debug archives.
    let wx_libs = [
        ("wxmsw33ud_adv", "wxmsw33u_adv"),
        ("wxmsw33ud_aui", "wxmsw33u_aui"),
        ("wxmsw33ud_html", "wxmsw33u_html"),
        ("wxmsw33ud_media", "wxmsw33u_media"),
        ("wxmsw33ud_propgrid", "wxmsw33u_propgrid"),
        ("wxmsw33ud_ribbon", "wxmsw33u_ribbon"),
        ("wxmsw33ud_richtext", "wxmsw33u_richtext"),
        ("wxmsw33ud_stc", "wxmsw33u_stc"),
        ("wxmsw33ud_webview", "wxmsw33u_webview"),
        ("wxmsw33ud_xrc", "wxmsw33u_xrc"),
        ("wxmsw33ud_gl", "wxmsw33u_gl"),
        ("wxmsw33ud_core", "wxmsw33u_core"),
        ("wxbase33ud", "wxbase33u"),
        ("wxbase33ud_net", "wxbase33u_net"),
        ("wxbase33ud_xml", "wxbase33u_xml"),
        ("wxpngd", "wxpng"),
        ("wxjpegd", "wxjpeg"),
        ("wxtiffd", "wxtiff"),
        ("wxzlibd", "wxzlib"),
        ("wxregexud", "wxregexu"),
        ("wxexpatd", "wxexpat"),
        ("wxwebpd", "wxwebp"),
        ("wxwebpdemuxd", "wxwebpdemux"),
        ("wxsharpyuvd", "wxsharpyuv"),
        ("wxlunasvgd", "wxlunasvg"),
        ("wxscintillad", "wxscintilla"),
        ("wxlexillad", "wxlexilla"),
    ];
    for (debug_name, release_name) in wx_libs {
        let name = if debug { debug_name } else { release_name };
        println!(
            "cargo:rustc-link-arg={}",
            wx_lib_dir.join(format!("{name}.a")).display()
        );
    }

    // The native libraries are C++ (LLVM-MinGW / libc++). This toolchain's
    // libc++.a and libc++abi.a both define the standard exception classes, so
    // the duplicate definitions are tolerated instead of failing the link.
    println!("cargo:rustc-link-arg=-Wl,--allow-multiple-definition");
    println!("cargo:rustc-link-arg=-lc++");
    println!("cargo:rustc-link-arg=-lc++abi");

    // Windows system libraries required by wxWidgets.
    for lib in [
        "comctl32", "uxtheme", "rpcrt4", "ole32", "oleaut32", "oleacc", "uuid", "winmm", "ws2_32",
        "shell32", "shlwapi", "comdlg32", "gdi32", "gdiplus", "msimg32", "winspool", "version",
        "advapi32", "user32", "kernel32", "winhttp", "opengl32", "urlmon", "wininet", "dwmapi",
    ] {
        println!("cargo:rustc-link-arg=-l{lib}");
    }

    embed_manifest(&manifest_dir, &package);
}
