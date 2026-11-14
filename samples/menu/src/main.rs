#![windows_subsystem = "windows"]

//! Port of the wxWidgets `menu` sample
//! (`C:/rwCode/wxSamples/src/menu/menu.cpp`).
//!
//! Reproduces both windows of the original:
//!
//! * `MyFrame` -- the `wxWidgets menu sample` frame: the sample icon, a
//!   one-pane status bar, the five-entry menu bar (`&File`, `Menu&bar`, `&Menu`,
//!   `&Test`, `&Help`) with the 68 stock items, the dynamic menu-item
//!   machinery, the file history menu and the read-only log text control that
//!   fills the top three quarters of the client area.
//! * `MyDialog` -- the modal `Test Dialog` opened from `&File/Show &Dialog`,
//!   holding its own read-only text control that logs menu open/close/highlight.
//!
//! Conversion notes:
//!
//! * `MyEvtHandler` (`PushEventHandler`) is reproduced with `bind`: the generic
//!   `EVT_MENU(wxID_ANY)` logging closure is bound *after* the specific
//!   handlers, and wxWidgets dispatches dynamic entries newest-first, so it
//!   runs before them exactly as the pushed handler does; it calls `Skip()`
//!   so the specific handler still receives the event.
//! * `wxLog`/`wxLogTextCtrl` are installed exactly as in the C++ constructor.
//! * `~MyFrame` deletes the file history, the detached menu and the log target
//!   before the frame's children are destroyed. The port does this from the
//!   `EVT_CLOSE_WINDOW` handler (and again, as a no-op, after the main loop),
//!   which reproduces the C++ destructor ordering.
//! * Every handler returns immediately when it is called with a null event:
//!   `wxClosure::~wxClosure` invokes the callback one last time with `nullptr`
//!   to let the foreign side clean up, and that call must not touch wxWidgets.
//! * The icon is the C++ `../sample.xpm`, converted to `data/sample.svg`.
//!
//! Binding notes:
//!
//! * `MenuItem::set_bitmap` is generated with a `&impl WxBitmap` parameter, but
//!   the native adapter (`wxMenuItem_SetBitmap`) takes a `wxBitmapBundle*`; the
//!   raw `sys::wxMenuItem_SetBitmap` export is called instead so the bundle the
//!   C++ passes is the value the adapter receives. `MenuItem::set_bitmaps` takes
//!   `&impl WxObject`, so the same adapter gets real bundles there without a
//!   raw call (a `Bitmap` would be misread as a bundle and crash).
//! * `wxMenuItem::CreateEx` always passes a null parent menu, so each created
//!   item is appended to its menu before its bitmap is set.
//! * `wxFileHistory::SetMenuPathStyle` has no generated binding, so the three
//!   "Select file history menu style" items keep the default path style.
//! * `wxMenuItem::GetAccel` has no generated binding, so `OnGetMenuItemInfo`
//!   cannot print the "Its accelerator is ..." paragraph.
//! * The 8x8 white image behind the checked-state bitmap is built with
//!   `static_data = true`: `wxImage::Create()` stores the buffer as-is and
//!   frees it with `free()` otherwise, which must not happen to a Rust array.

use std::cell::RefCell;
use std::os::raw::c_void;
use std::rc::Rc;

use kwxffi_sys::art_provider::ArtProvider;
use kwxffi_sys::bitmap::Bitmap;
use kwxffi_sys::bitmap_bundle::BitmapBundle;
use kwxffi_sys::bitmap_vector::BitmapVector;
use kwxffi_sys::closure::Closure;
use kwxffi_sys::command_event::CommandEvent;
use kwxffi_sys::constants;
use kwxffi_sys::context_menu_event::ContextMenuEvent;
use kwxffi_sys::dialog::Dialog;
use kwxffi_sys::event::Event;
use kwxffi_sys::events;
use kwxffi_sys::file_history::FileHistory;
use kwxffi_sys::frame::Frame;
use kwxffi_sys::freefuncs;
use kwxffi_sys::icon::Icon;
use kwxffi_sys::image::Image;
use kwxffi_sys::keys;
use kwxffi_sys::log::Log;
use kwxffi_sys::log_null::LogNull;
use kwxffi_sys::log_text_ctrl::LogTextCtrl;
use kwxffi_sys::menu::Menu;
use kwxffi_sys::menu_bar::MenuBar;
use kwxffi_sys::menu_event::MenuEvent;
use kwxffi_sys::menu_item::MenuItem;
use kwxffi_sys::support::{take_string, Point, Rect, Size, NO_PARENT};
use kwxffi_sys::sys;
use kwxffi_sys::text_ctrl::TextCtrl;
use kwxffi_sys::update_ui_event::UpdateUIEvent;

mod app_host;

/// `wxFrame(nullptr, wxID_ANY, "wxWidgets menu sample")`.
const FRAME_TITLE: &str = "wxWidgets menu sample";

/// `frame->SetStatusText("Welcome to wxWidgets menu sample")`.
const STATUS_WELCOME: &str = "Welcome to wxWidgets menu sample";

/// `wxDialog(parent, wxID_ANY, "Test Dialog")`.
const DIALOG_TITLE: &str = "Test Dialog";

/// Directory holding the sample's own assets (`data/` next to `Cargo.toml`).
const ASSET_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/data/");

/// The C callback shape expected by `wxClosure`: `(closureFun, data, event)`.
type Handler = extern "C" fn(*mut c_void, *mut c_void, *mut c_void);

// ---------------------------------------------------------------------------
// The sample's own ids (`enum { ... }` in menu.cpp, lines 267-336)
// ---------------------------------------------------------------------------

const MENU_FILE_CLEAR_LOG: i32 = 100;
const MENU_FILE_SHOW_DIALOG: i32 = 101;

const MENU_MENUBAR_TOGGLE: i32 = 200;
const MENU_MENUBAR_APPEND: i32 = 201;
const MENU_MENUBAR_INSERT: i32 = 202;
const MENU_MENUBAR_DELETE: i32 = 203;
const MENU_MENUBAR_ENABLE: i32 = 204;
const MENU_MENUBAR_GET_LABEL: i32 = 205;
const MENU_MENUBAR_SET_LABEL: i32 = 206;
const MENU_MENUBAR_FIND_MENU: i32 = 207;

const MENU_MENU_APPEND: i32 = 300;
const MENU_MENU_APPEND_SUB: i32 = 301;
const MENU_MENU_INSERT: i32 = 302;
const MENU_MENU_DELETE: i32 = 303;
const MENU_MENU_DELETE_SUB: i32 = 304;
const MENU_MENU_ENABLE: i32 = 305;
const MENU_MENU_CHECK: i32 = 306;
const MENU_MENU_GET_LABEL: i32 = 307;
const MENU_MENU_SET_LABEL: i32 = 308;
const MENU_MENU_GET_INFO: i32 = 309;
const MENU_MENU_FIND_ITEM: i32 = 310;
const MENU_MENU_FILE_HISTORY1: i32 = 311;
const MENU_MENU_FILE_HISTORY2: i32 = 312;
const MENU_MENU_FILE_HISTORY3: i32 = 313;

const MENU_TEST_NORMAL: i32 = 400;
const MENU_TEST_CHECK: i32 = 401;
const MENU_TEST_RADIO1: i32 = 402;
const MENU_TEST_RADIO2: i32 = 403;
const MENU_TEST_RADIO3: i32 = 404;

const MENU_SUBMENU: i32 = 450;
const MENU_SUBMENU_NORMAL: i32 = 451;
const MENU_SUBMENU_CHECK: i32 = 452;
const MENU_SUBMENU_RADIO1: i32 = 453;
const MENU_SUBMENU_RADIO2: i32 = 454;
const MENU_SUBMENU_RADIO3: i32 = 455;

const MENU_DUMMY_FIRST: i32 = 500;
const MENU_DUMMY_SECOND: i32 = 501;
const MENU_DUMMY_THIRD: i32 = 502;
const MENU_DUMMY_FOURTH: i32 = 503;
const MENU_DUMMY_LAST: i32 = 504;

const MENU_POPUP_TO_BE_DELETED: i32 = 2000;
const MENU_POPUP_TO_BE_GREYED: i32 = 2001;
const MENU_POPUP_TO_BE_CHECKED: i32 = 2002;
const MENU_POPUP_SUBMENU: i32 = 2003;
const MENU_POPUP_CHOICE: i32 = 2004;

/// Text logged when the frame's log window is created.
const INTRO_TEXT: &str = "Brief explanations: the commands in the \"Menu\" menu \
append/insert/delete items to/from the \"Test\" menu.\n\
The commands in the \"Menubar\" menu work with the menubar itself.\n\n\
Right click the band below to test popup menus.\n";

/// Text appended to the dialog's read-only text control.
const DIALOG_INTRO: &str =
    "Dialogs do not have menus, but popup menus should function the same\n\n\
Right click this text ctrl to test popup menus.\n";

/// `static const char *choices[] = { "Apple", "Banana", "Cherry" };`
const CHOICES: [&str; 3] = ["Apple", "Banana", "Cherry"];

