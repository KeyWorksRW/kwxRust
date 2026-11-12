//! wxApp integration for Rust applications
//!
//! This module provides the core application lifecycle management for wxWidgets
//! applications written in Rust.
//!
//! # Architecture
//!
//! wxWidgets requires a wxApp subclass to be registered and initialized before
//! any GUI operations. For Rust bindings, we use the kwxApp C API from
//! kwxFFI, which provides:
//! - `kwxApp_Initialize()` - Initialize the wxApp
//! - `kwxApp_MainLoop()` - Runs the main event loop
//! - `kwxApp_ExitMainLoop()` - Exits the main event loop
//! - `kwxApp_Shutdown()` - Cleanup on exit
//! - `kwxApp_ConnectClosure()` - Connects event handlers
//!
//! Note: The Rust application must be linked with kwxApp.cpp and kwxFFI.

use crate::sys::WxClosure;
use std::ffi::{c_char, c_void};
use std::os::raw::c_int;

// External functions from kwxApp C API
extern "C" {
    /// Initialize the wxApp
    /// Returns non-zero on success
    pub fn kwxApp_Initialize(argc: c_int, argv: *mut *mut c_char) -> c_int;

    /// Run the wxWidgets main event loop
    /// Returns the exit code
    pub fn kwxApp_MainLoop() -> c_int;

    /// Exit the main event loop
    pub fn kwxApp_ExitMainLoop();

    /// Clean up wxWidgets (call on exit)
    pub fn kwxApp_Shutdown();

    /// Check if application is terminating
    pub fn kwxApp_IsTerminating() -> c_int;

    /// Check if there are pending events
    pub fn kwxApp_Pending() -> c_int;

    /// Dispatch a single pending event
    pub fn kwxApp_Dispatch();

    /// Set the application name
    pub fn kwxApp_SetAppName(name: *const c_char);

    /// Get the application name
    pub fn kwxApp_GetAppName() -> *const c_char;

    /// Set the vendor name
    pub fn kwxApp_SetVendorName(name: *const c_char);

    /// Get the vendor name
    pub fn kwxApp_GetVendorName() -> *const c_char;

    /// Set the top-level window
    pub fn kwxApp_SetTopWindow(wnd: *mut c_void);

    /// Get the top-level window
    pub fn kwxApp_GetTopWindow() -> *mut c_void;

    /// Set whether the app exits when the top frame is deleted
    pub fn kwxApp_SetExitOnFrameDelete(flag: c_int);

    /// Get whether the app exits when the top frame is deleted
    pub fn kwxApp_GetExitOnFrameDelete() -> c_int;

    /// Initialize all image handlers (PNG, JPEG, etc.)
    pub fn kwxApp_InitAllImageHandlers();

    /// Play system bell sound
    pub fn kwxApp_Bell();

    /// Connect an event handler using a wxClosure
    ///
    /// # Parameters
    /// - `obj`: Event handler (window) to connect to
    /// - `first`: First window ID to handle (use -1 for any)
    /// - `last`: Last window ID to handle (use -1 for any)
    /// - `event_type`: Event type (e.g., wxEVT_BUTTON)
    /// - `closure`: wxClosure to invoke when event fires
    pub fn kwxApp_ConnectClosure(
        obj: *mut c_void,
        first: c_int,
        last: c_int,
        event_type: c_int,
        closure: *mut WxClosure,
    ) -> c_int;

    /// Disconnect an event handler
    pub fn kwxApp_Disconnect(
        obj: *mut c_void,
        first: c_int,
        last: c_int,
        event_type: c_int,
    ) -> c_int;

    /// Get the operating system description (`wxGetOsDescription()`)
    ///
    /// The returned buffer is allocated by wxWidgets and must be released with
    /// `kwxApp_FreeString`.
    pub fn kwxApp_GetOsDescription() -> *mut c_char;

    /// Release a string returned by a `kwxApp_Get*` function
    pub fn kwxApp_FreeString(value: *mut c_char);
}

/// Initialize the wxWidgets application
///
/// Call this before creating any windows. This function:
/// 1. Calls kwxApp_Initialize to initialize the wxWidgets runtime
/// 2. Initializes image handlers
///
/// # Safety
///
/// Must be called from the main thread before any other wxWidgets operations.
pub fn init() {
    unsafe {
        let result = kwxApp_Initialize(0, std::ptr::null_mut());
        if result == 0 {
            eprintln!("Warning: kwxApp_Initialize failed");
        }

        kwxApp_InitAllImageHandlers();
    }
}

