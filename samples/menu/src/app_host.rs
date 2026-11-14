//! The sample's single hand-written `extern "C"` block.
//!
//! These are the GUI entry points implemented by the repository's only
//! hand-written C++ file, `src/kwxApp.cpp`. `kwxgen` does not generate
//! declarations for them, so a sample declares exactly the ones it calls.

use std::os::raw::{c_char, c_void};

use kwxffi_sys::traits::WxObject;

#[link(name = "kwxFFI")]
extern "C" {
    fn kwxApp_Initialize(argc: i32, argv: *mut *mut c_char) -> i32;
    fn kwxApp_MainLoop() -> i32;
    fn kwxApp_Shutdown();
    fn kwxApp_SetTopWindow(window: *mut c_void);
}

/// `MyApp::OnInit()`: starts wxWidgets and calls the hidden app's `OnInit`.
/// Returns non-zero on success.
pub fn initialize() -> i32 {
    // No command line is forwarded; the shim substitutes a dummy argv.
    unsafe { kwxApp_Initialize(0, std::ptr::null_mut()) }
}

/// `wxTheApp->OnRun()`; returns the process exit code, or `-1` if the app
/// was never initialized.
pub fn main_loop() -> i32 {
    unsafe { kwxApp_MainLoop() }
}

/// Calls `OnExit()` and then `wxEntryCleanup()`.
pub fn shutdown() {
    unsafe { kwxApp_Shutdown() }
}

/// `wxTheApp->SetTopWindow(window)`.
pub fn set_top_window(window: &impl WxObject) {
    unsafe { kwxApp_SetTopWindow(window.as_ptr()) }
}
