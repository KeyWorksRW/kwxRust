//! wxStatusBar FFI bindings

use super::types::{WxStatusBar, WxString};
use libc::c_int;

extern "C" {
    /// Set the text of one status bar field
    ///
    /// # Parameters
    /// - `text`: New text
    /// - `number`: Field index (0 is the first field)
    ///
    /// # Safety
    /// - `self_` must be a valid wxStatusBar pointer
    /// - `text` must be a valid wxString pointer
    pub fn wxStatusBar_SetStatusText(self_: *mut WxStatusBar, text: *mut WxString, number: c_int);
}