/// `MyFrame`'s member fields.
struct State {
    frame: Frame,
    /// `m_textctrl` -- the log window (always created: `USE_LOG_WINDOW`).
    textctrl: Option<TextCtrl>,
    /// `new wxLogTextCtrl(m_textctrl)`.
    log_textctrl: Option<LogTextCtrl>,
    /// `m_logOld` -- the target that was active before the log window existed.
    old_log_target: *mut c_void,
    /// `m_menu` -- the `&File` menu while it is detached from the menu bar.
    detached_menu: Option<Menu>,
    /// `m_countDummy`.
    count_dummy: u32,
    /// `m_fileHistoryMenu` -- created for `UseMenu`, owned by the `&File` menu.
    #[allow(dead_code)]
    file_history_menu: Option<Menu>,
    /// `m_fileHistory`.
    file_history: Option<FileHistory>,
    /// `Closure` values that must keep firing for the lifetime of the frame.
    closures: Vec<Closure>,
}

/// `MyDialog`'s member fields, reachable from its own event callbacks.
struct DialogState {
    dialog: Dialog,
    textctrl: TextCtrl,
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

    let state = build_frame();
    let frame = state.borrow().frame;

    // frame->Show(true);
    app_host::set_top_window(&frame);
    frame.show();

    // #if wxUSE_STATUSBAR: frame->SetStatusText("Welcome to ...")
    frame.set_status_text(STATUS_WELCOME, 0);

    let exit_code = app_host::main_loop();

    // ~MyFrame()
    teardown(&state);

    app_host::shutdown();
    exit_code
}

// ---------------------------------------------------------------------------
// MyFrame::MyFrame()
// ---------------------------------------------------------------------------

fn build_frame() -> Rc<RefCell<State>> {
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
        // wxFrame copies the icon; this handle is not handed to wxWidgets.
        icon.delete();
    }

    // CreateStatusBar()
    let _ = frame.create_status_bar(1, constants::STB_DEFAULT_STYLE(), 0);

    // ---- &File ------------------------------------------------------------
    let file_menu = Menu::new("");

    let stock_sub_menu = Menu::new("");
    for id in stock_ids() {
        // stockSubMenu->Append(wxID_XXX) -- the label comes from the stock table.
        stock_sub_menu.append_by_id(id);
    }
    file_menu.append_sub_menu(&stock_sub_menu, "&Standard items demo", "");

    // wxMenuItem *item = new wxMenuItem(fileMenu, Menu_File_ClearLog,
    //                                   "Clear &log\tCtrl-L");
    // item->SetBitmap(wxArtProvider::GetBitmapBundle(wxART_DELETE, wxART_MENU));
    file_menu.append(MENU_FILE_CLEAR_LOG, "Clear &log\tCtrl-L", "", false);
    if let Some(bundle) = art_bitmap_bundle(constants::ART_DELETE()) {
        if let Some(item) = file_menu.find_item(MENU_FILE_CLEAR_LOG) {
            // The generated `MenuItem::set_bitmap` declares `&impl WxBitmap`, but
            // the native adapter takes a `wxBitmapBundle*`; the raw export is used
            // so the adapter receives the bundle the C++ passes.
            unsafe { sys::wxMenuItem_SetBitmap(item.as_ptr(), bundle.as_ptr()) };
        }
        bundle.delete();
    }
    file_menu.append_separator();

    file_menu.append(
        MENU_FILE_SHOW_DIALOG,
        "Show &Dialog\tCtrl-D",
        "Show a dialog",
        false,
    );
    file_menu.append_separator();

    // #if wxUSE_FILE_HISTORY
    let file_history_menu = Menu::new("");
    let file_history = FileHistory::new(9);
    file_history.use_menu(&file_history_menu);
    file_history.add_file_to_history(absolute_path("menu.cpp"));
    file_history.add_file_to_history(absolute_path("Makefile.in"));
    file_history.add_file_to_history(history_path("minimal", "minimal", "cpp"));
    file_menu.append_sub_menu(&file_history_menu, "Sample file history", "");

    file_menu.append(
        constants::ID_EXIT(),
        "E&xit\tAlt-X",
        "Quit menu sample",
        false,
    );

    // ---- Menu&bar ---------------------------------------------------------
    let menubar_menu = Menu::new("");
    menubar_menu.append(
        MENU_MENUBAR_APPEND,
        "&Append menu\tCtrl-A",
        "Append a menu to the menubar",
        false,
    );
    menubar_menu.append(
        MENU_MENUBAR_INSERT,
        "&Insert menu\tCtrl-I",
        "Insert a menu into the menubar",
        false,
    );
    menubar_menu.append(
        MENU_MENUBAR_DELETE,
        "&Delete menu\tCtrl-D",
        "Delete the last menu from the menubar",
        false,
    );
    menubar_menu.append(
        MENU_MENUBAR_TOGGLE,
        "&Toggle menu\tCtrl-T",
        "Toggle the first menu in the menubar",
        true,
    );
    menubar_menu.append_separator();
    menubar_menu.append(
        MENU_MENUBAR_ENABLE,
        "&Enable menu\tCtrl-E",
        "Enable or disable the last menu",
        true,
    );
    menubar_menu.append_separator();
    menubar_menu.append(
        MENU_MENUBAR_GET_LABEL,
        "&Get menu label\tCtrl-G",
        "Get the label of the last menu",
        false,
    );
    menubar_menu.append(
        MENU_MENUBAR_SET_LABEL,
        "&Set menu label\tCtrl-S",
        "Change the label of the last menu",
        false,
    );
    menubar_menu.append_separator();
    menubar_menu.append(
        MENU_MENUBAR_FIND_MENU,
        "&Find menu from label\tCtrl-F",
        "Find a menu by searching for its label",
        false,
    );

    let sub_menu = Menu::new("");
    sub_menu.append(
        MENU_SUBMENU_NORMAL,
        "&Normal submenu item",
        "Disabled submenu item",
        false,
    );
    sub_menu.append_check_item(
        MENU_SUBMENU_CHECK,
        "&Check submenu item",
        "Check submenu item",
    );
    sub_menu.append_radio_item(MENU_SUBMENU_RADIO1, "Radio item &1", "Radio item");
    sub_menu.append_radio_item(MENU_SUBMENU_RADIO2, "Radio item &2", "Radio item");
    sub_menu.append_radio_item(MENU_SUBMENU_RADIO3, "Radio item &3", "Radio item");

    menubar_menu.append_sub(MENU_SUBMENU, "Submenu", &sub_menu, "");

    // ---- &Menu ------------------------------------------------------------
    let menu_menu = Menu::new("");
    menu_menu.append(
        MENU_MENU_APPEND,
        "&Append menu item\tAlt-A",
        "Append a menu item to the 'Test' menu",
        false,
    );
    menu_menu.append(
        MENU_MENU_APPEND_SUB,
        "&Append sub menu\tAlt-S",
        "Append a sub menu to the 'Test' menu",
        false,
    );
    menu_menu.append(
        MENU_MENU_INSERT,
        "&Insert menu item\tAlt-I",
        "Insert a menu item in head of the 'Test' menu",
        false,
    );
    menu_menu.append(
        MENU_MENU_DELETE,
        "&Delete menu item\tAlt-D",
        "Delete the last menu item from the 'Test' menu",
        false,
    );
    menu_menu.append(
        MENU_MENU_DELETE_SUB,
        "Delete last &submenu\tAlt-K",
        "Delete the last submenu from the 'Test' menu",
        false,
    );
    menu_menu.append_separator();
    menu_menu.append(
        MENU_MENU_ENABLE,
        "&Enable menu item\tAlt-E",
        "Enable or disable the last menu item",
        true,
    );
    menu_menu.append(
        MENU_MENU_CHECK,
        "&Check menu item\tAlt-C",
        "Check or uncheck the last menu item",
        true,
    );

    // #ifdef __WXMSW__: menuMenu->Break(); (other ports append a separator).
    if cfg!(target_os = "windows") {
        menu_menu.break_();
    } else {
        menu_menu.append_separator();
    }

    menu_menu.append(
        MENU_MENU_GET_INFO,
        "Get menu item inf&o\tAlt-O",
        "Show the state of the last menu item",
        false,
    );
    menu_menu.append(
        MENU_MENU_SET_LABEL,
        "Set menu item label\tAlt-L",
        "Set the label of a menu item",
        false,
    );
    menu_menu.append_separator();
    menu_menu.append(
        MENU_MENU_FIND_ITEM,
        "Find menu item from label",
        "Find a menu item by searching for its label",
        false,
    );

    let file_history_style = Menu::new("");
    file_history_style.append_radio_item(MENU_MENU_FILE_HISTORY1, "Hide current path", "");
    file_history_style.append_radio_item(MENU_MENU_FILE_HISTORY2, "Hide all paths", "");
    file_history_style.append_radio_item(MENU_MENU_FILE_HISTORY3, "Show all paths", "");

    menu_menu.append_separator();
    menu_menu.append_sub_menu(&file_history_style, "Select file history menu style", "");

    // ---- &Test ------------------------------------------------------------
    let test_menu = Menu::new("");
    test_menu.append(MENU_TEST_NORMAL, "&Normal item", "", false);
    test_menu.append_separator();
    test_menu.append_check_item(MENU_TEST_CHECK, "&Check item", "");

    // #ifdef __WXMSW__ / #if wxUSE_IMAGE: three check items with bitmaps.
    if cfg!(target_os = "windows") {
        add_bitmap_check_items(&test_menu);
    }

    test_menu.append_separator();
    test_menu.append_radio_item(MENU_TEST_RADIO1, "Radio item &1", "");
    test_menu.append_radio_item(MENU_TEST_RADIO2, "Radio item &2", "");
    test_menu.append_radio_item(MENU_TEST_RADIO3, "Radio item &3", "");

    // ---- &Help ------------------------------------------------------------
    let help_menu = Menu::new("");
    help_menu.append(
        constants::ID_ABOUT(),
        "&About\tF1",
        "About menu sample",
        false,
    );

    // ---- menu bar ---------------------------------------------------------
    let menu_bar = MenuBar::new_full(constants::MB_DOCKABLE());
    menu_bar.append(&file_menu, "&File");
    menu_bar.append(&menubar_menu, "Menu&bar");
    menu_bar.append(&menu_menu, "&Menu");
    menu_bar.append(&test_menu, "&Test");
    menu_bar.append(&help_menu, "&Help");

    // these items should be initially checked
    menu_bar.check(MENU_MENUBAR_TOGGLE, true);
    menu_bar.check(MENU_MENUBAR_ENABLE, true);
    menu_bar.check(MENU_MENU_ENABLE, true);
    menu_bar.check(MENU_MENU_CHECK, false);

    frame.set_menu_bar(&menu_bar);

    // ---- the log window ---------------------------------------------------
    let textctrl = TextCtrl::new_full(
        &frame,
        constants::ID_ANY(),
        "",
        Rect::default(),
        constants::TE_MULTILINE(),
    );
    textctrl.set_editable(false);

    Log::disable_timestamp();
    let old_log_target = Log::get_active_target();
    let log_textctrl = LogTextCtrl::new(&textctrl);
    Log::set_active_target(&log_textctrl);

    freefuncs::LogMessage(INTRO_TEXT);

    let state = State {
        frame,
        textctrl: Some(textctrl),
        log_textctrl: Some(log_textctrl),
        old_log_target,
        detached_menu: None,
        count_dummy: 0,
        file_history_menu: Some(file_history_menu),
        file_history: Some(file_history),
        closures: Vec::new(),
    };
    let state = Rc::new(RefCell::new(state));
    connect_events(&state);
    state
}

