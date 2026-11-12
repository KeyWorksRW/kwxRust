//! Safe wrappers for wxMenu and wxMenuBar
//!
//! A `Menu` appended to a `MenuBar` is owned by that menu bar, and a `MenuBar`
//! attached to a `Frame` is owned by the frame. Neither type frees anything:
//! wxWidgets destroys the whole tree with the frame.
//!
//! # Example
//!
//! ```rust,no_run
//! use kwxrust::frame::Frame;
//! use kwxrust::menu::{Menu, MenuBar};
//!
//! let frame = Frame::new("Menu Example");
//! let menu_bar = MenuBar::new();
//!
//! let file_menu = Menu::new();
//! file_menu.append(kwxrust::sys::WX_ID_EXIT, "E&xit\tAlt-X", "Quit this program");
//! menu_bar.append(&file_menu, "&File");
//!
//! frame.set_menu_bar(&menu_bar);
//! ```

use crate::sys::{
    wxMenuBar_Append, wxMenuBar_Create, wxMenu_Append, wxMenu_Create, WxMenu, WxMenuBar,
};
use crate::wxstr::WxStr;

/// A pull-down menu
///
/// Owned by the `MenuBar` it is appended to; dropping a `Menu` that was never
/// appended leaks the native object (wxWidgets has no owner to destroy it).
pub struct Menu {
    ptr: *mut WxMenu,
}

impl Menu {
    /// Create an empty menu
    ///
    /// `new wxMenu` has an empty title; kwxFFI's `wxMenu_Create` dereferences the
    /// title pointer, so it must be a real (empty) wxString rather than null.
    pub fn new() -> Self {
        let title = WxStr::new("");
        let ptr = unsafe { wxMenu_Create(title.as_ptr(), 0) };
        assert!(!ptr.is_null(), "Failed to create wxMenu");
        Menu { ptr }
    }

    /// Get the raw pointer
    pub fn as_ptr(&self) -> *mut WxMenu {
        self.ptr
    }

    /// Append an item
    ///
    /// `text` keeps any accelerator text (`"E&xit\tAlt-X"`) and `help` becomes
    /// the status-bar help string for the item. The id is what an EVT_MENU
    /// handler matches on.
    pub fn append(&self, id: i32, text: &str, help: &str) {
        let label = WxStr::new(text);
        let help_text = WxStr::new(help);
        unsafe {
            wxMenu_Append(self.ptr, id, label.as_ptr(), help_text.as_ptr(), 0);
        }
    }
}

impl Default for Menu {
    fn default() -> Self {
        Self::new()
    }
}

/// A menu bar holding pull-down menus
///
/// Owned by the `Frame` it is attached to.
pub struct MenuBar {
    ptr: *mut WxMenuBar,
}

impl MenuBar {
    /// Create an empty menu bar
    pub fn new() -> Self {
        let ptr = unsafe { wxMenuBar_Create(0) };
        assert!(!ptr.is_null(), "Failed to create wxMenuBar");
        MenuBar { ptr }
    }

    /// Get the raw pointer
    pub fn as_ptr(&self) -> *mut WxMenuBar {
        self.ptr
    }

    /// Append a menu to the bar under the given title
    ///
    /// The menu bar takes ownership of `menu`.
    pub fn append(&self, menu: &Menu, title: &str) {
        let label = WxStr::new(title);
        unsafe {
            wxMenuBar_Append(self.ptr, menu.as_ptr(), label.as_ptr());
        }
    }
}

impl Default for MenuBar {
    fn default() -> Self {
        Self::new()
    }
}
