//! wxWidgets constants
//!
//! These constants are defined based on wxWidgets header values.
//! They are stable across wxWidgets 3.x versions.

use libc::c_int;

extern "C" {
    /// wxID_EXIT, resolved at runtime
    pub fn expwxID_EXIT() -> c_int;

    /// wxID_ABOUT, resolved at runtime
    pub fn expwxID_ABOUT() -> c_int;
}

// ============================================================================
// Window IDs
// ============================================================================

/// Special ID meaning "generate an ID automatically"
pub const WX_ID_ANY: c_int = -1;

/// Highest valid ID value
pub const WX_ID_HIGHEST: c_int = 5999;

/// Lowest valid ID value
pub const WX_ID_LOWEST: c_int = 0;

/// Exit command ID (wxID_EXIT)
pub const WX_ID_EXIT: c_int = 5006;

/// About command ID
pub const WX_ID_ABOUT: c_int = 5014;

/// OK button ID
pub const WX_ID_OK: c_int = 5100;

/// Cancel button ID
pub const WX_ID_CANCEL: c_int = 5101;

/// Yes button ID
pub const WX_ID_YES: c_int = 5103;

/// No button ID
pub const WX_ID_NO: c_int = 5104;

// ============================================================================
// Frame Styles (wxTLW = Top Level Window)
// ============================================================================

/// Frame can be minimized
pub const WX_MINIMIZE_BOX: c_int = 0x0400;

/// Frame can be maximized
pub const WX_MAXIMIZE_BOX: c_int = 0x0200;

/// Frame has close button
pub const WX_CLOSE_BOX: c_int = 0x1000;

/// Frame has caption bar
pub const WX_CAPTION: c_int = 0x20000000;

/// Frame has system menu
pub const WX_SYSTEM_MENU: c_int = 0x0800;

/// Frame can be resized
pub const WX_RESIZE_BORDER: c_int = 0x0040;

/// Frame stays on top of other windows
pub const WX_STAY_ON_TOP: c_int = 0x8000;

/// Default frame style (caption, resize border, system menu, minimize/maximize/close buttons)
pub const WX_DEFAULT_FRAME_STYLE: c_int = WX_SYSTEM_MENU
    | WX_RESIZE_BORDER
    | WX_MINIMIZE_BOX
    | WX_MAXIMIZE_BOX
    | WX_CLOSE_BOX
    | WX_CAPTION;

// ============================================================================
// Button Styles
// ============================================================================

/// Button contents are left-justified
pub const WX_BU_LEFT: c_int = 0x0040;

/// Button contents are right-justified
pub const WX_BU_RIGHT: c_int = 0x0100;

/// Button contents are top-justified
pub const WX_BU_TOP: c_int = 0x0080;

/// Button contents are bottom-justified
pub const WX_BU_BOTTOM: c_int = 0x0200;

/// Exact fit to label size
pub const WX_BU_EXACTFIT: c_int = 0x0001;

/// No border
pub const WX_BU_NOTEXT: c_int = 0x0002;

// ============================================================================
// Alignment and Direction
// ============================================================================

/// Center alignment
pub const WX_CENTER: c_int = 0x0001;

/// Centre (British spelling, same as CENTER)
pub const WX_CENTRE: c_int = WX_CENTER;

/// Horizontal orientation
pub const WX_HORIZONTAL: c_int = 0x0004;

/// Vertical orientation
pub const WX_VERTICAL: c_int = 0x0008;

/// Both horizontal and vertical
pub const WX_BOTH: c_int = WX_HORIZONTAL | WX_VERTICAL;

/// Left alignment
pub const WX_LEFT: c_int = 0x0010;

/// Right alignment
pub const WX_RIGHT: c_int = 0x0020;

/// Top alignment
pub const WX_TOP: c_int = 0x0040;

/// Bottom alignment
pub const WX_BOTTOM: c_int = 0x0080;

/// All sides (for borders/padding)
pub const WX_ALL: c_int = WX_TOP | WX_BOTTOM | WX_LEFT | WX_RIGHT;

/// Align center horizontally
pub const WX_ALIGN_CENTER_HORIZONTAL: c_int = 0x0100;

/// Align center vertically
pub const WX_ALIGN_CENTER_VERTICAL: c_int = 0x0200;