/// `~MyFrame()`.
///
/// The frame destructor runs while the frame is still alive, so the close
/// handler calls the same function first; whichever runs first does the work
/// and the other becomes a no-op.
fn teardown(state: &Rc<RefCell<State>>) {
    let mut s = state.borrow_mut();
    frame_destructor(&mut s);
}

/// `~MyFrame()`: `delete m_fileHistory; delete m_menu;` and the restoration of
/// the previous log target, in the C++ order (all before the frame's children
/// are destroyed).
fn frame_destructor(state: &mut State) {
    // delete m_fileHistory;
    if let Some(file_history) = state.file_history.take() {
        file_history.delete();
    }

    // delete m_menu; (the &File menu while it is detached from the menu bar)
    if let Some(menu) = state.detached_menu.take() {
        menu.delete_pointer();
    }

    // delete wxLog::SetActiveTarget(m_logOld);
    restore_log_target(state);
}

/// `delete wxLog::SetActiveTarget(m_logOld)` -- restores the previous log target
/// and releases this frame's `wxLogTextCtrl`.
fn restore_log_target(state: &mut State) {
    let Some(log_textctrl) = state.log_textctrl.take() else {
        return;
    };

    let old_log_target = state.old_log_target;
    if old_log_target.is_null() {
        // No previous target to restore: install a plain logger so the active
        // target never points at the logger being released below.
        let fallback = Log::new();
        Log::set_active_target(&fallback);
        log_textctrl.delete();
        return;
    }

    let previous = unsafe { Log::from_ptr(old_log_target) };
    Log::set_active_target(&previous);
    state.old_log_target = std::ptr::null_mut();

    log_textctrl.delete();
}

// ---------------------------------------------------------------------------
// Event table
// ---------------------------------------------------------------------------

fn connect_events(state: &Rc<RefCell<State>>) {
    let ctx = state_ctx(state);
    let frame = state.borrow().frame;

    let menu_event = events::EVT_MENU();
    let update_ui = events::EVT_UPDATE_UI();

    let mut closures: Vec<Closure> = Vec::new();
    let mut span_closures: Vec<Closure> = Vec::new();

    // Every specific EVT_MENU handler is bound before the generic logging
    // handler; wxWidgets walks the dynamic table newest-first, so the logging
    // handler runs first (like the pushed MyEvtHandler) and skips.
    {
        let mut bind_menu = |id: i32, handler: Handler| {
            let closure = closure_of(handler, ctx);
            frame.bind(id, id, menu_event, &closure);
            closures.push(closure);
        };
        let mut bind_range = |first: i32, last: i32, type_: i32, handler: Handler| {
            let closure = closure_of(handler, ctx);
            frame.bind(first, last, type_, &closure);
            span_closures.push(closure);
        };

        bind_menu(constants::ID_EXIT(), handle_quit);
        bind_menu(MENU_FILE_CLEAR_LOG, handle_clear_log);
        bind_menu(MENU_FILE_SHOW_DIALOG, handle_show_dialog);
        bind_menu(constants::ID_ABOUT(), handle_about);

        bind_menu(MENU_MENUBAR_TOGGLE, handle_toggle_menu);
        bind_menu(MENU_MENUBAR_APPEND, handle_append_menu);
        bind_menu(MENU_MENUBAR_INSERT, handle_insert_menu);
        bind_menu(MENU_MENUBAR_DELETE, handle_delete_menu);
        bind_menu(MENU_MENUBAR_ENABLE, handle_enable_menu);
        bind_menu(MENU_MENUBAR_GET_LABEL, handle_get_label_menu);
        bind_menu(MENU_MENUBAR_SET_LABEL, handle_set_label_menu);
        bind_menu(MENU_MENUBAR_FIND_MENU, handle_find_menu);

        bind_menu(MENU_MENU_APPEND, handle_append_menu_item);
        bind_menu(MENU_MENU_APPEND_SUB, handle_append_sub_menu);
        bind_menu(MENU_MENU_INSERT, handle_insert_menu_item);
        bind_menu(MENU_MENU_DELETE, handle_delete_menu_item);
        bind_menu(MENU_MENU_DELETE_SUB, handle_delete_sub_menu);
        bind_menu(MENU_MENU_ENABLE, handle_enable_menu_item);
        bind_menu(MENU_MENU_CHECK, handle_check_menu_item);
        bind_menu(MENU_MENU_GET_LABEL, handle_get_label_menu_item);
        bind_menu(MENU_MENU_SET_LABEL, handle_set_label_menu_item);
        bind_menu(MENU_MENU_GET_INFO, handle_get_menu_item_info);
        bind_menu(MENU_MENU_FIND_ITEM, handle_find_menu_item);

        bind_menu(MENU_TEST_NORMAL, handle_test_normal);
        bind_menu(MENU_TEST_CHECK, handle_test_check);
        bind_menu(MENU_TEST_RADIO1, handle_test_radio);
        bind_menu(MENU_TEST_RADIO2, handle_test_radio);
        bind_menu(MENU_TEST_RADIO3, handle_test_radio);

        bind_menu(constants::ID_FILE1(), handle_file_history_menu_item);
        bind_menu(constants::ID_FILE2(), handle_file_history_menu_item);
        bind_menu(constants::ID_FILE3(), handle_file_history_menu_item);

        bind_menu(MENU_MENU_FILE_HISTORY1, handle_file_history_style_item);
        bind_menu(MENU_MENU_FILE_HISTORY2, handle_file_history_style_item);
        bind_menu(MENU_MENU_FILE_HISTORY3, handle_file_history_style_item);

        bind_range(MENU_DUMMY_FIRST, MENU_DUMMY_LAST, menu_event, handle_dummy);

        // EVT_UPDATE_UI
        bind_range(
            MENU_FILE_CLEAR_LOG,
            MENU_FILE_CLEAR_LOG,
            update_ui,
            handle_clear_log_update_ui,
        );
        bind_range(
            MENU_SUBMENU_NORMAL,
            MENU_SUBMENU_NORMAL,
            update_ui,
            handle_update_submenu_normal,
        );
        bind_range(
            MENU_SUBMENU_CHECK,
            MENU_SUBMENU_CHECK,
            update_ui,
            handle_update_submenu_check,
        );
        bind_range(
            MENU_SUBMENU_RADIO1,
            MENU_SUBMENU_RADIO3,
            update_ui,
            handle_update_submenu_radio,
        );
        bind_range(
            MENU_MENU_CHECK,
            MENU_MENU_CHECK,
            update_ui,
            handle_update_check_menu_item_ui,
        );

        // EVT_CONTEXT_MENU / EVT_MENU_OPEN / EVT_MENU_CLOSE / EVT_MENU_HIGHLIGHT_ALL
        bind_range(
            constants::ID_ANY(),
            constants::ID_ANY(),
            events::EVT_CONTEXT_MENU(),
            handle_context_menu,
        );
        bind_range(
            constants::ID_ANY(),
            constants::ID_ANY(),
            events::EVT_MENU_OPEN(),
            handle_menu_open,
        );
        bind_range(
            constants::ID_ANY(),
            constants::ID_ANY(),
            events::EVT_MENU_CLOSE(),
            handle_menu_close,
        );
        // EVT_MENU_HIGHLIGHT_ALL == EVT_MENU_HIGHLIGHT(wxID_ANY) in wxWidgets;
        // the generator only exposes the id form.
        bind_range(
            constants::ID_ANY(),
            constants::ID_ANY(),
            events::EVT_MENU_HIGHLIGHT(),
            handle_menu_highlight,
        );

        // EVT_SIZE
        bind_range(
            constants::ID_ANY(),
            constants::ID_ANY(),
            events::EVT_SIZE(),
            handle_size,
        );

        // The C++ frame destructor restores the log target before the log text
        // control is destroyed; a close handler reproduces that ordering.
        bind_range(
            constants::ID_ANY(),
            constants::ID_ANY(),
            events::EVT_CLOSE_WINDOW(),
            handle_close,
        );

        // MyEvtHandler: EVT_MENU(wxID_ANY) -- bound last, processed first.
        bind_range(
            constants::ID_ANY(),
            constants::ID_ANY(),
            menu_event,
            handle_log_menu_event,
        );
    }

    closures.extend(span_closures);
    state.borrow_mut().closures = closures;
}

