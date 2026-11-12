//! wxClosure FFI bindings
//!
//! wxClosure wraps a callback function and user data for event handling.
//! This is the bridge between wxWidgets events and Rust callback functions.

use super::types::{ClosureFun, WxClosure, WxEvent};
use std::ffi::c_void;

extern "C" {
    /// Create a new closure
    ///
    /// The closure wraps a callback function and user data. When an event occurs,
    /// the callback is invoked with: `fun(fun_ptr, data, event_ptr)`
    ///
    /// When the closure is deleted, it calls the function one last time with
    /// a null event pointer, giving an opportunity to clean up resources.
    ///
    /// # Parameters
    /// - `fun`: Callback function pointer
    /// - `data`: User data passed to the callback
    ///
    /// # Safety
    /// - `fun` must remain valid for the lifetime of the closure
    /// - `data` lifetime must exceed the closure lifetime
    pub fn wxClosure_Create(fun: ClosureFun, data: *mut c_void) -> *mut WxClosure;

    /// Get the user data from a closure
    ///
    /// # Safety
    /// - `closure` must be a valid wxClosure pointer
    pub fn wxClosure_GetData(closure: *mut WxClosure) -> *mut c_void;
}

/// Rust-compatible closure callback type
///
/// This is the function signature that Rust code should implement for callbacks.
/// The function receives:
/// - `fun`: The function pointer (for identification, usually ignored)
/// - `data`: User data that was passed when creating the closure
/// - `event`: The event that triggered the callback (null when closure is being deleted)
pub type RustClosureFn = extern "C" fn(*mut c_void, *mut c_void, *mut WxEvent);
