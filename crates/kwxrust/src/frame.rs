//! Safe wrapper for wxFrame
//!
//! `Frame` is a top-level window with optional menu bar, tool bar, and status bar.
//! It's the main window type for most wxWidgets applications.

use crate::menu::MenuBar;
use crate::sys::{
    wxFrame_Create, wxFrame_CreateStatusBar, wxFrame_GetStatusBar, wxFrame_Iconize,
    wxFrame_IsIconized, wxFrame_IsMaximized, wxFrame_Maximize, wxFrame_Restore, wxFrame_SetMenuBar,
    wxStatusBar_SetStatusText, wxString_CreateUTF8, wxString_Delete, wxString_GetString,
    wxTopLevelWindow_GetTitle, wxTopLevelWindow_SetTitle, wxWindow_Center, wxWindow_Close,
    wxWindow_Hide, wxWindow_Move, wxWindow_SetSize, wxWindow_Show, wx_default_frame_style,
    wx_id_any, WxEvtHandler, WxFrame, WxWindow,
};
use libc::c_int;
use std::ffi::CString;
use std::ptr;

/// A top-level frame window
///
/// Frame is a safe wrapper around wxFrame. The frame is automatically closed
/// when dropped (unless transferred to wxWidgets ownership).
///
/// # Example
///
/// ```rust,no_run
/// use kwxrust::frame::Frame;
///
/// let frame = Frame::builder("My Application")
///     .size(800, 600)
///     .center()
///     .build();
/// frame.show();
/// ```
pub struct Frame {
    ptr: *mut WxFrame,
    /// If true, we don't own the window (wxWidgets manages it)
    owned_by_parent: bool,
}

impl Frame {
    /// Create a new frame with default settings
    ///
    /// Uses default position, size, and style.
    pub fn new(title: &str) -> Self {
        Self::builder(title).build()
    }

    /// Create a builder for configuring the frame
    pub fn builder(title: &str) -> FrameBuilder {
        FrameBuilder::new(title)
    }

    /// Create a Frame from a raw pointer
    ///
    /// # Safety
    ///
    /// - `ptr` must be a valid wxFrame pointer
    /// - Caller decides ownership with `owned_by_parent`
    pub unsafe fn from_raw(ptr: *mut WxFrame, owned_by_parent: bool) -> Self {
        assert!(!ptr.is_null(), "Cannot create Frame from null pointer");
        Frame {
            ptr,
            owned_by_parent,
        }
    }

    /// Get the raw pointer
    ///
    /// The pointer remains valid until the Frame is dropped.
    pub fn as_ptr(&self) -> *mut WxFrame {
        self.ptr
    }

    /// Get as a window pointer (for parent relationships)
    pub fn as_window(&self) -> *mut WxWindow {
        self.ptr as *mut WxWindow
    }

    /// Get as an event handler pointer (for event binding)
    ///
    /// In wxWidgets, wxWindow inherits from wxEvtHandler, so this is a simple cast.
    pub fn as_evt_handler(&self) -> *mut WxEvtHandler {
        self.ptr as *mut WxEvtHandler
    }

    /// Show the frame
    pub fn show(&self) {
        unsafe {
            wxWindow_Show(self.ptr as *mut WxWindow);
        }
    }

    /// Hide the frame
    pub fn hide(&self) {
        unsafe {
            wxWindow_Hide(self.ptr as *mut WxWindow);
        }
    }

    /// Close the frame
    ///
    /// This generates a close event which can be vetoed. If not vetoed,
    /// the window is destroyed.
    pub fn close(&self, force: bool) -> bool {
        unsafe { wxWindow_Close(self.ptr as *mut WxWindow, force as c_int) != 0 }
    }

    /// Set the frame title
    pub fn set_title(&self, title: &str) {
        let cstr = CString::new(title).expect("String contains null bytes");
        unsafe {
            let wx_str = wxString_CreateUTF8(cstr.as_ptr());
            wxTopLevelWindow_SetTitle(self.ptr, wx_str);
            wxString_Delete(wx_str);
        }
    }