// ============================================================================
// Event Types
// ============================================================================
// IMPORTANT: wxWidgets event types are runtime-assigned and should NOT be
// used as compile-time constants. Use the exp_wxEVT_*() functions from wxevent.rs
// to get the correct runtime values.
//
// These constants are DEPRECATED and kept only for compatibility.
// They may have incorrect values.

/// Button clicked event (wxEVT_BUTTON)
/// DEPRECATED: Use exp_wxEVT_BUTTON() from wxevent.rs instead
#[deprecated(note = "Use exp_wxEVT_BUTTON() for correct runtime value")]
pub const WX_EVT_BUTTON: c_int = 10049;

/// Menu selected event (wxEVT_MENU)
/// DEPRECATED: Use exp_wxEVT_MENU() from wxevent.rs instead
#[deprecated(note = "Use exp_wxEVT_MENU() for correct runtime value")]
pub const WX_EVT_MENU: c_int = 10009;

/// Window close event (wxEVT_CLOSE_WINDOW)
/// DEPRECATED: Use exp_wxEVT_CLOSE_WINDOW() from wxevent.rs instead
#[deprecated(note = "Use exp_wxEVT_CLOSE_WINDOW() for correct runtime value")]
pub const WX_EVT_CLOSE_WINDOW: c_int = 10401;

/// Window destroy event (wxEVT_DESTROY)
#[deprecated(note = "Event types should be retrieved at runtime")]
pub const WX_EVT_DESTROY: c_int = 10403;

/// Idle event (wxEVT_IDLE)
/// DEPRECATED: Use exp_wxEVT_IDLE() from wxevent.rs instead
#[deprecated(note = "Use exp_wxEVT_IDLE() for correct runtime value")]
pub const WX_EVT_IDLE: c_int = 10416;

/// Size event (wxEVT_SIZE)
/// DEPRECATED: Use exp_wxEVT_SIZE() from wxevent.rs instead
#[deprecated(note = "Use exp_wxEVT_SIZE() for correct runtime value")]
pub const WX_EVT_SIZE: c_int = 10073;

/// Paint event (wxEVT_PAINT)
/// DEPRECATED: Use exp_wxEVT_PAINT() from wxevent.rs instead
#[deprecated(note = "Use exp_wxEVT_PAINT() for correct runtime value")]
pub const WX_EVT_PAINT: c_int = 10001;

// ============================================================================
// Size Flags
// ============================================================================

/// Use existing size
pub const WX_SIZE_USE_EXISTING: c_int = 0x0000;

/// Allow negative values for position
pub const WX_SIZE_ALLOW_MINUS_ONE: c_int = 0x0004;

// ============================================================================
// Convenience Functions (for FFI compatibility with existing code)
// ============================================================================

/// Returns wxID_ANY (-1)
#[inline]
pub const fn wx_id_any() -> c_int {
    WX_ID_ANY
}

/// Returns wxID_EXIT from wxWidgets (5006 in wx 3.x)
#[inline]
pub fn wx_id_exit() -> c_int {
    unsafe { expwxID_EXIT() }
}

/// Returns wxID_ABOUT from wxWidgets (5014 in wx 3.x)
#[inline]
pub fn wx_id_about() -> c_int {
    unsafe { expwxID_ABOUT() }
}

/// Returns wxDEFAULT_FRAME_STYLE
#[inline]
pub const fn wx_default_frame_style() -> c_int {
    WX_DEFAULT_FRAME_STYLE
}

/// Returns wxEVT_BUTTON event type
/// DEPRECATED: Use exp_wxEVT_BUTTON() from wxevent.rs instead
#[inline]
#[deprecated(note = "Use exp_wxEVT_BUTTON() for correct runtime value")]
#[allow(deprecated)]
pub const fn wx_evt_button() -> c_int {
    WX_EVT_BUTTON
}

/// Returns wxCENTER
#[inline]
pub const fn wx_center() -> c_int {
    WX_CENTER
}

/// Returns wxBOTH
#[inline]
pub const fn wx_both() -> c_int {
    WX_BOTH
}

/// Returns wxALL
#[inline]
pub const fn wx_all() -> c_int {
    WX_ALL
}