// ---------------------------------------------------------------------------
// Frame handlers
// ---------------------------------------------------------------------------

/// `MyFrame::OnQuit` -- `Close(true)`.
///
/// The frame handle is copied out before closing: `Close()` dispatches
/// `wxEVT_CLOSE_WINDOW` synchronously, and the close handler needs the state
/// borrow that this handler would otherwise still hold.
extern "C" fn handle_quit(_fun: *mut c_void, data: *mut c_void, event: *mut c_void) {
    if event.is_null() {
        return;
    }
    let Some(frame) = frame_of(data) else {
        return;
    };
    frame.close(true);
}

/// The frame's destructor ordering: `~MyFrame` deletes the file history, the
/// detached menu and the log target before the frame's children are destroyed.
///
/// The close event is always skipped, even if the state is unavailable, so the
/// default handling still destroys the frame.
extern "C" fn handle_close(_fun: *mut c_void, data: *mut c_void, event: *mut c_void) {
    if event.is_null() {
        return;
    }
    if !data.is_null() {
        let cell = unsafe { &*(data as *const RefCell<State>) };
        if let Ok(mut state) = cell.try_borrow_mut() {
            frame_destructor(&mut state);
        }
    }
    if !event.is_null() {
        let event = unsafe { Event::from_ptr(event) };
        event.skip(true);
    }
}

/// `MyFrame::OnClearLog` -- `m_textctrl->Clear()`.
extern "C" fn handle_clear_log(_fun: *mut c_void, data: *mut c_void, event: *mut c_void) {
    if event.is_null() {
        return;
    }
    with_state(data, |state| {
        if let Some(textctrl) = state.textctrl {
            textctrl.clear();
        }
    });
}

/// `MyFrame::OnClearLogUpdateUI` -- enable when the log holds more than 5 lines.
extern "C" fn handle_clear_log_update_ui(_fun: *mut c_void, data: *mut c_void, event: *mut c_void) {
    if event.is_null() {
        return;
    }
    with_state(data, |state| {
        let event = unsafe { UpdateUIEvent::from_ptr(event) };
        let lines = state
            .textctrl
            .map(|textctrl| textctrl.get_number_of_lines())
            .unwrap_or(0);
        event.enable(lines > 5);
    });
}

/// `MyFrame::OnShowDialog` -- `MyDialog dlg(this); dlg.ShowModal();`
extern "C" fn handle_show_dialog(_fun: *mut c_void, data: *mut c_void, event: *mut c_void) {
    if event.is_null() {
        return;
    }
    // The modal loop dispatches the dialog's own events; the state borrow must
    // be released before it starts.
    let Some(frame) = frame_of(data) else {
        return;
    };
    show_test_dialog(&frame);
}

/// `MyFrame::OnAbout`.
extern "C" fn handle_about(_fun: *mut c_void, _data: *mut c_void, event: *mut c_void) {
    if event.is_null() {
        return;
    }
    // `wxMessageBox(message, caption, wxOK | wxICON_INFORMATION)` -- the C++
    // sample passes no parent.
    freefuncs::kwxMessageBox(
        "wxWidgets menu sample\n(c) 1999-2001 Vadim Zeitlin",
        "About wxWidgets menu sample",
        constants::OK() | constants::ICON_INFORMATION(),
        &NO_PARENT,
        -1,
        -1,
    );
}

/// `MyFrame::OnToggleMenu` -- remove/restore the first menu bar entry.
extern "C" fn handle_toggle_menu(_fun: *mut c_void, data: *mut c_void, event: *mut c_void) {
    if event.is_null() {
        return;
    }
    with_state(data, |state| {
        let Some(menu_bar) = state.frame.get_menu_bar() else {
            return;
        };
        match state.detached_menu.take() {
            None => {
                // hide the menu
                state.detached_menu = menu_bar.remove(0);
            }
            Some(menu) => {
                // restore it
                menu_bar.insert(0, &menu, "&File");
            }
        }
    });
}

/// `MyFrame::OnAppendMenu`.
extern "C" fn handle_append_menu(_fun: *mut c_void, data: *mut c_void, event: *mut c_void) {
    if event.is_null() {
        return;
    }
    with_state(data, |state| {
        let Some(menu_bar) = state.frame.get_menu_bar() else {
            return;
        };
        let mut title = String::new();
        let menu = create_dummy_menu(&mut state.count_dummy, Some(&mut title));
        menu_bar.append(&menu, &title);
    });
}

/// `MyFrame::OnInsertMenu` -- inserted at index 4, before `&Help`.
extern "C" fn handle_insert_menu(_fun: *mut c_void, data: *mut c_void, event: *mut c_void) {
    if event.is_null() {
        return;
    }
    with_state(data, |state| {
        let Some(menu_bar) = state.frame.get_menu_bar() else {
            return;
        };
        let mut title = String::new();
        let menu = create_dummy_menu(&mut state.count_dummy, Some(&mut title));
        menu_bar.insert(4, &menu, &title);
    });
}

/// `MyFrame::OnDeleteMenu`.
extern "C" fn handle_delete_menu(_fun: *mut c_void, data: *mut c_void, event: *mut c_void) {
    if event.is_null() {
        return;
    }
    with_state(data, |state| {
        let Some(menu_bar) = state.frame.get_menu_bar() else {
            return;
        };
        let count = menu_bar.get_menu_count();
        if count == 4 {
            // don't let delete the first 4 menus
            freefuncs::LogError("Can't delete any more menus");
        } else if let Some(menu) = menu_bar.remove(count - 1) {
            menu.delete_pointer();
        }
    });
}

/// `MyFrame::OnEnableMenu`.
extern "C" fn handle_enable_menu(_fun: *mut c_void, data: *mut c_void, event: *mut c_void) {
    if event.is_null() {
        return;
    }
    with_state(data, |state| {
        let Some(menu_bar) = state.frame.get_menu_bar() else {
            return;
        };
        let count = menu_bar.get_menu_count();
        let event = unsafe { CommandEvent::from_ptr(event) };
        menu_bar.enable_top(count - 1, event.is_checked());
    });
}

/// `MyFrame::OnGetLabelMenu`.
extern "C" fn handle_get_label_menu(_fun: *mut c_void, data: *mut c_void, event: *mut c_void) {
    if event.is_null() {
        return;
    }
    with_state(data, |state| {
        let Some(menu_bar) = state.frame.get_menu_bar() else {
            return;
        };
        let count = menu_bar.get_menu_count();
        if count == 0 {
            return;
        }
        freefuncs::LogMessage(format!(
            "The label of the last menu item is '{}'",
            menu_bar.get_menu_label(count - 1)
        ));
    });
}

/// `MyFrame::OnSetLabelMenu`.
extern "C" fn handle_set_label_menu(_fun: *mut c_void, data: *mut c_void, event: *mut c_void) {
    if event.is_null() {
        return;
    }
    let Some(frame) = frame_of(data) else {
        return;
    };
    let Some(menu_bar) = frame.get_menu_bar() else {
        return;
    };
    let count = menu_bar.get_menu_count();
    if count == 0 {
        return;
    }
    let label = freefuncs::wxGetTextFromUserEx(
        "Enter new label: ",
        "Change last menu text",
        menu_bar.get_menu_label(count - 1),
        &frame,
        -1,
        -1,
        1,
    );
    if !label.is_empty() {
        menu_bar.set_menu_label(count - 1, label);
    }
}

/// `MyFrame::OnFindMenu`.
extern "C" fn handle_find_menu(_fun: *mut c_void, data: *mut c_void, event: *mut c_void) {
    if event.is_null() {
        return;
    }
    let Some(frame) = frame_of(data) else {
        return;
    };
    let Some(menu_bar) = frame.get_menu_bar() else {
        return;
    };
    let count = menu_bar.get_menu_count();
    if count == 0 {
        return;
    }
    let label = freefuncs::wxGetTextFromUserEx(
        "Enter label to search for: ",
        "Find menu",
        "",
        &frame,
        -1,
        -1,
        1,
    );
    if label.is_empty() {
        return;
    }
    let index = menu_bar.find_menu(&label);
    if index == constants::NOT_FOUND() {
        freefuncs::LogWarning(format!("No menu with label '{}'", label));
    } else {
        freefuncs::LogMessage(format!("Menu {} has label '{}'", index, label));
    }
}

