//! Safe wrapper for wxButton
//!
//! `Button` is a simple push button control that fires events when clicked.

use crate::sys::{
    wxButton_Create, wxButton_SetDefault, wxString_CreateUTF8, wxString_Delete, wxWindow_Disable,
    wxWindow_Enable, wxWindow_GetId, wxWindow_Hide, wxWindow_SetSize, wxWindow_Show, wx_id_any,
    WxButton, WxEvtHandler, WxWindow,
};
use std::ffi::CString;

/// A push button control
///
/// Button is a safe wrapper around wxButton. Buttons are owned by their
/// parent window and automatically deleted when the parent is destroyed.
///
/// # Example
///
/// ```rust,no_run
/// use kwxrust::{frame::Frame, button::Button};
///
/// let frame = Frame::new("Button Example");
/// let button = Button::new(&frame, "Click Me!");
/// button.show();
/// ```
pub struct Button {
    ptr: *mut WxButton,
}

impl Button {
    /// Create a new button with default settings
    ///
    /// The button is created as a child of the given parent window.
    pub fn new<P: AsWindow>(parent: &P, label: &str) -> Self {
        Self::builder(parent, label).build()
    }

    /// Create a builder for configuring the button
    pub fn builder<P: AsWindow>(parent: &P, label: &str) -> ButtonBuilder {
        ButtonBuilder::new(parent.as_window(), label)
    }

    /// Create a Button from a raw pointer
    ///
    /// # Safety
    ///
    /// - `ptr` must be a valid wxButton pointer
    pub unsafe fn from_raw(ptr: *mut WxButton) -> Self {
        assert!(!ptr.is_null(), "Cannot create Button from null pointer");
        Button { ptr }
    }

    /// Get the raw pointer
    pub fn as_ptr(&self) -> *mut WxButton {
        self.ptr
    }

    /// Get as a window pointer
    pub fn as_window(&self) -> *mut WxWindow {
        self.ptr as *mut WxWindow
    }

    /// Get as an event handler pointer (for event binding)
    ///
    /// In wxWidgets, wxWindow inherits from wxEvtHandler, so this is a simple cast.
    pub fn as_evt_handler(&self) -> *mut WxEvtHandler {
        self.ptr as *mut WxEvtHandler
    }

    /// Get the button's window ID
    ///
    /// Queries the ID from wxWidgets on each call.
    pub fn id(&self) -> i32 {
        unsafe { wxWindow_GetId(self.ptr as *mut WxWindow) }
    }

    /// Show the button
    pub fn show(&self) {
        unsafe {
            wxWindow_Show(self.ptr as *mut WxWindow);
        }
    }

    /// Hide the button
    pub fn hide(&self) {
        unsafe {
            wxWindow_Hide(self.ptr as *mut WxWindow);
        }
    }

    /// Enable the button
    pub fn enable(&self) {
        unsafe {
            wxWindow_Enable(self.ptr as *mut WxWindow);
        }
    }

    /// Disable the button
    pub fn disable(&self) {
        unsafe {
            wxWindow_Disable(self.ptr as *mut WxWindow);
        }
    }

    /// Set the button size
    pub fn set_size(&self, width: i32, height: i32) {
        unsafe { wxWindow_SetSize(self.ptr as *mut WxWindow, -1, -1, width, height, 0) }
    }

    /// Make this the default button
    ///
    /// The default button is activated when Enter is pressed.
    pub fn set_default(&self) {
        unsafe { wxButton_SetDefault(self.ptr) }
    }
}

// Note: Buttons are owned by their parent, we don't implement Drop

/// Trait for types that can be used as window parents
pub trait AsWindow {
    fn as_window(&self) -> *mut WxWindow;
}

impl AsWindow for crate::frame::Frame {
    fn as_window(&self) -> *mut WxWindow {
        self.as_ptr() as *mut WxWindow
    }
}

/// Builder for Button with fluent configuration
pub struct ButtonBuilder {
    parent: *mut WxWindow,
    label: String,
    id: i32,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    style: i32,
}

impl ButtonBuilder {
    /// Create a new builder
    pub fn new(parent: *mut WxWindow, label: &str) -> Self {
        ButtonBuilder {
            parent,
            label: label.to_string(),
            id: wx_id_any(),
            x: -1,
            y: -1,
            width: -1,
            height: -1,
            style: 0,
        }
    }

    /// Set the button ID
    pub fn id(mut self, id: i32) -> Self {
        self.id = id;
        self
    }

    /// Set the position
    pub fn position(mut self, x: i32, y: i32) -> Self {
        self.x = x;
        self.y = y;
        self
    }

    /// Set the size
    pub fn size(mut self, width: i32, height: i32) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    /// Set the style flags
    pub fn style(mut self, style: i32) -> Self {
        self.style = style;
        self
    }

    /// Build the button
    pub fn build(self) -> Button {
        let cstr = CString::new(self.label).expect("Label contains null bytes");
        let ptr = unsafe {
            let wx_label = wxString_CreateUTF8(cstr.as_ptr());
            let button = wxButton_Create(
                self.parent,
                self.id,
                wx_label,
                self.x,
                self.y,
                self.width,
                self.height,
                self.style,
            );
            wxString_Delete(wx_label);
            button
        };

        assert!(!ptr.is_null(), "Failed to create button");

        Button { ptr }
    }
}
