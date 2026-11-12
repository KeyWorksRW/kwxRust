//! wxPanel FFI bindings
//!
//! wxPanel is a container window for other controls.
//! It provides tab traversal and other container-specific functionality.

use super::types::{WxPanel, WxWindow};
use libc::c_int;
use std::ffi::c_void;

extern "C" {
    /// Create a new panel
    ///
    /// # Parameters
    /// - `parent`: Parent window (required)
    /// - `id`: Panel ID (use wxID_ANY for auto-generation)
    /// - `x`, `y`: Position (use -1 for default)
    /// - `width`, `height`: Size (use -1 for default)
    /// - `style`: Panel style flags
    ///
    /// # Safety
    /// - `parent` must be a valid wxWindow pointer
    pub fn wxPanel_Create(
        parent: *mut WxWindow,
        id: c_int,
        x: c_int,
        y: c_int,
        width: c_int,
        height: c_int,
        style: c_int,
    ) -> *mut c_void;

    /// Initialize dialog layout (for use with validators)
    ///
    /// # Safety
    /// - `self_` must be a valid wxPanel pointer
    pub fn wxPanel_InitDialog(self_: *mut c_void);

    /// Get the default item (button activated by Enter)
    ///
    /// # Safety
    /// - `self_` must be a valid wxPanel pointer
    pub fn wxPanel_GetDefaultItem(self_: *mut c_void) -> *mut c_void;

    /// Set the default item (button activated by Enter)
    ///
    /// # Safety
    /// - `self_` must be a valid wxPanel pointer
    /// - `button` must be a valid wxButton pointer or null
    pub fn wxPanel_SetDefaultItem(self_: *mut c_void, button: *mut c_void);

    /// Set focus to the panel
    ///
    /// # Safety
    /// - `self_` must be a valid wxPanel pointer
    pub fn wxPanel_SetFocus(self_: *mut c_void);
}

/// Helper to cast panel pointer to correct type
#[inline]
pub fn wxPanel_Create_typed(
    parent: *mut WxWindow,
    id: c_int,
    x: c_int,
    y: c_int,
    width: c_int,
    height: c_int,
    style: c_int,
) -> *mut WxPanel {
    unsafe { wxPanel_Create(parent, id, x, y, width, height, style) as *mut WxPanel }
}
