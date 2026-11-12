//! Opaque types representing wxWidgets classes
//!
//! These are zero-sized types used only as markers for type-safe pointers.
//! All wxWidgets objects are accessed through `*mut WxXxx` pointers.

use std::ffi::c_void;

/// Opaque type representing a wxWindow
#[repr(C)]
pub struct WxWindow {
    _private: [u8; 0],
}

/// Opaque type representing a wxFrame (derives from wxTopLevelWindow -> wxWindow)
#[repr(C)]
pub struct WxFrame {
    _private: [u8; 0],
}

/// Opaque type representing a wxButton (derives from wxControl -> wxWindow)
#[repr(C)]
pub struct WxButton {
    _private: [u8; 0],
}

/// Opaque type representing a wxPanel (derives from wxWindow)
#[repr(C)]
pub struct WxPanel {
    _private: [u8; 0],
}

/// Opaque type representing a wxString
#[repr(C)]
pub struct WxString {
    _private: [u8; 0],
}

/// Opaque type representing a wxClosure (callback wrapper)
#[repr(C)]
pub struct WxClosure {
    _private: [u8; 0],
}

/// Opaque type representing a wxEvent
#[repr(C)]
pub struct WxEvent {
    _private: [u8; 0],
}

/// Opaque type representing a wxCommandEvent (derives from wxEvent)
#[repr(C)]
pub struct WxCommandEvent {
    _private: [u8; 0],
}

/// Opaque type representing a wxEvtHandler
#[repr(C)]
pub struct WxEvtHandler {
    _private: [u8; 0],
}

/// Opaque type representing a wxSize
#[repr(C)]
pub struct WxSize {
    _private: [u8; 0],
}

/// Opaque type representing a wxPoint
#[repr(C)]
pub struct WxPoint {
    _private: [u8; 0],
}

/// Opaque type representing a wxStatusBar
#[repr(C)]
pub struct WxStatusBar {
    _private: [u8; 0],
}

/// Opaque type representing a wxMenu
#[repr(C)]
pub struct WxMenu {
    _private: [u8; 0],
}

/// Opaque type representing a wxMenuBar
#[repr(C)]
pub struct WxMenuBar {
    _private: [u8; 0],
}

/// Opaque type representing a wxSizer
#[repr(C)]
pub struct WxSizer {
    _private: [u8; 0],
}

/// Opaque type representing a wxIcon
#[repr(C)]
pub struct WxIcon {
    _private: [u8; 0],
}

/// Opaque type representing a wxColour
#[repr(C)]
pub struct WxColour {
    _private: [u8; 0],
}

/// Opaque type representing a wxCharBuffer (UTF-8 string buffer)
#[repr(C)]
pub struct WxCharBuffer {
    _private: [u8; 0],
}

/// Type alias for clarity: wxWindowID is typically c_int
pub type WxWindowID = libc::c_int;

/// Generic void pointer for untyped FFI
pub type WxVoidPtr = *mut c_void;

/// Closure function signature used by wxFFI for callbacks
/// Parameters: (function_ptr, user_data, event_ptr)
pub type ClosureFun = Option<extern "C" fn(*mut c_void, *mut c_void, *mut WxEvent)>;
