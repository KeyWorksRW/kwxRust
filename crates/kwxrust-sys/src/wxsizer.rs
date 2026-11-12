//! wxSizer / wxBoxSizer FFI bindings
//!
//! A sizer passed to `wxWindow_SetSizerAndFit` is owned by that window.

use super::types::{WxSizer, WxWindow};
use libc::c_int;
use std::ffi::c_void;

extern "C" {
    /// Create a box sizer
    ///
    /// # Parameters
    /// - `orient`: wxVERTICAL or wxHORIZONTAL
    pub fn wxBoxSizer_Create(orient: c_int) -> *mut WxSizer;

    /// Add a window to a sizer
    ///
    /// # Safety
    /// - `self_` must be a valid wxSizer pointer
    /// - `window` must be a valid wxWindow pointer
    pub fn wxSizer_AddWindow(
        self_: *mut WxSizer,
        window: *mut WxWindow,
        option: c_int,
        flag: c_int,
        border: c_int,
        user_data: *mut c_void,
    );

    /// Attach a sizer to a window (and resize the window to fit)
    ///
    /// # Safety
    /// - `self_` must be a valid wxWindow pointer
    /// - `sizer` must be a valid wxSizer pointer
    pub fn wxWindow_SetSizerAndFit(self_: *mut WxWindow, sizer: *mut WxSizer, delete_old: c_int);
}