/// `MyFrame::OnDummy` -- the ids in the `500..504` range.
extern "C" fn handle_dummy(_fun: *mut c_void, data: *mut c_void, event: *mut c_void) {
    if event.is_null() {
        return;
    }
    with_state(data, |_state| {
        let event = unsafe { CommandEvent::from_ptr(event) };
        freefuncs::LogMessage(format!(
            "Dummy item #{}",
            event.get_id() - MENU_DUMMY_FIRST + 1
        ));
    });
}

/// `MyFrame::OnTestNormal`.
extern "C" fn handle_test_normal(_fun: *mut c_void, data: *mut c_void, event: *mut c_void) {
    if event.is_null() {
        return;
    }
    with_state(data, |_state| {
        freefuncs::LogMessage("Normal item selected");
    });
}

/// `MyFrame::OnTestCheck`.
extern "C" fn handle_test_check(_fun: *mut c_void, data: *mut c_void, event: *mut c_void) {
    if event.is_null() {
        return;
    }
    with_state(data, |_state| {
        let event = unsafe { CommandEvent::from_ptr(event) };
        freefuncs::LogMessage(format!(
            "Check item {}checked",
            if event.is_checked() { "" } else { "un" }
        ));
    });
}

/// `MyFrame::OnTestRadio` -- ids `402..404`.
extern "C" fn handle_test_radio(_fun: *mut c_void, data: *mut c_void, event: *mut c_void) {
    if event.is_null() {
        return;
    }
    with_state(data, |_state| {
        let event = unsafe { CommandEvent::from_ptr(event) };
        freefuncs::LogMessage(format!(
            "Radio item {} selected",
            event.get_id() - MENU_TEST_RADIO1 + 1
        ));
    });
}

/// `MyFrame::OnFileHistoryMenuItem` (`wxID_FILE1..3`).
extern "C" fn handle_file_history_menu_item(
    _fun: *mut c_void,
    data: *mut c_void,
    event: *mut c_void,
) {
    if event.is_null() {
        return;
    }
    // A modal box must not run while the state is borrowed: its event loop
    // would re-enter these handlers and every event would be dropped.
    let Some(file_history) = file_history_of(data) else {
        return;
    };
    let event = unsafe { CommandEvent::from_ptr(event) };
    let fname = file_history.get_history_file(event.get_id() - constants::ID_FILE1());

    freefuncs::kwxMessageBox(
        format!("Selected file {fname}"),
        "File history activated",
        constants::OK() | constants::ICON_INFORMATION(),
        &NO_PARENT,
        -1,
        -1,
    );

    file_history.add_file_to_history(&fname);
}

/// `MyFrame::OnFileHistoryStyleItem`.
///
/// gap: `wxFileHistory::SetMenuPathStyle` is not exposed by the generated
/// bindings (only the `FH_PATH_SHOW_*` constants exist), so the three items
/// keep the default path style.
extern "C" fn handle_file_history_style_item(
    _fun: *mut c_void,
    data: *mut c_void,
    event: *mut c_void,
) {
    if event.is_null() {
        return;
    }
    with_state(data, |_state| {
        // No-op: see the `gap:` note above.
    });
}

/// `MyFrame::OnMenuEvent` (MyEvtHandler) -- logs every menu command.
extern "C" fn handle_log_menu_event(_fun: *mut c_void, data: *mut c_void, event: *mut c_void) {
    if event.is_null() {
        return;
    }
    with_state(data, |state| {
        let event = unsafe { CommandEvent::from_ptr(event) };
        log_menu_event(state, &event);
        event.skip(true);
    });
}

/// `MyFrame::LogMenuEvent`.
fn log_menu_event(state: &State, event: &CommandEvent) {
    let id = event.get_id();
    let mut msg = format!("Menu command {id}");

    // catch all checkable menubar items and also the check item from the popup
    // menu
    let checkable = state
        .frame
        .get_menu_bar()
        .and_then(|menu_bar| menu_bar.find_item(id))
        .map(|item| item.is_checkable())
        .unwrap_or(false);

    if checkable || id == MENU_POPUP_TO_BE_CHECKED {
        msg.push_str(&format!(
            " (the item is currently {}checked)",
            if event.is_checked() { "" } else { "not " }
        ));
    }

    freefuncs::LogMessage(msg);
}

// ---------------------------------------------------------------------------
// &Menu handlers
// ---------------------------------------------------------------------------

/// `MyFrame::OnAppendMenuItem`.
extern "C" fn handle_append_menu_item(_fun: *mut c_void, data: *mut c_void, event: *mut c_void) {
    if event.is_null() {
        return;
    }
    with_state(data, |state| {
        let Some(menu) = find_test_menu(state) else {
            return;
        };
        menu.append_separator();
        menu.append(
            MENU_DUMMY_THIRD,
            "&Third dummy item\tCtrl-F3",
            "Checkable item",
            true,
        );
    });
}

/// `MyFrame::OnAppendSubMenu`.
extern "C" fn handle_append_sub_menu(_fun: *mut c_void, data: *mut c_void, event: *mut c_void) {
    if event.is_null() {
        return;
    }
    with_state(data, |state| {
        let Some(menu) = find_test_menu(state) else {
            return;
        };
        let sub_menu = create_dummy_menu(&mut state.count_dummy, None);
        menu.append_sub(
            MENU_DUMMY_LAST,
            "&Dummy sub menu",
            &sub_menu,
            "Dummy sub menu help",
        );
    });
}

/// `MyFrame::OnInsertMenuItem`.
extern "C" fn handle_insert_menu_item(_fun: *mut c_void, data: *mut c_void, event: *mut c_void) {
    if event.is_null() {
        return;
    }
    with_state(data, |state| {
        let Some(menu) = find_test_menu(state) else {
            return;
        };
        let item = MenuItem::create_ex(
            MENU_DUMMY_FOURTH,
            "Fourth dummy item\tCtrl-F4",
            "",
            constants::ITEM_NORMAL(),
            &NO_PARENT,
        );
        menu.insert_item(0, &item);

        let separator = MenuItem::create_separator();
        menu.insert_item(1, &separator);
    });
}

/// `MyFrame::OnDeleteMenuItem`.
extern "C" fn handle_delete_menu_item(_fun: *mut c_void, data: *mut c_void, event: *mut c_void) {
    if event.is_null() {
        return;
    }
    with_state(data, |state| {
        let Some(menu) = find_test_menu(state) else {
            return;
        };
        let count = menu.get_menu_item_count();
        if count == 0 {
            freefuncs::LogWarning("No items to delete!");
        } else if let Some(item) = menu.find_item_by_position(count - 1) {
            // `menu->Destroy(item)` -- destroy exactly that item.
            menu.destroy_by_item(&item);
        }
    });
}

/// `MyFrame::OnDeleteSubMenu`.
extern "C" fn handle_delete_sub_menu(_fun: *mut c_void, data: *mut c_void, event: *mut c_void) {
    if event.is_null() {
        return;
    }
    with_state(data, |state| {
        let Some(menu) = find_test_menu(state) else {
            return;
        };
        let count = menu.get_menu_item_count();
        for n in (0..count).rev() {
            if let Some(item) = menu.find_item_by_position(n) {
                if item.is_sub_menu() {
                    // `menu->Destroy(item)` -- destroy exactly that item.
                    menu.destroy_by_item(&item);
                    return;
                }
            }
        }
        freefuncs::LogWarning("No submenu to delete!");
    });
}

/// `MyFrame::OnEnableMenuItem`.
extern "C" fn handle_enable_menu_item(_fun: *mut c_void, data: *mut c_void, event: *mut c_void) {
    if event.is_null() {
        return;
    }
    with_state(data, |state| {
        if let Some(item) = last_menu_item(state) {
            item.enable(!item.is_enabled());
        }
    });
}

/// `MyFrame::OnCheckMenuItem`.
extern "C" fn handle_check_menu_item(_fun: *mut c_void, data: *mut c_void, event: *mut c_void) {
    if event.is_null() {
        return;
    }
    with_state(data, |state| {
        if let Some(item) = last_menu_item(state) {
            if item.is_checkable() {
                item.toggle();
            }
        }
    });
}

/// `MyFrame::OnUpdateCheckMenuItemUI`.
extern "C" fn handle_update_check_menu_item_ui(
    _fun: *mut c_void,
    data: *mut c_void,
    event: *mut c_void,
) {
    if event.is_null() {
        return;
    }
    with_state(data, |state| {
        let event = unsafe { UpdateUIEvent::from_ptr(event) };
        // wxLogNull nolog;
        let nolog = LogNull::new();
        let checkable = last_menu_item(state)
            .map(|item| item.is_checkable())
            .unwrap_or(false);
        event.enable(checkable);
        nolog.delete();
    });
}

