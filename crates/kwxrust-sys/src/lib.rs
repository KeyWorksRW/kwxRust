//! kwxrust-sys - Raw FFI bindings to wxWidgets via wxFFI
//!
//! This crate provides the raw `extern "C"` declarations for calling wxFFI functions.
//! All types are opaque pointers (`*mut WxXxx`) and functions match the C signatures exactly.
//!
//! # Safety
//!
//! All functions in this crate are unsafe. For safe Rust wrappers, use the `kwxrust` crate.
//!
//! # Type Conventions
//!
//! - `*mut WxXxx` - Opaque pointer to a wxWidgets object
//! - `c_int` - Standard C int (used for IDs, styles, positions, etc.)
//! - `*mut WxString` - Pointer to wxString (must be created/deleted via FFI)
//!
//! # Memory Management
//!
//! - **Windows/Controls**: Owned by their parent, deleted automatically
//! - **wxString**: Must be explicitly deleted with `wxString_Delete`
//! - **Returned objects** (wxString, wxSize, wxPoint): Caller owns, must delete

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

pub mod constants;
pub mod types;

pub mod wxbutton;
pub mod wxclosure;
pub mod wxevent;
pub mod wxevthandler;
pub mod wxframe;
pub mod wxmenu;
pub mod wxmsgbox;
pub mod wxpanel;
pub mod wxsizer;
pub mod wxstatusbar;
pub mod wxstring;
pub mod wxwindow;

// Re-export everything for convenience
pub use constants::*;
pub use types::*;

pub use wxbutton::*;
pub use wxclosure::*;
pub use wxevent::*;
pub use wxevthandler::*;
pub use wxframe::*;
pub use wxmenu::*;
pub use wxmsgbox::*;
pub use wxpanel::*;
pub use wxsizer::*;
pub use wxstatusbar::*;
pub use wxstring::*;
pub use wxwindow::*;
