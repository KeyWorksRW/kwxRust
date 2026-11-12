//! wxMenu / wxMenuBar FFI bindings
//!
//! Menus created with `wxMenu_Create` are owned by the menu bar they are
//! appended to, and the menu bar is owned by the frame it is attached to.
//! Nothing here may be deleted manually.

use super::types::{WxMenu, WxMenuBar, WxString};
use libc::{c_int, c_long};

extern "C" {
    /// Create a new menu
    ///
    /// # Parameters
    /// - `title`: Optional title (pass null for none)
    /// - `style`: Menu style (0 for the default)
    ///
    /// # Safety
    /// - `title` must be null or a valid wxString pointer
    pub fn wxMenu_Create(title: *mut WxString, style: c_long) -> *mut WxMenu;

    /// Append a menu item
    ///
    /// # Parameters
    /// - `id`: Command ID (also used to match EVT_MENU handlers)
    /// - `text`: Item label, including any accelerator text
    /// - `help`: Status-bar help string
    /// - `is_checkable`: Non-zero for a checkable item
    ///
    /// # Safety
    /// - `self_` must be a valid wxMenu pointer
    /// - `text` and `help` must be valid wxString pointers
    pub fn wxMenu_Append(
        self_: *mut WxMenu,
        id: c_int,
        text: *mut WxString,
        help: *mut WxString,
        is_checkable: c_int,
    );

    /// Create an empty menu bar
    ///
    /// # Safety
    /// - `style`: Menu bar style (0 for the default)
    pub fn wxMenuBar_Create(style: c_int) -> *mut WxMenuBar;

    /// Append a menu to the menu bar
    ///
    /// The menu bar takes ownership of the menu.
    ///
    /// # Safety
    /// - `self_` must be a valid wxMenuBar pointer
    /// - `menu` must be a valid wxMenu pointer
    /// - `title` must be a valid wxString pointer
    pub fn wxMenuBar_Append(
        self_: *mut WxMenuBar,
        menu: *mut WxMenu,
        title: *mut WxString,
    ) -> c_int;
}
