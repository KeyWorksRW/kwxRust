//! Message boxes and the wxWidgets version string

use libc::c_int;
use std::ffi::c_void;

extern "C" {
    /// Show a modal message box
    ///
    /// # Parameters
    /// - `message`: Message text (wxString)
    /// - `caption`: Dialog caption (wxString)
    /// - `style`: wxOK / wxICON_INFORMATION / ... flags
    /// - `parent`: Parent window, or null
    /// - `x`, `y`: Dialog position (use -1 for the default)
    ///
    /// # Safety
    /// - `message` and `caption` must be valid wxString pointers
    /// - `parent` must be null or a valid wxWindow pointer
    pub fn kwxMessageBox(
        message: *mut c_void,
        caption: *mut c_void,
        style: c_int,
        parent: *mut c_void,
        x: c_int,
        y: c_int,
    ) -> c_int;

    /// wxOK message box style, resolved at runtime
    pub fn expwxOK() -> c_int;

    /// wxICON_INFORMATION message box style, resolved at runtime
    pub fn expwxICON_INFORMATION() -> c_int;

    /// wxVERSION_NUM_DOT_STRING, returned as a new wxString
    pub fn expwxVERSION_NUM_DOT_STRING() -> *mut c_void;
}
