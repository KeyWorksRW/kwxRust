//! Message boxes
//!
//! Thin wrapper over `kwxMessageBox` with the style flags resolved at runtime.

use crate::frame::Frame;
use crate::sys::{expwxICON_INFORMATION, expwxOK, kwxMessageBox};
use crate::wxstr::WxStr;
use std::ffi::c_void;
use std::ptr;

/// Show a modal message box
///
/// Returns the value of the button the user pressed.
pub fn message_box(parent: Option<&Frame>, message: &str, caption: &str, style: i32) -> i32 {
    let text = WxStr::new(message);
    let title = WxStr::new(caption);
    let parent_ptr = match parent {
        Some(frame) => frame.as_window() as *mut c_void,
        None => ptr::null_mut(),
    };

    unsafe {
        kwxMessageBox(
            text.as_ptr() as *mut c_void,
            title.as_ptr() as *mut c_void,
            style,
            parent_ptr,
            -1,
            -1,
        )
    }
}

/// Show an informational message box (`wxOK | wxICON_INFORMATION`)
pub fn info(parent: Option<&Frame>, message: &str, caption: &str) -> i32 {
    let style = unsafe { expwxOK() | expwxICON_INFORMATION() };
    message_box(parent, message, caption, style)
}

/// Show an informational message box parented to the application's top window
///
/// This is the shape a menu handler needs: a `Frame` cannot be borrowed into a
/// `'static` event closure, but the frame is the app's top window at that point.
pub fn info_modal(message: &str, caption: &str) -> i32 {
    let text = WxStr::new(message);
    let title = WxStr::new(caption);
    let parent = crate::app::get_top_window();
    let style = unsafe { expwxOK() | expwxICON_INFORMATION() };

    unsafe {
        kwxMessageBox(
            text.as_ptr() as *mut c_void,
            title.as_ptr() as *mut c_void,
            style,
            parent,
            -1,
            -1,
        )
    }
}