/// `MyFrame::OnGetLabelMenuItem` -- bound to id 307, which no menu item uses
/// (the C++ event table binds it although the item is never appended).
extern "C" fn handle_get_label_menu_item(_fun: *mut c_void, data: *mut c_void, event: *mut c_void) {
    if event.is_null() {
        return;
    }
    with_state(data, |state| {
        if let Some(item) = last_menu_item(state) {
            freefuncs::LogMessage(format!(
                "The label of the last menu item is '{}'",
                item.get_item_label()
            ));
        }
    });
}

/// `MyFrame::OnSetLabelMenuItem`.
extern "C" fn handle_set_label_menu_item(_fun: *mut c_void, data: *mut c_void, event: *mut c_void) {
    if event.is_null() {
        return;
    }
    let Some(frame) = frame_of(data) else {
        return;
    };
    let Some(item) = last_menu_item_of(data) else {
        return;
    };
    let mut label = freefuncs::wxGetTextFromUserEx(
        "Enter new label: ",
        "Change last menu item text",
        item.get_item_label(),
        &frame,
        -1,
        -1,
        1,
    );
    label = label.replace("\\t", "\t");
    if !label.is_empty() {
        item.set_item_label(label);
    }
}

/// `MyFrame::OnGetMenuItemInfo`.
///
/// gap: the `#if wxUSE_ACCEL` paragraph is omitted because
/// `wxMenuItem::GetAccel` has no generated binding (`sub_wx_surface`:
/// `bound: false`, no raw export either).
extern "C" fn handle_get_menu_item_info(_fun: *mut c_void, data: *mut c_void, event: *mut c_void) {
    if event.is_null() {
        return;
    }
    with_state(data, |state| {
        let Some(item) = last_menu_item(state) else {
            return;
        };
        let mut msg = format!(
            "The item is {}\n",
            if item.is_enabled() {
                "enabled"
            } else {
                "disabled"
            }
        );

        if item.is_checkable() {
            msg.push_str(&format!(
                "It is checkable and {}checked\n",
                if item.is_checked() { "" } else { "un" }
            ));
        }

        freefuncs::LogMessage(msg);
    });
}

/// `MyFrame::OnFindMenuItem`.
extern "C" fn handle_find_menu_item(_fun: *mut c_void, data: *mut c_void, event: *mut c_void) {
    if event.is_null() {
        return;
    }
    let Some(frame) = frame_of(data) else {
        return;
    };
    let Some(menu_bar) = frame.get_menu_bar() else {
        return;
    };
    let count = menu_bar.get_menu_count();
    if count == 0 {
        return;
    }

    let label = freefuncs::wxGetTextFromUserEx(
        "Enter label to search for: ",
        "Find menu item",
        "",
        &frame,
        -1,
        -1,
        1,
    );
    if label.is_empty() {
        return;
    }

    let mut menu_index: i32 = 0;
    let mut index = constants::NOT_FOUND();

    while (menu_index < count) && (index == constants::NOT_FOUND()) {
        let title = menu_bar
            .get_menu(menu_index)
            .map(|menu| menu.get_title())
            .unwrap_or_default();
        index = menu_bar.find_menu_item(title, &label);
        menu_index += 1;
    }

    if index == constants::NOT_FOUND() {
        freefuncs::LogWarning(format!("No menu item with label '{}'", label));
    } else {
        freefuncs::LogMessage(format!(
            "Menu item {} in menu {} has label '{}'",
            index, menu_index, label
        ));
    }
}

// ---------------------------------------------------------------------------
// Update-UI handlers
// ---------------------------------------------------------------------------

/// `MyFrame::OnUpdateSubMenuNormal`.
extern "C" fn handle_update_submenu_normal(
    _fun: *mut c_void,
    data: *mut c_void,
    event: *mut c_void,
) {
    if event.is_null() {
        return;
    }
    with_state(data, |_state| {
        let event = unsafe { UpdateUIEvent::from_ptr(event) };
        event.enable(false);
    });
}

/// `MyFrame::OnUpdateSubMenuCheck`.
extern "C" fn handle_update_submenu_check(
    _fun: *mut c_void,
    data: *mut c_void,
    event: *mut c_void,
) {
    if event.is_null() {
        return;
    }
    with_state(data, |_state| {
        let event = unsafe { UpdateUIEvent::from_ptr(event) };
        event.enable(true);
    });
}

/// `MyFrame::OnUpdateSubMenuRadio` -- always selects `Radio item &2`.
extern "C" fn handle_update_submenu_radio(
    _fun: *mut c_void,
    data: *mut c_void,
    event: *mut c_void,
) {
    if event.is_null() {
        return;
    }
    with_state(data, |_state| {
        let event = unsafe { UpdateUIEvent::from_ptr(event) };
        let which = event.get_id() - MENU_SUBMENU_RADIO1 + 1;
        event.check(which == 2);
    });
}

// ---------------------------------------------------------------------------
// Context menu and menu open/close/highlight
// ---------------------------------------------------------------------------

/// `MyFrame::OnContextMenu`.
extern "C" fn handle_context_menu(_fun: *mut c_void, data: *mut c_void, event: *mut c_void) {
    if event.is_null() {
        return;
    }

    // The frame handle is copied out before the popup runs: `PopupMenu()`
    // dispatches the selected command synchronously, and those handlers need
    // the state borrow this handler would otherwise still hold.
    let Some(frame) = frame_of(data) else {
        return;
    };

    let event = unsafe { ContextMenuEvent::from_ptr(event) };
    let mut x: i32 = 0;
    let mut y: i32 = 0;
    event.get_position(&mut x as *mut i32, &mut y as *mut i32);

    let point = if x == -1 && y == -1 {
        // If from keyboard
        match frame.get_size() {
            Some(size) => {
                let point = Point {
                    x: size.get_width() / 2,
                    y: size.get_height() / 2,
                };
                size.delete();
                point
            }
            None => Point { x, y },
        }
    } else {
        match frame.screen_to_client(Point { x, y }) {
            Some(client) => {
                let point = Point {
                    x: client.get_x(),
                    y: client.get_y(),
                };
                client.delete();
                point
            }
            None => Point { x, y },
        }
    };

    show_context_menu(&frame, point);
}

/// `MyFrame::ShowContextMenu`.
fn show_context_menu(frame: &Frame, pos: Point) {
    if freefuncs::wxGetKeyState(keys::K_SHIFT()) {
        // when Shift is pressed, demonstrate the use of a simple function
        // returning the id of the item selected in the popup menu
        let menu = Menu::new("");
        menu.set_title("Choose one of:");
        for (n, choice) in CHOICES.iter().enumerate() {
            menu.append(MENU_POPUP_CHOICE + n as i32, *choice, "", false);
        }

        let rc = frame.get_popup_menu_selection_from_user(&menu, pos);
        let choice = CHOICES.get((rc - MENU_POPUP_CHOICE) as usize);
        match choice {
            Some(choice) if rc != constants::ID_NONE() => {
                freefuncs::LogMessage(format!("You have selected \"{}\"", choice));
            }
            _ => freefuncs::LogMessage("No selection"),
        }
        menu.delete_pointer();
    } else {
        // normal case, shift not pressed
        let menu = Menu::new("");
        menu.append(constants::ID_ABOUT(), "&About", "", false);
        let mut unused_count = 0u32;
        let sub_menu = create_dummy_menu(&mut unused_count, None);
        menu.append_sub(MENU_POPUP_SUBMENU, "&Submenu", &sub_menu, "");
        menu.append(MENU_POPUP_TO_BE_DELETED, "To be &deleted", "", false);
        menu.append_check_item(MENU_POPUP_TO_BE_CHECKED, "To be &checked", "");
        menu.append(
            MENU_POPUP_TO_BE_GREYED,
            "To be &greyed",
            "This menu item should be initially greyed out",
            false,
        );
        menu.append_separator();

        // menu.Append(Menu_File_Quit, "E&xit")
        //     ->SetBitmap(wxArtProvider::GetBitmapBundle(wxART_QUIT, wxART_MENU));
        let exit_item = MenuItem::create_ex(
            constants::ID_EXIT(),
            "E&xit",
            "",
            constants::ITEM_NORMAL(),
            &NO_PARENT,
        );
        // As above: append before setting the bitmap.
        menu.append_item(&exit_item);
        if let Some(bundle) = art_bitmap_bundle(constants::ART_QUIT()) {
            unsafe { sys::wxMenuItem_SetBitmap(exit_item.as_ptr(), bundle.as_ptr()) };
            bundle.delete();
        }

        menu.delete_by_id(MENU_POPUP_TO_BE_DELETED);
        menu.check(MENU_POPUP_TO_BE_CHECKED, true);
        menu.enable(MENU_POPUP_TO_BE_GREYED, false);

        frame.popup_menu(&menu, pos);
        menu.delete_pointer();
    }
}

/// `MyFrame::OnMenuOpen`.
extern "C" fn handle_menu_open(_fun: *mut c_void, data: *mut c_void, event: *mut c_void) {
    if event.is_null() {
        return;
    }
    with_state(data, |state| {
        log_menu_open_close_or_highlight(&state.frame, event, "opened");
    });
}

