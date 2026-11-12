//! wxString FFI bindings
//!
//! wxString is wxWidgets' string class. wxFFI requires UTF-8 mode, so strings
//! are passed as UTF-8 encoded C strings.
//!
//! # Memory Management
//!
//! - Strings created with `wxString_Create*` must be deleted with `wxString_Delete`
//! - Strings returned from getter functions (e.g., `wxWindow_GetLabel`) must also be deleted

use super::types::WxString;
use libc::{c_char, c_int, size_t};

extern "C" {
    /// Create a wxString from a null-terminated C string (platform encoding)
    ///
    /// # Safety
    /// - `buffer` must be a valid null-terminated C string
    /// - Caller must delete the returned wxString with `wxString_Delete`
    pub fn wxString_Create(buffer: *const c_char) -> *mut WxString;

    /// Create a wxString from a null-terminated UTF-8 string
    ///
    /// # Safety
    /// - `buffer` must be a valid null-terminated UTF-8 string
    /// - Caller must delete the returned wxString with `wxString_Delete`
    pub fn wxString_CreateUTF8(buffer: *const c_char) -> *mut WxString;

    /// Create a wxString from a buffer with explicit length
    ///
    /// # Safety
    /// - `buffer` must point to at least `len` valid bytes
    /// - Caller must delete the returned wxString with `wxString_Delete`
    pub fn wxString_CreateLen(buffer: *const c_char, len: c_int) -> *mut WxString;

    /// Delete a wxString
    ///
    /// # Safety
    /// - `s` must be a valid wxString pointer returned from wxString_Create* or a getter
    /// - `s` must not be used after deletion
    pub fn wxString_Delete(s: *mut WxString);

    /// Get the string content as a C string
    ///
    /// If `buffer` is null, returns the required buffer size (not including null terminator).
    /// If `buffer` is not null, copies the string content to the buffer.
    ///
    /// # Safety
    /// - `s` must be a valid wxString pointer
    /// - If `buffer` is not null, it must have enough space for the string plus null terminator
    pub fn wxString_GetString(s: *mut WxString, buffer: *mut c_char) -> c_int;

    /// Get the length of the string in characters
    ///
    /// # Safety
    /// - `s` must be a valid wxString pointer
    pub fn wxString_Length(s: *mut WxString) -> size_t;
}
