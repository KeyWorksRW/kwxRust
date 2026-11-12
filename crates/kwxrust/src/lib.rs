//! kwxrust - Safe Rust bindings to wxWidgets via wxFFI
//!
//! This crate provides safe, idiomatic Rust wrappers around wxWidgets, enabling
//! cross-platform GUI development in Rust.
//!
//! # Architecture
//!
//! ```text
//! Rust App → kwxrust (safe) → kwxrust-sys (FFI) → kwxFFI.lib → wxWidgets
//! ```
//!
//! # Quick Start
//!
//! ```rust,no_run
//! use kwxrust::prelude::*;
//!
//! fn main() {
//!     // Initialize wxWidgets
//!     app::init();
//!
//!     // Create the main window
//!     let frame = Frame::builder("Hello wxRust!")
//!         .size(400, 300)
//!         .center()
//!         .build();
//!
//!     // Add a button
//!     let button = Button::new(&frame, "Click Me!");
//!
//!     // Handle button clicks
//!     let conn = event::on_button(&frame, &button, |_evt| {
//!         println!("Button clicked!");
//!     });
//!     conn.forget();
//!
//!     // Show and run
//!     frame.show();
//!     app::run();
//! }
//! ```
//!
//! # Modules
//!
//! - [`app`] - Application lifecycle (init, run, exit)
//! - [`frame`] - Top-level frame windows
//! - [`button`] - Push button controls
//! - [`event`] - Event handling and binding
//! - [`wxstr`] - wxString RAII wrapper
//!
//! # Safety
//!
//! This crate aims to provide completely safe Rust APIs. All unsafe code is
//! contained within this crate and the `kwxrust-sys` crate.

pub mod app;
pub mod button;
pub mod dialog;
pub mod event;
pub mod frame;
pub mod menu;
pub mod wxstr;

// Re-export the sys crate for advanced users who need raw FFI access
pub use kwxrust_sys as sys;

/// Prelude module for convenient imports
///
/// Import everything from here for the common case:
/// ```rust
/// use kwxrust::prelude::*;
/// ```
pub mod prelude {
    pub use super::app;
    pub use super::button::Button;
    pub use super::dialog;
    pub use super::event::{self, Event, EventConnection, EventHandler};
    pub use super::frame::Frame;
    pub use super::menu::{Menu, MenuBar};
    pub use super::wxstr::WxStr;

    // Common constants
    pub use super::sys::{wx_default_frame_style, wx_id_any};
}
