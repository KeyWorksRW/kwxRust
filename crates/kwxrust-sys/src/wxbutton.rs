//! wxButton FFI bindings
//!
//! wxButton is a simple push button control.

use super::types::{WxButton, WxColour, WxString, WxWindow};
use libc::c_int;

extern "C" {
    /// Create a new button
    ///
    /// # Parameters
    /// - `parent`: Parent window (required)
    /// - `id`: Button ID (use wxID_ANY for auto-generation)
    /// - `text`: Button label text
    /// - `x`, `y`: Position (use -1 for default)
    /// - `width`, `height`: Size (use -1 for default)
    /// - `style`: Button style flags
    ///
    /// # Safety
    /// - `parent` must be a valid wxWindow pointer
    /// - `text` must be a valid wxString pointer
    pub fn wxButton_Create(
        parent: *mut WxWindow,
        id: c_int,
        text: *mut WxString,
        x: c_int,
        y: c_int,
        width: c_int,
        height: c_int,
        style: c_int,
    ) -> *mut WxButton;

    /// Set the button as the default button
    ///
    /// The default button is activated when Enter is pressed.
    ///
    /// # Safety
    /// - `self_` must be a valid wxButton pointer
    pub fn wxButton_SetDefault(self_: *mut WxButton);

    /// Set the button background colour
    ///
    /// # Safety
    /// - `self_` must be a valid wxButton pointer
    /// - `colour` must be a valid wxColour pointer
    pub fn wxButton_SetBackgroundColour(self_: *mut WxButton, colour: *mut WxColour) -> c_int;
}