/// `MyFrame::OnMenuClose`.
extern "C" fn handle_menu_close(_fun: *mut c_void, data: *mut c_void, event: *mut c_void) {
    if event.is_null() {
        return;
    }
    with_state(data, |state| {
        log_menu_open_close_or_highlight(&state.frame, event, "closed");
    });
}

/// `MyFrame::OnMenuHighlight` (`EVT_MENU_HIGHLIGHT_ALL`).
extern "C" fn handle_menu_highlight(_fun: *mut c_void, data: *mut c_void, event: *mut c_void) {
    if event.is_null() {
        return;
    }
    with_state(data, |state| {
        log_menu_open_close_or_highlight(&state.frame, event, "highlighted");
    });
}

/// `MyFrame::LogMenuOpenCloseOrHighlight`.
fn log_menu_open_close_or_highlight(frame: &Frame, event: *mut c_void, what: &str) {
    let event = unsafe { MenuEvent::from_ptr(event) };

    let mut msg = String::from("A ");
    if event.is_popup() {
        msg.push_str("popup ");
    }
    msg.push_str("menu has been ");
    msg.push_str(what);

    if event.get_event_type() == events::EVT_MENU_HIGHLIGHT() {
        msg.push_str(&format!(" (id={} in ", event.get_id()));
    } else {
        msg.push_str(" (");
    }

    let menu_ptr = event.get_menu();
    if menu_ptr.is_null() {
        msg.push_str("no menu provided)");
    } else {
        // The generated accessor hands back the wxMenu of the event.
        let menu = unsafe { Menu::from_ptr(menu_ptr) };
        msg.push_str(&format!("menu with title \"{}\")", menu.get_title()));
    }
    msg.push('.');

    freefuncs::LogStatusWindow(frame, msg);
    event.skip(true);
}

/// `MyFrame::OnSize` -- keep a band below the text control for popup menus.
extern "C" fn handle_size(_fun: *mut c_void, data: *mut c_void, event: *mut c_void) {
    if event.is_null() {
        return;
    }
    with_state(data, |state| {
        let Some(textctrl) = state.textctrl else {
            return;
        };
        let Some(size) = state.frame.get_client_size() else {
            return;
        };
        // leave a band below for popup menu testing
        textctrl.set_size(
            Rect::new(0, 0, size.get_width(), (3 * size.get_height()) / 4),
            constants::SIZE_AUTO(),
        );
        size.delete();
    });
}

// ---------------------------------------------------------------------------
// MyDialog
// ---------------------------------------------------------------------------

/// `MyDialog::MyDialog(parent)` + `dlg.ShowModal()`.
fn show_test_dialog(parent: &Frame) {
    let dialog = Dialog::new_full(
        parent,
        constants::ID_ANY(),
        DIALOG_TITLE,
        Rect::default(),
        constants::DEFAULT_DIALOG_STYLE(),
    );

    let textctrl = TextCtrl::new_full(
        &dialog,
        constants::ID_ANY(),
        "",
        Rect::default(),
        constants::TE_MULTILINE(),
    );
    textctrl.set_editable(false);
    textctrl.append_text(DIALOG_INTRO);

    let ctx = Box::into_raw(Box::new(DialogState {
        dialog,
        textctrl,
        closures: Vec::new(),
    }));
    let data = ctx as *mut c_void;

    let mut closures: Vec<Closure> = Vec::new();

    let context = closure_of(handle_dialog_context_menu, data);
    let menu_open = closure_of(handle_dialog_menu_open, data);
    let menu_close = closure_of(handle_dialog_menu_close, data);
    let menu_highlight = closure_of(handle_dialog_menu_highlight, data);

    let any = constants::ID_ANY();
    dialog.bind(any, any, events::EVT_CONTEXT_MENU(), &context);
    dialog.bind(any, any, events::EVT_MENU_OPEN(), &menu_open);
    dialog.bind(any, any, events::EVT_MENU_CLOSE(), &menu_close);
    dialog.bind(any, any, events::EVT_MENU_HIGHLIGHT(), &menu_highlight);

    closures.extend([context, menu_open, menu_close, menu_highlight]);
    unsafe { (*ctx).closures = closures };

    dialog.show_modal();

    // The C++ dialog is a stack object destroyed when ShowModal returns.
    dialog.destroy();
    // `wxWindow::Destroy()` defers the native deletion to the next idle pass,
    // so the dialog's callback context is intentionally not freed here: an
    // event delivered before that deletion would otherwise see freed memory.
}

/// `MyDialog::OnContextMenu`.
extern "C" fn handle_dialog_context_menu(_fun: *mut c_void, data: *mut c_void, event: *mut c_void) {
    if event.is_null() {
        return;
    }
    with_dialog(data, |state| {
        let event = unsafe { ContextMenuEvent::from_ptr(event) };
        let mut x: i32 = 0;
        let mut y: i32 = 0;
        event.get_position(&mut x as *mut i32, &mut y as *mut i32);

        let point = if x == -1 && y == -1 {
            match state.dialog.get_size() {
                Some(size) => {
                    let point = Point {
                        x: size.get_width() / 2,
                        y: size.get_height() / 2,
                    };
                    size.delete();
                    point
                }
                None => Point { x, y },
            }
        } else {
            match state.dialog.screen_to_client(Point { x, y }) {
                Some(client) => {
                    let point = Point {
                        x: client.get_x(),
                        y: client.get_y(),
                    };
                    client.delete();
                    point
                }
                None => Point { x, y },
            }
        };

        show_dialog_context_menu(state, point);
    });
}

/// `MyDialog::ShowContextMenu`.
fn show_dialog_context_menu(state: &DialogState, pos: Point) {
    let menu = Menu::new("");
    menu.append(constants::ID_ABOUT(), "&About", "", false);
    menu.append(MENU_POPUP_TO_BE_DELETED, "To be &deleted", "", false);
    menu.append_check_item(MENU_POPUP_TO_BE_CHECKED, "To be &checked", "");
    menu.append(
        MENU_POPUP_TO_BE_GREYED,
        "To be &greyed",
        "This menu item should be initially greyed out",
        false,
    );
    menu.append_separator();
    menu.append(constants::ID_EXIT(), "E&xit", "", false);

    menu.delete_by_id(MENU_POPUP_TO_BE_DELETED);
    menu.check(MENU_POPUP_TO_BE_CHECKED, true);
    menu.enable(MENU_POPUP_TO_BE_GREYED, false);

    state.dialog.popup_menu(&menu, pos);
    menu.delete_pointer();
}

/// `MyDialog::OnMenuOpen`.
extern "C" fn handle_dialog_menu_open(_fun: *mut c_void, data: *mut c_void, event: *mut c_void) {
    if event.is_null() {
        return;
    }
    with_dialog(data, |state| {
        log_dialog_menu_event(state, event, "opened");
    });
}

/// `MyDialog::OnMenuClose`.
extern "C" fn handle_dialog_menu_close(_fun: *mut c_void, data: *mut c_void, event: *mut c_void) {
    if event.is_null() {
        return;
    }
    with_dialog(data, |state| {
        log_dialog_menu_event(state, event, "closed");
    });
}

/// `MyDialog::OnMenuHighlight`.
extern "C" fn handle_dialog_menu_highlight(
    _fun: *mut c_void,
    data: *mut c_void,
    event: *mut c_void,
) {
    if event.is_null() {
        return;
    }
    with_dialog(data, |state| {
        log_dialog_menu_event(state, event, "highlighted");
    });
}

