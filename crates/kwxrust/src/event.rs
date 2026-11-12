//! Event handling for wxWidgets
//!
//! This module provides safe event binding for wxWidgets controls.
//! Events are connected using closures that are invoked when the event fires.

use crate::app::kwxApp_ConnectClosure;
use crate::sys::{
    wxClosure_Create, wxCommandEvent_GetInt, wxCommandEvent_GetString, wxCommandEvent_IsChecked,
    wxEvent_GetEventType, wxEvent_GetId, wxEvent_Skip, wxString_Delete, wxString_GetString,
    WxClosure, WxCommandEvent, WxEvent, WxEvtHandler,
};
use std::ffi::c_void;

/// Event types
pub mod event_type {
    use crate::sys;

    /// Button click event (wxEVT_COMMAND_BUTTON_CLICKED)
    ///
    /// This returns the runtime event type value from wxWidgets.
    pub fn button() -> i32 {
        // Use the FFI function to get the correct runtime value
        unsafe { sys::exp_wxEVT_BUTTON() }
    }

    // Add more event types as needed
}

/// Wrapper around wxEvent
///
/// Provides safe access to event data. The event pointer is only valid
/// during the callback invocation.
pub struct Event {
    ptr: *mut WxEvent,
}

impl Event {
    /// Create from raw pointer (called internally by event system)
    ///
    /// # Safety
    ///
    /// - `ptr` must be a valid wxEvent pointer
    /// - The Event must not outlive the callback invocation
    pub(crate) unsafe fn from_raw(ptr: *mut WxEvent) -> Self {
        Event { ptr }
    }

    /// Get the event type
    pub fn event_type(&self) -> i32 {
        unsafe { wxEvent_GetEventType(self.ptr) }
    }

    /// Get the window ID that generated the event
    pub fn id(&self) -> i32 {
        unsafe { wxEvent_GetId(self.ptr) }
    }

    /// Skip this event so it propagates to parent handlers
    pub fn skip(&self) {
        unsafe { wxEvent_Skip(self.ptr) }
    }

    /// Check if this is a command event and get additional info
    pub fn as_command(&self) -> Option<CommandEvent> {
        // All button events are command events
        // In a full implementation, we'd check the event type
        // Cast is safe because wxCommandEvent inherits from wxEvent
        Some(CommandEvent {
            ptr: self.ptr as *mut WxCommandEvent,
        })
    }
}

/// Command event (for buttons, menu items, etc.)
pub struct CommandEvent {
    ptr: *mut WxCommandEvent,
}

impl CommandEvent {
    /// Get the integer value associated with the event
    pub fn int(&self) -> i32 {
        unsafe { wxCommandEvent_GetInt(self.ptr) as i32 }
    }

    /// Check if a checkbox/toggle is checked
    pub fn is_checked(&self) -> bool {
        unsafe { wxCommandEvent_IsChecked(self.ptr) != 0 }
    }

    /// Get the string value associated with the event
    pub fn string(&self) -> String {
        unsafe {
            let wx_str = wxCommandEvent_GetString(self.ptr);
            if wx_str.is_null() {
                return String::new();
            }

            let len = wxString_GetString(wx_str, std::ptr::null_mut()) as usize;
            if len == 0 {
                wxString_Delete(wx_str);
                return String::new();
            }

            let mut buf: Vec<u8> = vec![0; len + 1];
            wxString_GetString(wx_str, buf.as_mut_ptr() as *mut libc::c_char);
            wxString_Delete(wx_str);

            String::from_utf8_lossy(&buf[..len]).to_string()
        }
    }
}

/// Trait for objects that can handle events
pub trait EventHandler {
    fn as_evt_handler(&self) -> *mut WxEvtHandler;
}

impl EventHandler for crate::frame::Frame {
    fn as_evt_handler(&self) -> *mut WxEvtHandler {
        crate::frame::Frame::as_evt_handler(self)
    }
}

impl EventHandler for crate::button::Button {
    fn as_evt_handler(&self) -> *mut WxEvtHandler {
        crate::button::Button::as_evt_handler(self)
    }
}

/// Event connection handle
///
/// Keeps the event handler alive. When dropped, the event handler is
/// disconnected (if still connected).
pub struct EventConnection {
    _closure: *mut WxClosure,
    // In a full implementation, we'd store info for disconnection
}

impl EventConnection {
    /// Keep this connection alive (don't disconnect on drop)
    ///
    /// Call this when you want the event handler to persist for the
    /// lifetime of the window.
    pub fn forget(self) {
        std::mem::forget(self);
    }
}

// Note: We don't implement Drop for EventConnection yet because
// disconnection requires keeping track of the handler and event type.
// For now, use .forget() to persist connections.

/// Bind an event handler to a control
///
/// The callback is invoked when the specified event type fires on the
/// control with the given ID.
///
/// # Arguments
///
/// * `handler` - The window/control to receive events
/// * `id` - Window ID to filter (-1 for any)
/// * `event_type` - Event type to handle
/// * `callback` - Closure to invoke when event fires
///
/// # Example
///
/// ```rust,no_run
/// use kwxrust::{frame::Frame, button::Button, event};
///
/// let frame = Frame::new("Event Example");
/// let button = Button::new(&frame, "Click Me!");
///
/// // Bind button click event
/// let conn = event::bind(&frame, button.id(), event::event_type::button(), |evt| {
///     println!("Button clicked!");
/// });
/// conn.forget(); // Keep the handler alive
/// ```
pub fn bind<H, F>(handler: &H, id: i32, event_type: i32, callback: F) -> EventConnection
where
    H: EventHandler,
    F: Fn(Event) + 'static,
{
    // Box the closure
    let boxed: Box<Box<dyn Fn(Event)>> = Box::new(Box::new(callback));
    let data = Box::into_raw(boxed) as *mut c_void;

    // Create the FFI callback
    extern "C" fn closure_callback(_fun: *mut c_void, data: *mut c_void, event: *mut WxEvent) {
        if event.is_null() {
            // Cleanup call - closure is being deleted
            unsafe {
                let _: Box<Box<dyn Fn(Event)>> = Box::from_raw(data as *mut _);
            }
            return;
        }

        // Normal event call
        unsafe {
            let callback: &Box<dyn Fn(Event)> = &*(data as *const Box<dyn Fn(Event)>);
            let evt = Event::from_raw(event);
            callback(evt);
        }
    }

    // Create wxClosure - ClosureFun is already Option<fn>, so we use Some()
    let closure = unsafe { wxClosure_Create(Some(closure_callback), data) };

    // Connect the event handler
    unsafe {
        kwxApp_ConnectClosure(
            handler.as_evt_handler() as *mut c_void,
            id,
            id, // last = first for single ID
            event_type,
            closure,
        );
    }

    EventConnection { _closure: closure }
}

/// Bind a button click handler
///
/// Convenience function for the common case of handling button clicks.
pub fn on_button<H, F>(handler: &H, button: &crate::button::Button, callback: F) -> EventConnection
where
    H: EventHandler,
    F: Fn(Event) + 'static,
{
    bind(handler, button.id(), event_type::button(), callback)
}
