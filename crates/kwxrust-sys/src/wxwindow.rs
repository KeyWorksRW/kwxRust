//! wxWindow FFI bindings
//!
//! wxWindow is the base class for all wxWidgets windows/controls.
//! Most controls inherit from wxWindow and share these basic operations.

use super::types::{WxPoint, WxSize, WxString, WxWindow};
use libc::c_int;

extern "C" {
    // ========================================================================
    // Creation and Destruction
    // ========================================================================

    /// Create a basic window
    ///
    /// # Parameters
    /// - `parent`: Parent window (null for top-level)
    /// - `id`: Window ID (use wxID_ANY for auto-generation)
    /// - `x`, `y`: Position (use -1 for default)
    /// - `width`, `height`: Size (use -1 for default)
    /// - `style`: Window style flags
    ///
    /// # Safety
    /// - `parent` must be null or a valid wxWindow pointer
    pub fn wxWindow_Create(
        parent: *mut WxWindow,
        id: c_int,
        x: c_int,
        y: c_int,
        width: c_int,
        height: c_int,
        style: c_int,
    ) -> *mut WxWindow;

    /// Close the window (may be vetoed)
    ///
    /// # Safety
    /// - `self_` must be a valid wxWindow pointer
    pub fn wxWindow_Close(self_: *mut WxWindow, force: c_int) -> c_int;

    /// Destroy the window immediately
    ///
    /// # Safety
    /// - `self_` must be a valid wxWindow pointer
    /// - Do not use the pointer after calling this
    pub fn wxWindow_Destroy(self_: *mut WxWindow) -> c_int;

    // ========================================================================
    // Visibility and State
    // ========================================================================

    /// Show the window
    ///
    /// # Safety
    /// - `self_` must be a valid wxWindow pointer
    pub fn wxWindow_Show(self_: *mut WxWindow) -> c_int;

    /// Hide the window
    ///
    /// # Safety
    /// - `self_` must be a valid wxWindow pointer
    pub fn wxWindow_Hide(self_: *mut WxWindow) -> c_int;

    /// Check if the window is shown
    ///
    /// # Safety
    /// - `self_` must be a valid wxWindow pointer
    pub fn wxWindow_IsShown(self_: *mut WxWindow) -> c_int;

    /// Enable the window
    ///
    /// # Safety
    /// - `self_` must be a valid wxWindow pointer
    pub fn wxWindow_Enable(self_: *mut WxWindow) -> c_int;

    /// Disable the window
    ///
    /// # Safety
    /// - `self_` must be a valid wxWindow pointer
    pub fn wxWindow_Disable(self_: *mut WxWindow) -> c_int;

    /// Check if the window is enabled
    ///
    /// # Safety
    /// - `self_` must be a valid wxWindow pointer
    pub fn wxWindow_IsEnabled(self_: *mut WxWindow) -> c_int;

    // ========================================================================
    // Labels and Names
    // ========================================================================

    /// Set the window label/caption
    ///
    /// # Safety
    /// - `self_` must be a valid wxWindow pointer
    /// - `label` must be a valid wxString pointer
    pub fn wxWindow_SetLabel(self_: *mut WxWindow, label: *mut WxString);

    /// Get the window label/caption
    ///
    /// # Safety
    /// - `self_` must be a valid wxWindow pointer
    /// - Caller must delete the returned wxString
    pub fn wxWindow_GetLabel(self_: *mut WxWindow) -> *mut WxString;

    /// Set the window name (for identification)
    ///
    /// # Safety
    /// - `self_` must be a valid wxWindow pointer
    /// - `name` must be a valid wxString pointer
    pub fn wxWindow_SetName(self_: *mut WxWindow, name: *mut WxString);

    /// Get the window name
    ///
    /// # Safety
    /// - `self_` must be a valid wxWindow pointer
    /// - Caller must delete the returned wxString
    pub fn wxWindow_GetName(self_: *mut WxWindow) -> *mut WxString;

    // ========================================================================
    // IDs
    // ========================================================================

    /// Set the window ID
    ///
    /// # Safety
    /// - `self_` must be a valid wxWindow pointer
    pub fn wxWindow_SetId(self_: *mut WxWindow, id: c_int);

    /// Get the window ID
    ///
    /// # Safety
    /// - `self_` must be a valid wxWindow pointer
    pub fn wxWindow_GetId(self_: *mut WxWindow) -> c_int;

    // ========================================================================
    // Size and Position
    // ========================================================================

    /// Set window size and position
    ///
    /// # Parameters
    /// - `x`, `y`: Position (-1 to keep current)
    /// - `width`, `height`: Size (-1 to keep current)
    /// - `size_flags`: Sizing flags (0 for default)
    ///
    /// # Safety
    /// - `self_` must be a valid wxWindow pointer
    pub fn wxWindow_SetSize(
        self_: *mut WxWindow,
        x: c_int,
        y: c_int,
        width: c_int,
        height: c_int,
        size_flags: c_int,
    );

    /// Move the window
    ///
    /// # Safety
    /// - `self_` must be a valid wxWindow pointer
    pub fn wxWindow_Move(self_: *mut WxWindow, x: c_int, y: c_int);

    /// Get window position
    ///
    /// # Safety
    /// - `self_` must be a valid wxWindow pointer
    /// - Caller must manage the returned wxPoint
    pub fn wxWindow_GetPosition(self_: *mut WxWindow) -> *mut WxPoint;

    /// Get window size
    ///
    /// # Safety
    /// - `self_` must be a valid wxWindow pointer
    /// - Caller must manage the returned wxSize
    pub fn wxWindow_GetSize(self_: *mut WxWindow) -> *mut WxSize;

    /// Set client area size
    ///
    /// # Safety
    /// - `self_` must be a valid wxWindow pointer
    pub fn wxWindow_SetClientSize(self_: *mut WxWindow, width: c_int, height: c_int);

    /// Get client area size
    ///
    /// # Safety
    /// - `self_` must be a valid wxWindow pointer
    /// - Caller must manage the returned wxSize
    pub fn wxWindow_GetClientSize(self_: *mut WxWindow) -> *mut WxSize;

    /// Get best size for the window
    ///
    /// # Safety
    /// - `self_` must be a valid wxWindow pointer
    /// - Caller must manage the returned wxSize
    pub fn wxWindow_GetBestSize(self_: *mut WxWindow) -> *mut WxSize;

    /// Center the window
    ///
    /// # Parameters
    /// - `direction`: Centering direction (1=horizontal, 2=vertical, 3=both)
    ///
    /// # Safety
    /// - `self_` must be a valid wxWindow pointer
    pub fn wxWindow_Center(self_: *mut WxWindow, direction: c_int);

    /// Center on parent
    ///
    /// # Safety
    /// - `self_` must be a valid wxWindow pointer
    pub fn wxWindow_CenterOnParent(self_: *mut WxWindow, direction: c_int);

    /// Fit window to its contents
    ///
    /// # Safety
    /// - `self_` must be a valid wxWindow pointer
    pub fn wxWindow_Fit(self_: *mut WxWindow);

    // ========================================================================
    // Hierarchy
    // ========================================================================

    /// Get the parent window
    ///
    /// # Safety
    /// - `self_` must be a valid wxWindow pointer
    pub fn wxWindow_GetParent(self_: *mut WxWindow) -> *mut WxWindow;

    /// Raise window to top of Z-order
    ///
    /// # Safety
    /// - `self_` must be a valid wxWindow pointer
    pub fn wxWindow_Raise(self_: *mut WxWindow);

    /// Lower window to bottom of Z-order
    ///
    /// # Safety
    /// - `self_` must be a valid wxWindow pointer
    pub fn wxWindow_Lower(self_: *mut WxWindow);

    // ========================================================================
    // Refresh
    // ========================================================================

    /// Refresh the window (marks for repaint)
    ///
    /// # Safety
    /// - `self_` must be a valid wxWindow pointer
    pub fn wxWindow_Refresh(self_: *mut WxWindow, eraseBackground: c_int);

    /// Clear the window background
    ///
    /// # Safety
    /// - `self_` must be a valid wxWindow pointer
    pub fn wxWindow_ClearBackground(self_: *mut WxWindow);

    // ========================================================================
    // Focus
    // ========================================================================

    /// Set focus to this window
    ///
    /// # Safety
    /// - `self_` must be a valid wxWindow pointer
    pub fn wxWindow_SetFocus(self_: *mut WxWindow);
}
