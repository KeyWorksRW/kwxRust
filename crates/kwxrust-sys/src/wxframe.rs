//! wxFrame FFI bindings
//!
//! wxFrame is a top-level window with optional menu bar, tool bar, and status bar.
//! This also includes wxTopLevelWindow methods inherited by wxFrame.

use super::types::{WxButton, WxFrame, WxIcon, WxMenuBar, WxStatusBar, WxString, WxWindow};
use libc::c_int;

extern "C" {
    // ========================================================================
    // wxFrame Creation
    // ========================================================================

    /// Create a new frame window
    ///
    /// # Parameters
    /// - `parent`: Parent window (typically null for frames)
    /// - `id`: Window ID (use wxID_ANY for auto-generation)
    /// - `title`: Frame title/caption
    /// - `x`, `y`: Position (use -1 for default)
    /// - `width`, `height`: Size (use -1 for default)
    /// - `style`: Frame style flags (use wxDEFAULT_FRAME_STYLE)
    ///
    /// # Safety
    /// - `parent` must be null or a valid wxWindow pointer
    /// - `title` must be a valid wxString pointer
    pub fn wxFrame_Create(
        parent: *mut WxWindow,
        id: c_int,
        title: *mut WxString,
        x: c_int,
        y: c_int,
        width: c_int,
        height: c_int,
        style: c_int,
    ) -> *mut WxFrame;

    // ========================================================================
    // Status Bar
    // ========================================================================

    /// Create a status bar for the frame
    ///
    /// # Parameters
    /// - `number`: Number of fields in the status bar
    /// - `style`: Status bar style (0 for the default)
    /// - `id`: Window ID (use wxID_ANY for auto-generation)
    ///
    /// # Safety
    /// - `self_` must be a valid wxFrame pointer
    pub fn wxFrame_CreateStatusBar(
        self_: *mut WxFrame,
        number: c_int,
        style: c_int,
        id: c_int,
    ) -> *mut WxStatusBar;

    /// Get the frame's status bar
    ///
    /// Returns null when the frame has no status bar.
    ///
    /// # Safety
    /// - `self_` must be a valid wxFrame pointer
    pub fn wxFrame_GetStatusBar(self_: *mut WxFrame) -> *mut WxStatusBar;

    /// Attach a menu bar to the frame
    ///
    /// The frame takes ownership of the menu bar.
    ///
    /// # Safety
    /// - `self_` must be a valid wxFrame pointer
    /// - `menubar` must be a valid wxMenuBar pointer
    pub fn wxFrame_SetMenuBar(self_: *mut WxFrame, menubar: *mut WxMenuBar);

    // ========================================================================
    // Window State
    // ========================================================================

    /// Maximize the frame
    ///
    /// # Safety
    /// - `self_` must be a valid wxFrame pointer
    pub fn wxFrame_Maximize(self_: *mut WxFrame);

    /// Restore the frame from maximized/iconized state
    ///
    /// # Safety
    /// - `self_` must be a valid wxFrame pointer
    pub fn wxFrame_Restore(self_: *mut WxFrame);

    /// Iconize (minimize) the frame
    ///
    /// # Safety
    /// - `self_` must be a valid wxFrame pointer
    pub fn wxFrame_Iconize(self_: *mut WxFrame);

    /// Check if frame is maximized
    ///
    /// # Safety
    /// - `self_` must be a valid wxFrame pointer
    pub fn wxFrame_IsMaximized(self_: *mut WxFrame) -> c_int;

    /// Check if frame is iconized
    ///
    /// # Safety
    /// - `self_` must be a valid wxFrame pointer
    pub fn wxFrame_IsIconized(self_: *mut WxFrame) -> c_int;

    // ========================================================================
    // Icon
    // ========================================================================

    /// Get the frame icon
    ///
    /// # Safety
    /// - `self_` must be a valid wxFrame pointer
    pub fn wxFrame_GetIcon(self_: *mut WxFrame) -> *mut WxIcon;

    /// Set the frame icon
    ///
    /// # Safety
    /// - `self_` must be a valid wxFrame pointer
    /// - `icon` must be a valid wxIcon pointer
    pub fn wxFrame_SetIcon(self_: *mut WxFrame, icon: *mut WxIcon);

    // ========================================================================
    // wxTopLevelWindow methods (inherited by wxFrame)
    // ========================================================================

    /// Get the frame title
    ///
    /// # Safety
    /// - `self_` must be a valid wxTopLevelWindow (wxFrame) pointer
    /// - Caller must delete the returned wxString
    pub fn wxTopLevelWindow_GetTitle(self_: *mut WxFrame) -> *mut WxString;

    /// Set the frame title
    ///
    /// # Safety
    /// - `self_` must be a valid wxTopLevelWindow (wxFrame) pointer
    /// - `title` must be a valid wxString pointer
    pub fn wxTopLevelWindow_SetTitle(self_: *mut WxFrame, title: *mut WxString);

    /// Iconize/restore the window
    ///
    /// # Safety
    /// - `self_` must be a valid wxTopLevelWindow pointer
    pub fn wxTopLevelWindow_Iconize(self_: *mut WxFrame, iconize: c_int);

    /// Check if window is iconized
    ///
    /// # Safety
    /// - `self_` must be a valid wxTopLevelWindow pointer
    pub fn wxTopLevelWindow_IsIconized(self_: *mut WxFrame) -> c_int;

    /// Check if window is maximized
    ///
    /// # Safety
    /// - `self_` must be a valid wxTopLevelWindow pointer
    pub fn wxTopLevelWindow_IsMaximized(self_: *mut WxFrame) -> c_int;

    /// Maximize/restore the window
    ///
    /// # Safety
    /// - `self_` must be a valid wxTopLevelWindow pointer
    pub fn wxTopLevelWindow_Maximize(self_: *mut WxFrame, maximize: c_int);

    /// Check if window is active (has focus)
    ///
    /// # Safety
    /// - `self_` must be a valid wxTopLevelWindow pointer
    pub fn wxTopLevelWindow_IsActive(self_: *mut WxFrame) -> c_int;

    /// Set the default button for the window
    ///
    /// # Safety
    /// - `self_` must be a valid wxTopLevelWindow pointer
    /// - `button` must be a valid wxButton pointer or null
    pub fn wxTopLevelWindow_SetDefaultButton(self_: *mut WxFrame, button: *mut WxButton);

    /// Get the default button for the window
    ///
    /// # Safety
    /// - `self_` must be a valid wxTopLevelWindow pointer
    pub fn wxTopLevelWindow_GetDefaultButton(self_: *mut WxFrame) -> *mut WxButton;

    /// Enable or disable the close button
    ///
    /// # Safety
    /// - `self_` must be a valid wxTopLevelWindow pointer
    pub fn wxTopLevelWindow_EnableCloseButton(self_: *mut WxFrame, enable: c_int) -> c_int;

    /// Set minimum window size
    ///
    /// # Safety
    /// - `self_` must be a valid wxTopLevelWindow pointer
    pub fn wxTopLevelWindow_SetMinSize(self_: *mut WxFrame, width: c_int, height: c_int);

    /// Set maximum window size
    ///
    /// # Safety
    /// - `self_` must be a valid wxTopLevelWindow pointer
    pub fn wxTopLevelWindow_SetMaxSize(self_: *mut WxFrame, width: c_int, height: c_int);
}