/// `MyDialog::LogMenuOpenCloseOrHighlight`.
fn log_dialog_menu_event(state: &DialogState, event: *mut c_void, what: &str) {
    let event = unsafe { MenuEvent::from_ptr(event) };

    let mut msg = String::from("A ");
    if event.is_popup() {
        msg.push_str("popup ");
    }
    msg.push_str("menu has been ");
    msg.push_str(what);
    if event.get_event_type() == events::EVT_MENU_HIGHLIGHT() {
        msg.push_str(&format!(" (id={})", event.get_id()));
    }
    msg.push_str(".\n");

    state.textctrl.append_text(msg);
    event.skip(true);
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// `MyFrame::CreateDummyMenu`.
fn create_dummy_menu(count_dummy: &mut u32, title: Option<&mut String>) -> Menu {
    let menu = Menu::new("");
    menu.append(MENU_DUMMY_FIRST, "&First item\tCtrl-F1", "", false);
    menu.append_separator();
    menu.append_check_item(MENU_DUMMY_SECOND, "&Second item\tCtrl-F2", "");

    if let Some(title) = title {
        *count_dummy += 1;
        *title = format!("Dummy menu &{}", count_dummy);
    }

    menu
}

/// `MyFrame::GetLastMenuItem`.
fn last_menu_item(state: &State) -> Option<MenuItem> {
    let menu = find_test_menu(state)?;

    let count = menu.get_menu_item_count();
    if count == 0 {
        freefuncs::LogWarning("No last item in the 'Test' menu!");
        None
    } else {
        menu.find_item_by_position(count - 1)
    }
}

/// `menubar->GetMenu(menubar->FindMenu("Test"))`.
fn find_test_menu(state: &State) -> Option<Menu> {
    let menu_bar = state.frame.get_menu_bar()?;
    let index = menu_bar.find_menu("Test");
    if index == constants::NOT_FOUND() {
        return None;
    }
    menu_bar.get_menu(index)
}

/// Adds the three `__WXMSW__` check items that carry 8x8 bitmaps.
fn add_bitmap_check_items(test_menu: &Menu) {
    // wxBitmap bmpUnchecked(8, 8);
    let bmp_unchecked = Bitmap::create_empty(
        Size {
            width: 8,
            height: 8,
        },
        -1,
    );

    // wxImage imageChecked(8, 8); imageChecked.Clear(0xff);
    //
    // `wxImage::Create()` stores the buffer as-is and frees it with `free()`
    // unless it was created with `static_data`, so the Rust array is handed
    // over as static data and stays owned by this function.
    let rgb = [0xffu8; 8 * 8 * 3];
    let image_checked = Image::create_from_data_ex(8, 8, rgb.as_ptr() as *mut c_void, true);
    let bmp_checked = Bitmap::create_from_image(&image_checked, -1);

    // wxMenuItem::SetBitmaps() takes wxBitmapBundle objects in the native
    // adapter, so each bitmap is wrapped in a one-element bundle.
    let (bundle_checked, bundle_unchecked) =
        match (bundle_of(&bmp_checked), bundle_of(&bmp_unchecked)) {
            (Some(checked), Some(unchecked)) => (checked, unchecked),
            (checked, unchecked) => {
                freefuncs::LogWarning("Could not create the check item bitmaps");
                if let Some(bundle) = checked {
                    bundle.delete();
                }
                if let Some(bundle) = unchecked {
                    bundle.delete();
                }
                image_checked.delete();
                bmp_checked.delete();
                bmp_unchecked.delete();
                return;
            }
        };

    // "Check item with bitmaps"
    let item = MenuItem::create_ex(
        constants::ID_ANY(),
        "Check item with bitmaps",
        "",
        constants::ITEM_CHECK(),
        &NO_PARENT,
    );
    test_menu.append_item(&item);
    item.set_bitmaps(&bundle_checked, &bundle_unchecked);

    // "Check item with bitmaps set afterwards"
    let item = MenuItem::create_ex(
        constants::ID_ANY(),
        "Check item with bitmaps set afterwards",
        "",
        constants::ITEM_CHECK(),
        &NO_PARENT,
    );
    test_menu.append_item(&item);
    item.set_bitmaps(&bundle_checked, &bundle_unchecked);

    // "Check item with bitmaps set afterwards (initially checked)"
    let item = MenuItem::create_ex(
        constants::ID_ANY(),
        "Check item with bitmaps set afterwards (initially checked)",
        "",
        constants::ITEM_CHECK(),
        &NO_PARENT,
    );
    test_menu.append_item(&item);
    item.check(true);
    item.set_bitmaps(&bundle_checked, &bundle_unchecked);

    bundle_checked.delete();
    bundle_unchecked.delete();
    // Static image data: the destructor does not free the Rust array.
    image_checked.delete();
    bmp_checked.delete();
    bmp_unchecked.delete();
}

/// `wxBitmapBundle::FromBitmap(bmp)`: the generator exposes only the vector
/// factory, so a single-element `wxVector<wxBitmap>` is used.
fn bundle_of(bitmap: &Bitmap) -> Option<BitmapBundle> {
    let vector = BitmapVector::new();
    vector.add(bitmap);
    let bundle = BitmapBundle::from_bitmaps(&vector);
    vector.delete();
    bundle
}

/// `wxArtProvider::GetBitmapBundle(id, wxART_MENU)`.
fn art_bitmap_bundle(id: *mut c_void) -> Option<BitmapBundle> {
    // The generated ART_* constants hand back a fresh wxString.
    let id = take_string(id);
    let client = take_string(constants::ART_MENU());
    ArtProvider::get_bitmap_bundle(
        id,
        client,
        Size {
            width: -1,
            height: -1,
        },
    )
}

/// `SetIcon(wxICON(sample))`: `sample.xpm` converted to `data/sample.svg`.
fn load_sample_icon() -> Option<Icon> {
    let path = format!("{ASSET_DIR}sample.svg");
    let size = Size {
        width: 32,
        height: 32,
    };

    let bundle = BitmapBundle::from_svg_file(path.as_str(), size)?;
    let Some(bitmap) = bundle.get_bitmap(size) else {
        bundle.delete();
        return None;
    };

    let icon = Icon::create_default();
    icon.copy_from_bitmap(&bitmap);

    // wxIcon copies the bitmap into a native icon; the intermediates are ours.
    bitmap.delete();
    bundle.delete();
    Some(icon)
}

/// `wxFileName(name).GetAbsolutePath()`.
fn absolute_path(name: &str) -> String {
    std::env::current_dir()
        .unwrap_or_default()
        .join(name)
        .to_string_lossy()
        .into_owned()
}

/// `wxFileName(path, name, ext).GetAbsolutePath()`.
fn history_path(path: &str, name: &str, ext: &str) -> String {
    let mut full = std::env::current_dir().unwrap_or_default();
    full.push(path);
    full.push(format!("{name}.{ext}"));
    full.to_string_lossy().into_owned()
}

/// The stock ids appended to `&Standard items demo`, in the C++ order.
fn stock_ids() -> [i32; 68] {
    [
        constants::ID_ADD(),
        constants::ID_APPLY(),
        constants::ID_BACKWARD(),
        constants::ID_BOLD(),
        constants::ID_BOTTOM(),
        constants::ID_CANCEL(),
        constants::ID_CDROM(),
        constants::ID_CLEAR(),
        constants::ID_CLOSE(),
        constants::ID_CONVERT(),
        constants::ID_COPY(),
        constants::ID_CUT(),
        constants::ID_DELETE(),
        constants::ID_DOWN(),
        constants::ID_EXECUTE(),
        constants::ID_EXIT(),
        constants::ID_FIND(),
        constants::ID_FIRST(),
        constants::ID_FLOPPY(),
        constants::ID_FORWARD(),
        constants::ID_HARDDISK(),
        constants::ID_HELP(),
        constants::ID_HOME(),
        constants::ID_INDENT(),
        constants::ID_INDEX(),
        constants::ID_INFO(),
        constants::ID_ITALIC(),
        constants::ID_JUMP_TO(),
        constants::ID_JUSTIFY_CENTER(),
        constants::ID_JUSTIFY_FILL(),
        constants::ID_JUSTIFY_LEFT(),
        constants::ID_JUSTIFY_RIGHT(),
        constants::ID_LAST(),
        constants::ID_NETWORK(),
        constants::ID_NEW(),
        constants::ID_NO(),
        constants::ID_OK(),
        constants::ID_OPEN(),
        constants::ID_PASTE(),
        constants::ID_PREFERENCES(),
        constants::ID_PREVIEW(),
        constants::ID_PRINT(),
        constants::ID_PROPERTIES(),
        constants::ID_REDO(),
        constants::ID_REFRESH(),
        constants::ID_REMOVE(),
        constants::ID_REPLACE(),
        constants::ID_REVERT_TO_SAVED(),
        constants::ID_SAVE(),
        constants::ID_SAVEAS(),
        constants::ID_SELECT_COLOR(),
        constants::ID_SELECT_FONT(),
        constants::ID_SORT_ASCENDING(),
        constants::ID_SORT_DESCENDING(),
        constants::ID_SPELL_CHECK(),
        constants::ID_STOP(),
        constants::ID_STRIKETHROUGH(),
        constants::ID_TOP(),
        constants::ID_UNDELETE(),
        constants::ID_UNDERLINE(),
        constants::ID_UNDO(),
        constants::ID_UNINDENT(),
        constants::ID_UP(),
        constants::ID_YES(),
        constants::ID_ZOOM_100(),
        constants::ID_ZOOM_FIT(),
        constants::ID_ZOOM_IN(),
        constants::ID_ZOOM_OUT(),
    ]
}

fn closure_of(handler: Handler, ctx: *mut c_void) -> Closure {
    Closure::new(handler as *const () as *mut c_void, ctx)
}

/// Copies the file history handle out of the state without holding the borrow.
fn file_history_of(data: *mut c_void) -> Option<FileHistory> {
    if data.is_null() {
        return None;
    }
    let cell = unsafe { &*(data as *const RefCell<State>) };
    cell.try_borrow().ok().and_then(|state| state.file_history)
}

/// `MyFrame::GetLastMenuItem` without holding the state borrow.
fn last_menu_item_of(data: *mut c_void) -> Option<MenuItem> {
    if data.is_null() {
        return None;
    }
    let cell = unsafe { &*(data as *const RefCell<State>) };
    cell.try_borrow()
        .ok()
        .and_then(|state| last_menu_item(&state))
}

/// Copies the frame handle out of the state without holding the borrow.
fn frame_of(data: *mut c_void) -> Option<Frame> {
    if data.is_null() {
        return None;
    }
    let cell = unsafe { &*(data as *const RefCell<State>) };
    cell.try_borrow().ok().map(|state| state.frame)
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

/// Runs `f` with the borrowed dialog state. A shared reference is used because
/// the popup and modal loops can re-enter these handlers.
fn with_dialog(data: *mut c_void, f: impl FnOnce(&DialogState)) {
    if data.is_null() {
        return;
    }
    let state = unsafe { &*(data as *const DialogState) };
    f(state);
}
