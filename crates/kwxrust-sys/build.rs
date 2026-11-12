// Build script for kwxrust-sys
//
// This build script:
// 1. Locates the CMake-built kwxFFI static library
// 2. Compiles kwxApp.cpp (Rust-specific app interface)
// 3. Links the kwxFFI library, the static wxWidgets archives and the C++ runtime
//
// Prerequisites:
// - CMake 3.30+ configured and built: cmake --preset ninja-clang && cmake --build build
// - The native libraries are built with the clang64 (LLVM-MinGW) toolchain, so the
//   Rust toolchain must be the gnullvm-hosted one (pinned in rust-toolchain.toml),
//   which also keeps cargo's output in target/debug and target/release.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

/// wxWidgets modules linked into every sample, in dependency order.
const WX_LIBS: &[&str] = &[
    "wxmsw33u_core",
    "wxmsw33u_adv",
    "wxmsw33u_aui",
    "wxmsw33u_html",
    "wxmsw33u_webview",
    "wxmsw33u_propgrid",
    "wxmsw33u_ribbon",
    "wxmsw33u_richtext",
    "wxmsw33u_stc",
    "wxmsw33u_xrc",
    "wxbase33u",
    "wxbase33u_net",
    "wxbase33u_xml",
];

/// Third-party libraries bundled with wxWidgets (dependency order).
const WX_SUPPORT_LIBS: &[&str] = &[
    "wxscintilla",
    "wxlexilla",
    "wxexpat",
    "wxjpeg",
    "wxpng",
    "wxtiff",
    "wxzlib",
    "wxregexu",
    "wxlunasvg",
    "wxwebp",
    "wxwebpdemux",
    "wxsharpyuv",
];

/// Windows system libraries required by wxWidgets.
const SYSTEM_LIBS: &[&str] = &[
    "kernel32",
    "user32",
    "gdi32",
    "gdiplus",
    "comdlg32",
    "winspool",
    "shell32",
    "ole32",
    "oleaut32",
    "uuid",
    "advapi32",
    "comctl32",
    "rpcrt4",
    "wsock32",
    "wininet",
    "winmm",
    "shlwapi",
    "version",
    "uxtheme",
    "dwmapi",
    "imm32",
    "msimg32",
    "oleacc",
    "windowscodecs",
    "msftedit",
    "opengl32",
];

fn main() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let workspace_root = PathBuf::from(&manifest_dir)
        .parent() // crates/
        .and_then(|p| p.parent()) // workspace root
        .expect("Failed to find workspace root")
        .to_path_buf();

    // Detect debug vs release build
    let profile = env::var("PROFILE").unwrap_or_else(|_| "debug".to_string());
    let is_debug = profile == "debug";
    let config = if is_debug { "Debug" } else { "Release" };

    println!("cargo:warning=Building kwxrust-sys ({} mode)", config);

    // CMake output paths
    let build_dir = workspace_root.join("build");
    let lib_dir = workspace_root.join("lib").join(config);

    if !build_dir.exists() || !lib_dir.exists() {
        println!("cargo:warning=CMake build directory not found!");
        println!("cargo:warning=Please run:");
        println!("cargo:warning=  cmake --preset ninja-clang");
        println!("cargo:warning=  cmake --build build --config {}", config);
        panic!("CMake build not found - cannot link");
    }

    // Find the kwxFFI static library. The clang64 build emits a GNU archive named
    // libkwxFFI.a; an MSVC build would emit kwxFFI.lib and cannot be linked from a
    // gnullvm target.
    let kwxffi_lib = lib_dir.join("libkwxFFI.a");
    if !kwxffi_lib.exists() {
        println!(
            "cargo:warning=kwxFFI library not found at: {}",
            kwxffi_lib.display()
        );
        println!("cargo:warning=Run: cmake --build build --config {}", config);
        panic!("kwxFFI library not found");
    }

    println!("cargo:rustc-link-search=native={}", lib_dir.display());
    println!("cargo:rustc-link-lib=static=kwxFFI");
    println!("cargo:rerun-if-changed={}", kwxffi_lib.display());

    // Compile kwxApp.cpp
    let kwxapp_cpp = workspace_root.join("src").join("kwxApp.cpp");
    if !kwxapp_cpp.exists() {
        println!(
            "cargo:warning=kwxApp.cpp not found at: {}",
            kwxapp_cpp.display()
        );
        panic!("kwxApp.cpp not found");
    }

    // Find wxWidgets include paths
    let wx_include = build_dir
        .join("_deps")
        .join("wxwidgets-src")
        .join("include");

    if !wx_include.exists() {
        println!(
            "cargo:warning=wxWidgets include not found at: {}",
            wx_include.display()
        );
        panic!("wxWidgets includes not found - run CMake build first");
    }

    let kwxffi_include = match find_kwxffi_include(&workspace_root, &build_dir) {
        Some(dir) => dir,
        None => {
            println!("cargo:warning=kwxFFI includes not found - run CMake build first");
            panic!("kwxFFI includes not found");
        }
    };

    // The wxWidgets archives are spelled without the "lib" prefix
    // (wxmsw33u_core.a), so expose them to the linker through a shim directory in
    // OUT_DIR named lib<name>.a - that is the spelling rustc's `-l` emits.
    let wx_lib_dir = find_wx_lib_dir(&build_dir);
    let shim_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR not set")).join("wxlibs");
    fs::create_dir_all(&shim_dir).expect("failed to create wxlibs shim directory");

    let all_libs: Vec<String> = WX_LIBS
        .iter()
        .chain(WX_SUPPORT_LIBS.iter())
        .map(|name| wx_lib_name(name, is_debug))
        .collect();

    println!("cargo:rustc-link-search=native={}", shim_dir.display());
    for name in &all_libs {
        let archive = wx_lib_dir.join(format!("{}.a", name));
        if !archive.exists() {
            println!(
                "cargo:warning=wxWidgets library not found: {}",
                archive.display()
            );
            println!(
                "cargo:warning=Rebuild the kwxFFI CMake project in {} mode",
                config
            );
            panic!("wxWidgets libraries for {} build not found", config);
        }

        let shim = shim_dir.join(format!("lib{}.a", name));
        let _ = fs::remove_file(&shim);
        if fs::hard_link(&archive, &shim).is_err() {
            fs::copy(&archive, &shim).unwrap_or_else(|err| {
                panic!(
                    "failed to expose {} to the linker: {err}",
                    archive.display()
                )
            });
        }

        println!("cargo:rustc-link-lib=static={}", name);
        println!("cargo:rerun-if-changed={}", archive.display());
    }

    // setup.h lives beside the archives, in mswu (release) or mswud (debug).
    let setup_include = wx_lib_dir.join(if is_debug { "mswud" } else { "mswu" });
    if !setup_include.join("wx").join("setup.h").exists() {
        println!(
            "cargo:warning=wx/setup.h not found at: {}",
            setup_include.display()
        );
        panic!("wxWidgets setup.h not found");
    }

    println!("cargo:rerun-if-changed={}", kwxapp_cpp.display());

    let mut build = cc::Build::new();
    build
        .cpp(true)
        .file(&kwxapp_cpp)
        .include(&kwxffi_include)
        .include(&wx_include)
        .include(&setup_include);

    // Mirror the flags kwxFFI itself was compiled with: the generated headers and
    // kwxApp.cpp must see the same wxWidgets configuration as the static libraries.
    build
        .std("c++23")
        .define("__WXMSW__", None)
        .define("__WX__", None)
        .define("wxUSE_GUI", "1")
        .define("UNICODE", None)
        .define("_UNICODE", None);

    // Locate the toolchain's C++ runtime (libc++.a, libc++abi.a, libunwind.a) from
    // the compiler cc-rs selected, so no path is hard-coded.
    let compiler = build
        .try_get_compiler()
        .unwrap_or_else(|err| panic!("no C++ compiler available: {err}"));
    let toolchain_lib = resolve_tool(compiler.path())
        .and_then(|path| {
            path.parent()
                .and_then(Path::parent)
                .map(|root| root.join("lib"))
        })
        .unwrap_or_else(|| {
            panic!(
                "failed to derive the toolchain lib directory from {:?}",
                compiler.path()
            )
        });

    build.compile("kwxapp");

    if toolchain_lib.join("libc++.a").exists() {
        println!("cargo:rustc-link-search=native={}", toolchain_lib.display());
        println!("cargo:rustc-link-lib=static=c++");
        println!("cargo:rustc-link-lib=static=c++abi");
        println!("cargo:rustc-link-lib=static=unwind");
    } else {
        println!(
            "cargo:warning=C++ runtime not found in {} - expected libc++.a",
            toolchain_lib.display()
        );
        panic!("C++ runtime libraries not found");
    }

    for lib in SYSTEM_LIBS {
        println!("cargo:rustc-link-lib={}", lib);
    }

    // Rerun if the environment changes
    println!("cargo:rerun-if-env-changed=PROFILE");
}

