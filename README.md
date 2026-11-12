# kwxRust

Rust language bindings for [wxWidgets](https://www.wxwidgets.org/) via [kwxFFI](https://github.com/KeyWorksRW/kwxFFI), enabling cross-platform GUI applications that render with native controls on Windows, macOS, and Linux.

## ⚠️ Warning: Not Production Ready

**Do not use this Rust/wxWidgets interface in production.**

- The kwxFFI ABI interface is **not stable** and can change without warning.
- Very little testing has been done, and won't be until late Q2 of 2026.
- API surface may change as idioms are refined.

## Overview

kwxRust provides a two-crate binding architecture:

1. **FFI Layer** (`crates/kwxrust-sys`) — Raw `extern "C"` declarations mapping directly to the
   kwxFFI C function exports. All wxWidgets objects are opaque pointer types
   (`*mut WxFrame`, `*mut WxButton`, …) and every function is `unsafe`.
2. **Safe Wrapper Layer** (`crates/kwxrust`) — Idiomatic, safe Rust APIs with RAII ownership,
   builder patterns, automatic `&str` ↔ `wxString` conversion, and closure-based event binding.

Users should work with the safe wrapper crate. The FFI crate is re-exported as `kwxrust::sys` for
direct access when needed.

```
Rust app  →  kwxrust (safe)  →  kwxrust-sys (FFI)  →  kwxFFI (C++)  →  wxWidgets (C++)
```

A third directory, `wx/`, holds a much larger `kwxgen`-generated binding surface
(`kwxffi-sys`, 700+ modules). It is generated code, is **not** part of the workspace, and is not
yet wired into the build.

## Quick Example

```rust
// A minimal application using the safe wrapper crate.
use kwxrust::prelude::*;

fn main() {
    // Initialize the wxWidgets runtime and image handlers.
    app::init();

    let frame = Frame::builder("Hello wxRust")
        .size(400, 300)
        .center()
        .build();

    app::set_top_window(&frame);

    // The button is owned by the frame and destroyed with it.
    let button = Button::new(&frame, "Click Me!");

    // Bind a click handler. `forget()` keeps the connection alive for the
    // lifetime of the program.
    event::bind(&frame, button.id(), event::event_type::button(), |_evt| {
        println!("Button clicked!");
    })
    .forget();

    frame.show();

    let exit_code = app::run();
    app::shutdown();
    std::process::exit(exit_code);
}
```

A complete, faithful port of the wxWidgets C++ `minimal` sample — menu bar, status bar, and About
dialog — is in [`samples/minimal`](samples/minimal/src/main.rs). Run it with:

```
cargo run -p minimal
```

## Crate Layout

| Path | Purpose |
| --- | --- |
| `crates/kwxrust-sys` | Raw `extern "C"` bindings; `unsafe` throughout |
| `crates/kwxrust` | Safe wrappers and the `prelude` module |
| `samples/minimal` | Port of the wxWidgets `minimal` sample |
| `src/kwxApp.cpp` | C++ shim implementing the `kwxApp_*` application API |
| `wx/` | `kwxgen`-generated bindings (not built yet) |
| `CMakeLists.txt`, `CMakePresets.json` | Builds kwxFFI and wxWidgets static libraries |

## Safe Wrapper Surface

`crates/kwxrust` currently exposes these modules (re-exported by `kwxrust::prelude`):

| Module | Types / functions |
| --- | --- |
| `app` | `init`, `run`, `shutdown`, `set_top_window`, `set_app_name`, `os_description`, `version_string` |
| `frame` | `Frame`, `FrameBuilder` |
| `button` | `Button`, `ButtonBuilder`, `AsWindow` |
| `menu` | `Menu`, `MenuBar` |
| `dialog` | `message_box`, `info`, `info_modal` |
| `event` | `Event`, `CommandEvent`, `EventConnection`, `bind`, `on_button`, `event_type` |
| `wxstr` | `WxStr` (RAII wrapper for `wxString`) |

## Type Model

Unlike C++, Rust has no inheritance. wxWidgets classes are represented twice:

- **Opaque FFI types** (`WxFrame`, `WxButton`, `WxMenu`, …) — zero-sized `#[repr(C)]` markers used
  only to type raw pointers. Related classes are interchangeable by pointer cast, because a
  `wxFrame` *is* a `wxWindow` *is* a `wxEvtHandler` in the underlying C++ object.
- **Safe wrappers** (`Frame`, `Button`, `Menu`, `MenuBar`, `WxStr`) — own or borrow those pointers
  and encode the ownership rule in Rust's type system.

Ownership rules enforced by the wrappers:

- **Windows and controls** (frame children, buttons, menus, menu bars) are owned by wxWidgets and
  destroyed with their parent. Their wrappers have **no `Drop`** implementation.
- **`Frame`** closes itself on `Drop` unless `release()` transfers ownership to wxWidgets.
- **`wxString`** is wrapped by `WxStr`, which deletes the native string on `Drop`. Strings returned
  from FFI are converted to `String` and freed immediately.
- **`EventConnection`** keeps a closure alive; call `.forget()` to persist the handler for the
  program's lifetime. Dropping it currently does not disconnect.

## Naming Conventions

| Entity | Convention | Example |
| --- | --- | --- |
| Crates | `kebab-case` | `kwxrust`, `kwxrust-sys` |
| Modules | `snake_case` | `app`, `frame`, `event`, `wxstr` |
| Safe wrapper types | `PascalCase` | `Frame`, `Button`, `WxStr` |
| Opaque FFI types | `Wx` + `PascalCase` | `WxFrame`, `WxButton` |
| Raw FFI functions | Match C name | `wxFrame_Create`, `kwxApp_Initialize` |
| Constants | `SCREAMING_SNAKE_CASE` | `WX_ID_ANY`, `WX_EVT_BUTTON` |
| Helper functions | `snake_case` | `wx_id_any()`, `wx_default_frame_style()` |
| Runtime constants | `exp` + wxWidgets name | `exp_wxEVT_BUTTON()`, `expwxID_EXIT()` |

Values that are not compile-time stable in wxWidgets (event types, standard IDs) are resolved at
runtime through `exp*` functions. Never hard-code them.

## Why Rust + wxWidgets?

Rust has excellent GUI crates, but they either reimplement every control or ship a browser engine.
kwxRust binds the actual wxWidgets toolkit, so applications get genuine native controls, native
menus, and native file dialogs without bundling a web runtime.

## Compilers & Toolchain

| Toolchain | Status |
| --- | --- |
| `stable-x86_64-pc-windows-gnullvm` + clang64 / LLVM-MinGW (Windows) | ✅ Primary |
| MSVC-hosted Rust | ❌ Not supported (ABI mismatch with the clang64 static libraries) |
| `gcc` / `clang` targets (Linux / macOS) | 🔧 Planned |

The Rust toolchain is pinned in `rust-toolchain.toml`. Do **not** add a `[build] target` to
`.cargo/config.toml` — a cross-target build would move artifacts to `target/<triple>/debug`
instead of `target/debug`, which breaks the `build.rs` assumptions.

## Building

Prerequisites:

- CMake 3.30+
- Clang / `clang++` and `llvm-rc` on `PATH`, plus `llvm-windres` for the sample manifest
- Rust toolchain from `rust-toolchain.toml`
- A sibling `../kwxFFI` checkout — `CMakeLists.txt` currently uses the local checkout so source
  changes to kwxFFI show up immediately

The native libraries must be built **before** any `cargo` command; `crates/kwxrust-sys/build.rs`
links against the CMake output and panics with instructions if it is missing.

```bash
# 1. Build kwxFFI + wxWidgets static libraries
cmake --preset ninja-clang
cmake --build build --config Debug      # or: --config Release

# 2. Build the Rust crates (debug configuration matches the libraries above)
cargo build

# 3. Run the minimal sample
cargo run -p minimal
```

Build outputs (git-ignored):

- `lib/Debug`, `lib/Release` — `libkwxFFI.a` and the wxWidgets archives linked by `build.rs`
- `bin/Debug`, `bin/Release` — CMake runtime output
- `target/` — Rust artifacts

Because `build.rs` selects libraries by `PROFILE`, build the CMake configuration that matches the
cargo profile:

| cargo command | CMake `--config` |
| --- | --- |
| `cargo build` | `Debug` |
| `cargo build --release` | `Release` |

## License

Apache License 2.0 — see [LICENSE](LICENSE) for details.
