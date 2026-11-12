//! Safe RAII wrapper for wxString
//!
//! `WxStr` owns a wxString pointer and automatically deletes it on drop.
//! This prevents memory leaks from forgotten wxString deletions.

use kwxrust_sys::{
    wxString_CreateUTF8, wxString_Delete, wxString_GetString, wxString_Length, WxString,
};
use libc::c_char;
use std::ffi::CString;

/// RAII wrapper for wxString
///
/// Automatically deletes the wxString when dropped.
/// Create from Rust strings using `WxStr::new()`.
///
/// # Example
///
/// ```rust,no_run
/// use kwxrust::wxstr::WxStr;
///
/// let title = WxStr::new("Hello, wxWidgets!");
/// // title.as_ptr() can be passed to FFI functions
/// // Automatically deleted when `title` goes out of scope
/// ```
pub struct WxStr {
    ptr: *mut WxString,
}

impl WxStr {
    /// Create a new wxString from a Rust string
    ///
    /// The string is converted to UTF-8 for wxWidgets.
    ///
    /// # Panics
    ///
    /// Panics if the string contains null bytes.
    pub fn new(s: &str) -> Self {
        let cstr = CString::new(s).expect("String contains null bytes");
        let ptr = unsafe { wxString_CreateUTF8(cstr.as_ptr()) };
        assert!(!ptr.is_null(), "Failed to create wxString");
        WxStr { ptr }
    }

    /// Create a WxStr from a raw wxString pointer
    ///
    /// # Safety
    ///
    /// - `ptr` must be a valid wxString pointer returned from wxFFI
    /// - Caller transfers ownership to WxStr (it will be deleted on drop)
    pub unsafe fn from_raw(ptr: *mut WxString) -> Self {
        assert!(!ptr.is_null(), "Cannot create WxStr from null pointer");
        WxStr { ptr }
    }

    /// Get the raw pointer for FFI calls
    ///
    /// The pointer remains valid until this WxStr is dropped.
    pub fn as_ptr(&self) -> *mut WxString {
        self.ptr
    }

    /// Get the length of the string in characters
    pub fn len(&self) -> usize {
        unsafe { wxString_Length(self.ptr) }
    }

    /// Check if the string is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Convert to a Rust String
    ///
    /// This copies the string content.
    pub fn to_string(&self) -> String {
        let len = unsafe { wxString_GetString(self.ptr, std::ptr::null_mut()) } as usize;
        if len == 0 {
            return String::new();
        }

        let mut buf: Vec<u8> = vec![0; len + 1];
        unsafe {
            wxString_GetString(self.ptr, buf.as_mut_ptr() as *mut c_char);
        }
        buf.truncate(len);

        String::from_utf8(buf).unwrap_or_else(|_| String::new())
    }

    /// Consume self and return the raw pointer without deleting
    ///
    /// Use this when transferring ownership to wxWidgets.
    /// The caller is responsible for ensuring the pointer is eventually deleted.
    pub fn into_raw(self) -> *mut WxString {
        let ptr = self.ptr;
        std::mem::forget(self);
        ptr
    }
}

impl Drop for WxStr {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe {
                wxString_Delete(self.ptr);
            }
        }
    }
}

impl std::fmt::Display for WxStr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl std::fmt::Debug for WxStr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WxStr")
            .field("value", &self.to_string())
            .finish()
    }
}

// WxStr is not Send or Sync because wxWidgets is not thread-safe
// The lack of these implementations is intentional

#[cfg(test)]
mod tests {
    use super::*;

    // Note: These tests require wxFFI to be available
    // They are disabled by default and can be run with:
    // cargo test --features test-ffi

    #[test]
    #[ignore = "requires wxFFI library"]
    fn test_wxstr_creation() {
        let s = WxStr::new("Hello");
        assert_eq!(s.len(), 5);
        assert!(!s.is_empty());
    }

    #[test]
    #[ignore = "requires wxFFI library"]
    fn test_wxstr_round_trip() {
        let original = "Hello, 世界!";
        let wxs = WxStr::new(original);
        let result = wxs.to_string();
        assert_eq!(original, result);
    }
}