/// Locate the kwxFFI include directory: the sibling checkout used by
/// FetchContent, or the fetched copy under build/_deps.
fn find_kwxffi_include(workspace_root: &Path, build_dir: &Path) -> Option<PathBuf> {
    let sibling = workspace_root
        .parent()
        .map(|parent| parent.join("kwxFFI").join("include"));
    if let Some(dir) = sibling {
        if dir.join("kwx_wrapper.h").exists() {
            return Some(dir);
        }
    }

    let fetched = build_dir.join("_deps").join("kwxffi-src").join("include");
    if fetched.join("kwx_wrapper.h").exists() {
        return Some(fetched);
    }

    None
}

/// Resolve a compiler name to an absolute path by searching PATH when cc-rs
/// returned a bare command name instead of a full path.
fn resolve_tool(command: &Path) -> Option<PathBuf> {
    if command.is_absolute() && command.exists() {
        return Some(command.to_path_buf());
    }

    let path_var = env::var_os("PATH")?;
    for dir in env::split_paths(&path_var) {
        let candidate = dir.join(command);
        if candidate.is_file() {
            return Some(candidate);
        }

        // Windows resolves `clang++` through the PATHEXT list.
        let with_exe = dir.join(format!("{}.exe", command.display()));
        if with_exe.is_file() {
            return Some(with_exe);
        }
    }

    None
}

/// Map a release archive name to its debug spelling. The debug marker goes right
/// after the versioned `33u` stem (wxmsw33ud_core, wxbase33ud_net) and is appended
/// for the support libraries (wxscintillad, wxexpatd, wxregexud).
fn wx_lib_name(base: &str, is_debug: bool) -> String {
    if !is_debug {
        return base.to_string();
    }

    if let Some(index) = base.find("33u") {
        let split = index + "33u".len();
        return format!("{}d{}", &base[..split], &base[split..]);
    }

    format!("{base}d")
}

/// Locate the directory holding the wxWidgets static archives.
fn find_wx_lib_dir(build_dir: &Path) -> PathBuf {
    let lib_root = build_dir.join("_deps").join("wxwidgets-build").join("lib");
    let candidates = [lib_root.join("clang_x64_lib"), lib_root.join("vc_x64_lib")];

    for candidate in &candidates {
        if candidate.join("wxbase33u.a").exists() {
            return candidate.clone();
        }
    }

    println!(
        "cargo:warning=wxWidgets library directory not found under: {}",
        lib_root.display()
    );
    panic!("wxWidgets library directory not found");
}
