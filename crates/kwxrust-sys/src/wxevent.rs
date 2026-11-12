//! wxEvent FFI bindings
//!
//! wxEvent is the base class for all events. wxCommandEvent is for
//! control events like button clicks.

use super::types::{WxCommandEvent, WxEvent, WxString};
use libc::{c_int, c_long};
use std::ffi::c_void;

extern "C" {
    // ========================================================================
    // Event Type Functions (runtime values from wxWidgets)
    // ========================================================================

    /// Get the wxEVT_BUTTON event type (wxEVT_COMMAND_BUTTON_CLICKED)
    ///
    /// This is the event fired when a wxButton is clicked.
    pub fn exp_wxEVT_BUTTON() -> c_int;

    /// Get the wxEVT_MENU event type (wxEVT_COMMAND_MENU_SELECTED)
    ///
    /// This is the event fired when a menu item is selected.
    pub fn exp_wxEVT_MENU() -> c_int;

    /// Get the wxEVT_CLOSE_WINDOW event type
    pub fn exp_wxEVT_CLOSE_WINDOW() -> c_int;

    /// Get the wxEVT_SIZE event type
    pub fn exp_wxEVT_SIZE() -> c_int;

    /// Get the wxEVT_IDLE event type
    pub fn exp_wxEVT_IDLE() -> c_int;

    /// Get the wxEVT_PAINT event type
    pub fn exp_wxEVT_PAINT() -> c_int;
    // ========================================================================
    // wxEvent
    // ========================================================================

    /// Get the event timestamp
    ///
    /// # Safety
    /// - `self_` must be a valid wxEvent pointer
    pub fn wxEvent_GetTimestamp(self_: *mut WxEvent) -> c_int;

    /// Set the event timestamp
    ///
    /// # Safety
    /// - `self_` must be a valid wxEvent pointer
    pub fn wxEvent_SetTimestamp(self_: *mut WxEvent, timestamp: c_int);

    /// Skip the event (allow further processing)
    ///
    /// Call this to allow the event to propagate to the next handler.
    ///
    /// # Safety
    /// - `self_` must be a valid wxEvent pointer
    pub fn wxEvent_Skip(self_: *mut WxEvent);

    /// Get the event type
    ///
    /// # Safety
    /// - `self_` must be a valid wxEvent pointer
    pub fn wxEvent_GetEventType(self_: *mut WxEvent) -> c_int;

    /// Set the event type
    ///
    /// # Safety
    /// - `self_` must be a valid wxEvent pointer
    pub fn wxEvent_SetEventType(self_: *mut WxEvent, type_: c_int);

    /// Get the object that generated the event
    ///
    /// # Safety
    /// - `self_` must be a valid wxEvent pointer
    pub fn wxEvent_GetEventObject(self_: *mut WxEvent) -> *mut c_void;

    /// Set the object that generated the event
    ///
    /// # Safety
    /// - `self_` must be a valid wxEvent pointer
    /// - `obj` must be a valid wxObject pointer or null
    pub fn wxEvent_SetEventObject(self_: *mut WxEvent, obj: *mut c_void);

    /// Get the window ID associated with the event
    ///
    /// # Safety
    /// - `self_` must be a valid wxEvent pointer
    pub fn wxEvent_GetId(self_: *mut WxEvent) -> c_int;

    /// Set the window ID associated with the event
    ///
    /// # Safety
    /// - `self_` must be a valid wxEvent pointer
    pub fn wxEvent_SetId(self_: *mut WxEvent, id: c_int);

    /// Check if the event was skipped
    ///
    /// # Safety
    /// - `self_` must be a valid wxEvent pointer
    pub fn wxEvent_GetSkipped(self_: *mut WxEvent) -> c_int;

    /// Check if this is a command event
    ///
    /// # Safety
    /// - `self_` must be a valid wxEvent pointer
    pub fn wxEvent_IsCommandEvent(self_: *mut WxEvent) -> c_int;

    // ========================================================================
    // wxCommandEvent
    // ========================================================================

    /// Create a new command event
    ///
    /// # Safety
    /// - Caller must delete with `wxCommandEvent_Delete`
    pub fn wxCommandEvent_Create(type_: c_int, id: c_int) -> *mut WxCommandEvent;

    /// Delete a command event
    ///
    /// # Safety
    /// - `self_` must be a valid wxCommandEvent pointer
    pub fn wxCommandEvent_Delete(self_: *mut WxCommandEvent);

    /// Get the selection index
    ///
    /// # Safety
    /// - `self_` must be a valid wxCommandEvent pointer
    pub fn wxCommandEvent_GetSelection(self_: *mut WxCommandEvent) -> c_int;

    /// Get the string associated with the event (e.g., for text controls)
    ///
    /// # Safety
    /// - `self_` must be a valid wxCommandEvent pointer
    /// - Caller must delete the returned wxString
    pub fn wxCommandEvent_GetString(self_: *mut WxCommandEvent) -> *mut WxString;

    /// Set the string associated with the event
    ///
    /// # Safety
    /// - `self_` must be a valid wxCommandEvent pointer
    /// - `s` must be a valid wxString pointer
    pub fn wxCommandEvent_SetString(self_: *mut WxCommandEvent, s: *mut WxString);

    /// Check if a checkbox/radio button is checked
    ///
    /// # Safety
    /// - `self_` must be a valid wxCommandEvent pointer
    pub fn wxCommandEvent_IsChecked(self_: *mut WxCommandEvent) -> c_int;

    /// Check if this is a selection event
    ///
    /// # Safety
    /// - `self_` must be a valid wxCommandEvent pointer
    pub fn wxCommandEvent_IsSelection(self_: *mut WxCommandEvent) -> c_int;

    /// Get the extra long value
    ///
    /// # Safety
    /// - `self_` must be a valid wxCommandEvent pointer
    pub fn wxCommandEvent_GetExtraLong(self_: *mut WxCommandEvent) -> c_long;

    /// Set the extra long value
    ///
    /// # Safety
    /// - `self_` must be a valid wxCommandEvent pointer
    pub fn wxCommandEvent_SetExtraLong(self_: *mut WxCommandEvent, extra: c_long);

    /// Get the integer value
    ///
    /// # Safety
    /// - `self_` must be a valid wxCommandEvent pointer
    pub fn wxCommandEvent_GetInt(self_: *mut WxCommandEvent) -> c_long;

    /// Set the integer value
    ///
    /// # Safety
    /// - `self_` must be a valid wxCommandEvent pointer
    pub fn wxCommandEvent_SetInt(self_: *mut WxCommandEvent, i: c_int);

    /// Get client data pointer
    ///
    /// # Safety
    /// - `self_` must be a valid wxCommandEvent pointer
    pub fn wxCommandEvent_GetClientData(self_: *mut WxCommandEvent) -> *mut c_void;

    /// Set client data pointer
    ///
    /// # Safety
    /// - `self_` must be a valid wxCommandEvent pointer
    pub fn wxCommandEvent_SetClientData(self_: *mut WxCommandEvent, data: *mut c_void);
}
