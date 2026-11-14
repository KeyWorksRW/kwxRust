#![windows_subsystem = "windows"]

//! Port of the wxWidgets `minimal` sample
//! (`C:/rwCode/wxSamples/src/minimal/minimal.cpp`).
//!
//! Reproduces `MyFrame`:
//!
//! * the `Minimal wxWidgets App` frame with `wxDEFAULT_FRAME_STYLE` and the
//!   32x32 sample icon;
//! * the `&File` (`E&xit\tAlt-X`) and `&Help` (`&About\tF1`) menus in a
//!   two-entry menu bar;
//! * a two-pane status bar whose first pane reads `Welcome to wxWidgets!`;
//! * `EVT_MENU` handlers for `wxID_EXIT` (`Close(true)`) and `wxID_ABOUT`
//!   (the modal `About wxWidgets minimal sample` message box).
//!
//! `wxUSE_MENUBAR` and `wxUSE_STATUSBAR` are both enabled in the wxWidgets
//! Windows build, so the `About...` button branch of the C++ sample is not
//! compiled there and is not reproduced here.
//!
//! The icon is loaded from `data/sample.svg` (the C++ `sample.xpm`, which is
//! the icon source for the non-resource builds) through `BitmapBundle`'s
//! `from_svg_file` factory.

use std::cell::RefCell;
use std::os::raw::c_void;
use std::rc::Rc;

use kwxffi_sys::bitmap_bundle::BitmapBundle;
use kwxffi_sys::closure::Closure;
use kwxffi_sys::constants;
use kwxffi_sys::events;
use kwxffi_sys::frame::Frame;
use kwxffi_sys::freefuncs;
use kwxffi_sys::icon::Icon;
use kwxffi_sys::menu::Menu;
use kwxffi_sys::menu_bar::MenuBar;
use kwxffi_sys::support::{Rect, Size, NO_PARENT};

mod app_host;

/// `MyFrame("Minimal wxWidgets App")`.
const FRAME_TITLE: &str = "Minimal wxWidgets App";

/// Directory holding the sample's own assets (`data/` next to `Cargo.toml`).
const ASSET_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/data/");

/// The C callback shape expected by `wxClosure`: `(closureFun, data, event)`.
type Handler = extern "C" fn(*mut c_void, *mut c_void, *mut c_void);