    /// Get the frame title
    pub fn title(&self) -> String {
        unsafe {
            let wx_str = wxTopLevelWindow_GetTitle(self.ptr);
            if wx_str.is_null() {
                return String::new();
            }

            // Get the string content
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

    /// Set the frame size
    pub fn set_size(&self, width: i32, height: i32) {
        unsafe { wxWindow_SetSize(self.ptr as *mut WxWindow, -1, -1, width, height, 0) }
    }

    /// Set the frame position
    pub fn set_position(&self, x: i32, y: i32) {
        unsafe { wxWindow_Move(self.ptr as *mut WxWindow, x, y) }
    }

    /// Center the frame on screen
    pub fn center(&self) {
        unsafe { wxWindow_Center(self.ptr as *mut WxWindow, 3) } // 3 = wxBOTH
    }

    /// Maximize the frame
    pub fn maximize(&self) {
        unsafe { wxFrame_Maximize(self.ptr) }
    }

    /// Minimize (iconize) the frame
    pub fn minimize(&self) {
        unsafe { wxFrame_Iconize(self.ptr) }
    }

    /// Restore the frame from maximized/minimized state
    pub fn restore(&self) {
        unsafe { wxFrame_Restore(self.ptr) }
    }

    /// Check if the frame is maximized
    pub fn is_maximized(&self) -> bool {
        unsafe { wxFrame_IsMaximized(self.ptr) != 0 }
    }

    /// Check if the frame is minimized
    pub fn is_minimized(&self) -> bool {
        unsafe { wxFrame_IsIconized(self.ptr) != 0 }
    }

    /// Create a status bar with the given number of fields
    pub fn create_status_bar(&self, fields: i32) {
        unsafe { wxFrame_CreateStatusBar(self.ptr, fields, 0, wx_id_any()) };
    }

    /// Set the text of one status bar field
    ///
    /// Field 0 is the first pane. Does nothing when the frame has no status bar.
    pub fn set_status_text(&self, text: &str, field: i32) {
        unsafe {
            let status_bar = wxFrame_GetStatusBar(self.ptr);
            if status_bar.is_null() {
                return;
            }

            let cstr = CString::new(text).expect("Status text contains null bytes");
            let wx_text = wxString_CreateUTF8(cstr.as_ptr());
            wxStatusBar_SetStatusText(status_bar, wx_text, field);
            wxString_Delete(wx_text);
        }
    }

    /// Attach a menu bar to the frame
    ///
    /// The frame takes ownership of the menu bar.
    pub fn set_menu_bar(&self, menu_bar: &MenuBar) {
        unsafe { wxFrame_SetMenuBar(self.ptr, menu_bar.as_ptr()) }
    }

    /// Transfer ownership to wxWidgets
    ///
    /// After calling this, dropping the Frame will not close the window.
    /// wxWidgets will manage the window's lifetime.
    pub fn release(mut self) -> *mut WxFrame {
        self.owned_by_parent = true;
        self.ptr
    }
}

impl Drop for Frame {
    fn drop(&mut self) {
        if !self.owned_by_parent && !self.ptr.is_null() {
            // Close the window, which will destroy it
            unsafe { wxWindow_Close(self.ptr as *mut WxWindow, 1) };
        }
    }
}

/// Builder for Frame with fluent configuration
pub struct FrameBuilder {
    title: String,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    style: i32,
    center_on_create: bool,
}

impl FrameBuilder {
    /// Create a new builder with the given title
    pub fn new(title: &str) -> Self {
        FrameBuilder {
            title: title.to_string(),
            x: -1, // wxDefaultCoord
            y: -1,
            width: -1,
            height: -1,
            style: wx_default_frame_style(),
            center_on_create: false,
        }
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

    /// Center the frame on screen after creation
    pub fn center(mut self) -> Self {
        self.center_on_create = true;
        self
    }

    /// Build the frame
    pub fn build(self) -> Frame {
        let cstr = CString::new(self.title).expect("Title contains null bytes");
        let ptr = unsafe {
            let wx_title = wxString_CreateUTF8(cstr.as_ptr());
            let frame = wxFrame_Create(
                ptr::null_mut(), // no parent
                wx_id_any(),
                wx_title,
                self.x,
                self.y,
                self.width,
                self.height,
                self.style,
            );
            wxString_Delete(wx_title);
            frame
        };

        assert!(!ptr.is_null(), "Failed to create frame");

        let frame = Frame {
            ptr,
            owned_by_parent: false,
        };

        if self.center_on_create {
            frame.center();
        }

        frame
    }
}
