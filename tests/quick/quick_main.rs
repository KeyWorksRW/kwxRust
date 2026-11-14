//! Hand-written entry point for the kwxRust quick test app.
//!
//! quick.wxui generates quick_mainframe.rs and quick_dlg.rs into this folder; the
//! generated files only build the forms. This file owns the application bootstrap:
//! initialise wxWidgets, create the frame, register it as the top window, run the
//! modal dialog and then enter the event loop.
//!
//! The lifecycle functions are the port's own C shim (kwxRust/src/kwxApp.cpp).
//! kwxgen cannot wrap them because kwxFFI does not export them, so -- exactly as
//! kwxGO's wx/app.go declares its cgo preamble -- the runner declares them here.
//!
//! Build (never launches the GUI):
//!   cargo build --manifest-path tests\quick\Cargo.toml
//! or from the repository root:
//!   build_quick.cmd

#![windows_subsystem = "windows"]

use std::os::raw::c_char;
use std::os::raw::c_void;

use wx::closure::Closure;
use wx::constants;
use wx::events;

mod quick_dlg;
mod quick_mainframe;

extern "C" {
    fn kwxApp_Initialize(argc: i32, argv: *mut *mut c_char) -> i32;
    fn kwxApp_InitAllImageHandlers();
    fn kwxApp_ExitMainLoop();
    fn kwxApp_MainLoop() -> i32;
    fn kwxApp_Shutdown();
    fn kwxApp_SetTopWindow(window: *mut c_void);
}

// State the frame close handler needs. The runner keeps it on the stack of main()
// for the whole life of the frame and hands its address to the closure.
struct CloseState {
    dialog: *mut c_void,
    modal_open: bool,
    requested: bool,
}

// wxWidgets vetoes a frame close while an app-modal dialog is open
// (wxTopLevelWindowBase::OnCloseWindow), so while the dialog is modal the runner
// records the request, ends the modal loop and stops the handler chain
// (Skip(false)); the frame is destroyed once ShowModal has returned. Once the
// dialog is gone the handler skips the event (Skip(true)) and wxWidgets runs its
// default close handling, which destroys the frame and ends the main loop. This
// is the shape quick_main.go and quick_main.lua use.
extern "C" fn on_frame_close(_closure: *mut c_void, data: *mut c_void, event: *mut c_void) {
    if data.is_null() {
        return;
    }
    // SAFETY: main() passes a pointer to a CloseState that outlives the frame.
    let state = unsafe { &mut *(data as *mut CloseState) };

    if state.modal_open && !state.dialog.is_null() {
        state.requested = true;
        state.modal_open = false;
        if !event.is_null() {
            unsafe { wx::sys::wxEvent_Skip(event, 0) };
        }
        unsafe { wx::sys::wxDialog_EndModal(state.dialog, constants::CANCEL()) };
        return;
    }

    // The main loop is running. wxWidgets would end it when the last top-level window is
    // destroyed, but the hidden modal dialog still counts as one, so stop the loop here
    // (Skip(false) keeps the generated handler and the default handler out of the way).
    if !event.is_null() {
        unsafe { wx::sys::wxEvent_Skip(event, 0) };
    }
    unsafe { kwxApp_ExitMainLoop() };
}

fn main() {
    // SAFETY: every call runs on this thread after kwxApp_Initialize() succeeded, and the
    // pointers handed over are the ones the generated forms returned.
    unsafe {
        if kwxApp_Initialize(0, std::ptr::null_mut()) == 0 {
            return;
        }

        // Register the image handlers before any form decodes an embedded bitmap.
        kwxApp_InitAllImageHandlers();

        let frame = match quick_mainframe::create_quick_main_frame(std::ptr::null_mut()) {
            Some(frame) => frame,
            None => {
                kwxApp_Shutdown();
                return;
            }
        };

        frame.show();
        kwxApp_SetTopWindow(frame.as_ptr());

        let mut state = CloseState {
            dialog: std::ptr::null_mut(),
            modal_open: false,
            requested: false,
        };
        let close_closure = Closure::new(
            on_frame_close as *mut c_void,
            &mut state as *mut CloseState as *mut c_void,
        );
        frame.bind(
            constants::ID_ANY(),
            constants::ID_ANY(),
            events::EVT_CLOSE_WINDOW(),
            &close_closure,
        );

        if let Some(dialog) = quick_dlg::create_quick_dialog(frame.as_ptr()) {
            state.dialog = dialog.as_ptr();
            state.modal_open = true;
            dialog.show_modal();
            state.modal_open = false;
        }

        if state.requested {
            // The dialog unwound because the frame was closed: destroy the frame and leave
            // without entering the main loop, so the process exits with code 0.
            frame.destroy();
            kwxApp_Shutdown();
            std::process::exit(0);
        }

        let exit_code = kwxApp_MainLoop();
        kwxApp_Shutdown();
        std::process::exit(exit_code);
    }
}