/// Run the wxWidgets main event loop
///
/// This function blocks until the application exits (all windows closed or
/// `exit_main_loop()` called).
///
/// Returns the application exit code.
pub fn run() -> i32 {
    unsafe { kwxApp_MainLoop() as i32 }
}

/// Exit the main event loop
///
/// Call this to terminate the application programmatically.
pub fn exit_main_loop() {
    unsafe { kwxApp_ExitMainLoop() }
}

/// Shut down the wxWidgets application
///
/// Call this after the main loop exits to clean up resources.
pub fn shutdown() {
    unsafe { kwxApp_Shutdown() }
}

/// Check if the application is terminating
pub fn is_terminating() -> bool {
    unsafe { kwxApp_IsTerminating() != 0 }
}

/// Check if there are pending events
pub fn has_pending_events() -> bool {
    unsafe { kwxApp_Pending() != 0 }
}

/// Process a single pending event
pub fn dispatch() {
    unsafe { kwxApp_Dispatch() }
}

/// Set the top-level window
///
/// The application will exit when this window is closed.
pub fn set_top_window(window: &crate::frame::Frame) {
    unsafe { kwxApp_SetTopWindow(window.as_window() as *mut c_void) }
}

/// Get the top-level window
pub fn get_top_window() -> *mut c_void {
    unsafe { kwxApp_GetTopWindow() }
}

/// Set whether the application exits when the top frame is deleted
///
/// Default is true.
pub fn set_exit_on_frame_delete(flag: bool) {
    unsafe { kwxApp_SetExitOnFrameDelete(if flag { 1 } else { 0 }) }
}

/// Get whether the application exits when the top frame is deleted
pub fn get_exit_on_frame_delete() -> bool {
    unsafe { kwxApp_GetExitOnFrameDelete() != 0 }
}

/// Set the application name
///
/// This is used for configuration file paths and other system integrations.
pub fn set_app_name(name: &str) {
    let c_name = std::ffi::CString::new(name).unwrap();
    unsafe { kwxApp_SetAppName(c_name.as_ptr()) }
}

/// Set the vendor name
///
/// This is used for configuration file paths and other system integrations.
pub fn set_vendor_name(name: &str) {
    let c_name = std::ffi::CString::new(name).unwrap();
    unsafe { kwxApp_SetVendorName(c_name.as_ptr()) }
}

/// Play the system bell sound
pub fn bell() {
    unsafe { kwxApp_Bell() }
}

/// Close the top-level window (the frame set with `set_top_window`)
///
/// Mirrors calling `Close(true)` on the frame: the close event is sent but the
/// window cannot veto it, and the application exits once the frame is gone.
pub fn close_top_window(force: bool) {
    unsafe {
        let window = kwxApp_GetTopWindow();
        if !window.is_null() {
            crate::sys::wxWindow_Close(window as *mut crate::sys::WxWindow, force as c_int);
        }
    }
}

/// Get the operating system description (`wxGetOsDescription()`)
///
/// Example: `"Microsoft Windows 11 Home (build 22631), 64-bit edition"`.
pub fn os_description() -> String {
    unsafe {
        let ptr = kwxApp_GetOsDescription();
        if ptr.is_null() {
            return String::new();
        }

        let text = std::ffi::CStr::from_ptr(ptr).to_string_lossy().into_owned();
        kwxApp_FreeString(ptr);
        text
    }
}

/// Get the wxWidgets version string (`wxVERSION_STRING`), e.g. `"wxWidgets 3.3.5"`
pub fn version_string() -> String {
    use crate::sys::{expwxVERSION_NUM_DOT_STRING, wxString_Delete, wxString_GetString, WxString};

    unsafe {
        let ptr = expwxVERSION_NUM_DOT_STRING() as *mut WxString;
        if ptr.is_null() {
            return String::new();
        }

        let len = wxString_GetString(ptr, std::ptr::null_mut()) as usize;
        let mut buffer = vec![0u8; len + 1];
        wxString_GetString(ptr, buffer.as_mut_ptr() as *mut c_char);
        wxString_Delete(ptr);

        if len == 0 {
            return String::new();
        }

        format!("wxWidgets {}", String::from_utf8_lossy(&buffer[..len]))
    }
}