/// `MyFrame`'s member fields. Every widget wrapper is `Copy` and none of them
/// owns a wxWidgets object, so the handles are stored by value.
struct State {
    frame: Frame,
    /// `Closure` values that must keep firing for the lifetime of the frame.
    closures: Vec<Closure>,
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

fn main() {
    std::process::exit(run());
}

fn run() -> i32 {
    // MyApp::OnInit(): wxEntryStart() + the hidden app's OnInit().
    if app_host::initialize() == 0 {
        return 1;
    }

    // MyFrame *frame = new MyFrame("Minimal wxWidgets App");
    let state = build_frame();

    // frame->Show(true)
    let frame = state.borrow().frame;
    app_host::set_top_window(&frame);
    frame.show();

    let exit_code = app_host::main_loop();
    app_host::shutdown();
    exit_code
}

// ---------------------------------------------------------------------------
// MyFrame::MyFrame()
// ---------------------------------------------------------------------------

fn build_frame() -> Rc<RefCell<State>> {
    // wxFrame(nullptr, wxID_ANY, title) -- default position/size and
    // wxDEFAULT_FRAME_STYLE.
    let frame = Frame::new_full(
        &NO_PARENT,
        constants::ID_ANY(),
        FRAME_TITLE,
        Rect::default(),
        constants::DEFAULT_FRAME_STYLE(),
    );

    // SetIcon(wxICON(sample))
    if let Some(icon) = load_sample_icon() {
        frame.set_icon(&icon);
    }

    // #if wxUSE_MENUBAR
    let file_menu = Menu::new("");

    // the "About" item should be in the help menu
    let help_menu = Menu::new("");
    help_menu.append(
        constants::ID_ABOUT(),
        "&About\tF1",
        "Show about dialog",
        false,
    );

    file_menu.append(
        constants::ID_EXIT(),
        "E&xit\tAlt-X",
        "Quit this program",
        false,
    );

    let menu_bar = MenuBar::new();
    menu_bar.append(&file_menu, "&File");
    menu_bar.append(&help_menu, "&Help");
    frame.set_menu_bar(&menu_bar);

    // #if wxUSE_STATUSBAR : CreateStatusBar(2) + SetStatusText(pane 0)
    let _ = frame.create_status_bar(2, constants::STB_DEFAULT_STYLE(), constants::ID_ANY());
    frame.set_status_text("Welcome to wxWidgets!", 0);

    let state = State {
        frame,
        closures: Vec::new(),
    };
    let state = Rc::new(RefCell::new(state));
    connect_events(&state);
    state
}

// ---------------------------------------------------------------------------
// Event table
// ---------------------------------------------------------------------------

fn connect_events(state: &Rc<RefCell<State>>) {
    let ctx = state_ctx(state);
    let frame = state.borrow().frame;

    let quit = closure_of(handle_quit, ctx);
    let about = closure_of(handle_about, ctx);

    let menu_event = events::EVT_MENU();

    // EVT_MENU(Minimal_Quit, MyFrame::OnQuit)
    let quit_id = constants::ID_EXIT();
    frame.bind(quit_id, quit_id, menu_event, &quit);

    // EVT_MENU(Minimal_About, MyFrame::OnAbout)
    let about_id = constants::ID_ABOUT();
    frame.bind(about_id, about_id, menu_event, &about);

    state.borrow_mut().closures = vec![quit, about];
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// `EVT_MENU(Minimal_Quit, MyFrame::OnQuit)` -- `Close(true)`.
extern "C" fn handle_quit(_fun: *mut c_void, data: *mut c_void, event: *mut c_void) {
    if event.is_null() {
        return;
    }
    with_state(data, |state| {
        state.frame.close(true);
    });
}

/// `EVT_MENU(Minimal_About, MyFrame::OnAbout)`.
extern "C" fn handle_about(_fun: *mut c_void, data: *mut c_void, event: *mut c_void) {
    if event.is_null() {
        return;
    }
    with_state(data, |state| {
        let message = format!(
            "Welcome to {}!\n\nThis is the minimal wxWidgets sample\nrunning under {}.",
            freefuncs::wxVersionString(),
            freefuncs::wxGetOsDescription(),
        );
        freefuncs::kwxMessageBox(
            message,
            "About wxWidgets minimal sample",
            constants::OK() | constants::ICON_INFORMATION(),
            &state.frame,
            -1,
            -1,
        );
    });
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn closure_of(handler: Handler, ctx: *mut c_void) -> Closure {
    Closure::new(handler as *const () as *mut c_void, ctx)
}

/// `SetIcon(wxICON(sample))`: `sample.xpm` converted to `data/sample.svg`.
fn load_sample_icon() -> Option<Icon> {
    let path = format!("{ASSET_DIR}sample.svg");
    let size = Size {
        width: 32,
        height: 32,
    };

    let bundle = BitmapBundle::from_svg_file(path.as_str(), size)?;
    let bitmap = bundle.get_bitmap(size)?;

    let icon = Icon::create_default();
    icon.copy_from_bitmap(&bitmap);
    Some(icon)
}

/// Hands the state to a C callback. The `Rc` is intentionally leaked so that
/// every connection keeps a valid strong reference for the process lifetime.
fn state_ctx(state: &Rc<RefCell<State>>) -> *mut c_void {
    Rc::into_raw(Rc::clone(state)) as *mut c_void
}

/// Runs `f` with the borrowed state. Never panics: if the state is already
/// borrowed (a handler that triggers another event) the callback is skipped.
fn with_state(data: *mut c_void, f: impl FnOnce(&mut State)) {
    if data.is_null() {
        return;
    }
    let cell = unsafe { &*(data as *const RefCell<State>) };
    if let Ok(mut state) = cell.try_borrow_mut() {
        f(&mut state);
    }
}
