//! wxEvtHandler FFI bindings
//!
//! wxEvtHandler is the base class for event handling. All windows inherit from it.
//! Events are connected using closures that wrap callback functions.

use super::types::{WxClosure, WxEvent, WxEvtHandler};
use libc::c_int;
use std::ffi::c_void;

extern "C" {
    // ========================================================================
    // Event Handler Creation
    // ========================================================================

    /// Create a standalone event handler
    ///
    /// # Safety
    /// - Caller must delete with `wxEvtHandler_Delete`
    pub fn wxEvtHandler_Create() -> *mut WxEvtHandler;

    /// Delete an event handler
    ///
    /// # Safety
    /// - `self_` must be a valid wxEvtHandler pointer
    /// - Do not use the pointer after deletion
    pub fn wxEvtHandler_Delete(self_: *mut WxEvtHandler);

    // ========================================================================
    // Event Connection
    // ========================================================================

    /// Connect an event handler
    ///
    /// This uses the base wxEvtHandler_Connect from wxFFI. However, note that
    /// different language ports may need to use their own Connect implementation
    /// (like wxEvtHandler_ConnectSampleApp in the example) to properly route
    /// events through their app class's HandleEvent method.
    ///
    /// # Parameters
    /// - `self_`: Event handler to connect to (usually a window)
    /// - `first`: First window ID to handle (use -1 for any)
    /// - `last`: Last window ID to handle (use -1 for any)
    /// - `type_`: Event type (e.g., wxEVT_COMMAND_BUTTON_CLICKED)
    /// - `data`: User data passed to the callback (usually a wxClosure pointer)
    ///
    /// # Safety
    /// - `self_` must be a valid wxEvtHandler pointer
    /// - `data` lifetime must exceed the connection lifetime
    pub fn wxEvtHandler_Connect(
        self_: *mut WxEvtHandler,
        first: c_int,
        last: c_int,
        type_: c_int,
        data: *mut c_void,
    ) -> c_int;

    /// Disconnect an event handler
    ///
    /// # Parameters
    /// - `self_`: Event handler to disconnect from
    /// - `first`: First window ID
    /// - `last`: Last window ID
    /// - `type_`: Event type
    /// - `id`: Handler ID (0 to disconnect all matching)
    ///
    /// # Safety
    /// - `self_` must be a valid wxEvtHandler pointer
    pub fn wxEvtHandler_Disconnect(
        self_: *mut WxEvtHandler,
        first: c_int,
        last: c_int,
        type_: c_int,
        id: c_int,
    ) -> c_int;

    // ========================================================================
    // Event Processing
    // ========================================================================

    /// Process an event
    ///
    /// # Safety
    /// - `self_` must be a valid wxEvtHandler pointer
    /// - `event` must be a valid wxEvent pointer
    pub fn wxEvtHandler_ProcessEvent(self_: *mut WxEvtHandler, event: *mut WxEvent) -> c_int;

    /// Add a pending event to the queue
    ///
    /// # Safety
    /// - `self_` must be a valid wxEvtHandler pointer
    /// - `event` must be a valid wxEvent pointer
    pub fn wxEvtHandler_AddPendingEvent(self_: *mut WxEvtHandler, event: *mut WxEvent);

    /// Process all pending events
    ///
    /// # Safety
    /// - `self_` must be a valid wxEvtHandler pointer
    pub fn wxEvtHandler_ProcessPendingEvents(self_: *mut WxEvtHandler);

    // ========================================================================
    // Handler Chain
    // ========================================================================

    /// Get the next handler in the chain
    ///
    /// # Safety
    /// - `self_` must be a valid wxEvtHandler pointer
    pub fn wxEvtHandler_GetNextHandler(self_: *mut WxEvtHandler) -> *mut WxEvtHandler;

    /// Get the previous handler in the chain
    ///
    /// # Safety
    /// - `self_` must be a valid wxEvtHandler pointer
    pub fn wxEvtHandler_GetPreviousHandler(self_: *mut WxEvtHandler) -> *mut WxEvtHandler;

    /// Set the next handler in the chain
    ///
    /// # Safety
    /// - `self_` must be a valid wxEvtHandler pointer
    /// - `handler` must be a valid wxEvtHandler pointer or null
    pub fn wxEvtHandler_SetNextHandler(self_: *mut WxEvtHandler, handler: *mut WxEvtHandler);

    /// Set the previous handler in the chain
    ///
    /// # Safety
    /// - `self_` must be a valid wxEvtHandler pointer
    /// - `handler` must be a valid wxEvtHandler pointer or null
    pub fn wxEvtHandler_SetPreviousHandler(self_: *mut WxEvtHandler, handler: *mut WxEvtHandler);

    // ========================================================================
    // Handler State
    // ========================================================================

    /// Enable or disable the event handler
    ///
    /// # Safety
    /// - `self_` must be a valid wxEvtHandler pointer
    pub fn wxEvtHandler_SetEvtHandlerEnabled(self_: *mut WxEvtHandler, enabled: c_int);

    /// Check if the event handler is enabled
    ///
    /// # Safety
    /// - `self_` must be a valid wxEvtHandler pointer
    pub fn wxEvtHandler_GetEvtHandlerEnabled(self_: *mut WxEvtHandler) -> c_int;

    // ========================================================================
    // Client Data (Closure)
    // ========================================================================

    /// Get the client closure associated with this handler
    ///
    /// # Safety
    /// - `self_` must be a valid wxEvtHandler pointer
    pub fn wxEvtHandler_GetClientClosure(self_: *mut WxEvtHandler) -> *mut WxClosure;

    /// Set the client closure for this handler
    ///
    /// # Safety
    /// - `self_` must be a valid wxEvtHandler pointer
    /// - `closure` must be a valid wxClosure pointer
    pub fn wxEvtHandler_SetClientClosure(self_: *mut WxEvtHandler, closure: *mut WxClosure);
}
