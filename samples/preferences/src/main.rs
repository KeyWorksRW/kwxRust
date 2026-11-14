#![windows_subsystem = "windows"]

//! Port of the wxWidgets `preferences` sample
//! (`C:/rwCode/wxSamples/src/preferences/preferences.cpp`).
//!
//! Reproduces the sample's single feature: a frame whose `&File` menu opens a
//! `wxPreferencesEditor` with two pages.
//!
//! * `MyFrame` -- `Preferences sample`, `&File` menu with the stock
//!   `&Preferences` (`wxID_PREFERENCES`) and `&Quit` (`wxID_EXIT`) items, a
//!   borderless `wxPanel` showing two static texts (`Markdown syntax:` /
//!   `Spell checking:`) driven by the application settings.
//! * `PrefsPageGeneralPanel` -- the stock `Kind_General` page with the
//!   `Use Markdown syntax` and `Check spelling` check boxes.
//! * `PrefsPageTopicsPanel` -- the `Topics` page with a label, a 400x300 list
//!   box and the `Automatically build fulltext index` check box.
//! * `MyApp` -- holds the settings, owns the lazily created editor, applies the
//!   settings and refreshes the frame labels.
//!
//! Two wxWidgets facilities the C++ sample subclasses cannot be subclassed from
//! Rust, so their generated hook objects are used directly:
//!
//! * the page windows' `TransferDataToWindow` / `TransferDataFromWindow`
//!   overrides -- reproduced with a `Validator` hook object attached to each
//!   page panel, which the dialog's transfer traversal invokes exactly as it
//!   invokes the panel overrides;
//! * `wxPreferencesPage` / `wxStockPreferencesPage` -- driven by their callback
//!   setters (`set_on_create_window`, `set_on_get_icon`, ...).
//!
//! The create-window callbacks receive the notebook the editor builds as a raw
//! `wxWindow*`; `Window::from_ptr` wraps it so the page panels are created
//! directly under it, as the C++ sample does.

use std::cell::RefCell;
use std::os::raw::{c_int, c_void};
use std::rc::Rc;

use kwxffi_sys::app::App;
use kwxffi_sys::art_provider::ArtProvider;
use kwxffi_sys::box_sizer::BoxSizer;
use kwxffi_sys::check_box::CheckBox;
use kwxffi_sys::closure::Closure;
use kwxffi_sys::command_event::CommandEvent;
use kwxffi_sys::constants;
use kwxffi_sys::event::Event;
use kwxffi_sys::events;
use kwxffi_sys::flex_grid_sizer::FlexGridSizer;
use kwxffi_sys::frame::Frame;
use kwxffi_sys::list_box::ListBox;
use kwxffi_sys::menu::Menu;
use kwxffi_sys::menu_bar::MenuBar;
use kwxffi_sys::panel::Panel;
use kwxffi_sys::preferences_editor::PreferencesEditor;
use kwxffi_sys::preferences_page::PreferencesPage;
use kwxffi_sys::sizer::Sizer;
use kwxffi_sys::sizer_flags::SizerFlags;
use kwxffi_sys::static_text::StaticText;
use kwxffi_sys::stock_preferences_page::StockPreferencesPage;
use kwxffi_sys::support::{Rect, Size, NO_PARENT};
use kwxffi_sys::traits::WxWindow;
use kwxffi_sys::validator::Validator;
use kwxffi_sys::window::Window;

mod app_host;

/// `wxFrame(nullptr, wxID_ANY, "Preferences sample")`.
const FRAME_TITLE: &str = "Preferences sample";

/// `SetAppDisplayName("wxWidgets Sample")`.
const APP_DISPLAY_NAME: &str = "wxWidgets Sample";

/// `wxStockPreferencesPage::Kind_General`.
const KIND_GENERAL: i32 = 0;

/// The C callback shape expected by `wxClosure`: `(closureFun, data, event)`.
type Handler = extern "C" fn(*mut c_void, *mut c_void, *mut c_void);

/// `MySettings`.
#[derive(Copy, Clone)]
struct Settings {
    use_markdown: bool,
    spellcheck: bool,
}

impl Default for Settings {
    /// `MySettings::MySettings()` -- hardcoded initial values.
    fn default() -> Self {
        Self {
            use_markdown: true,
            spellcheck: false,
        }
    }
}

/// `MyApp` + `MyFrame` member fields.
struct State {
    frame: Frame,
    text_markdown: StaticText,
    text_spellcheck: StaticText,

    /// `MyApp::m_settings`.
    settings: Settings,
    /// `PrefsPageGeneralPanel::m_settingsCurrent`.
    current: Settings,

    /// `MyApp::m_prefEditor`, created on the first `ShowPreferencesEditor`.
    editor: Option<PreferencesEditor>,
    general_page: Option<StockPreferencesPage>,
    topics_page: Option<PreferencesPage>,

    /// The live page controls (replaced every time the editor shows a page).
    general_use_markdown: Option<CheckBox>,
    general_spellcheck: Option<CheckBox>,
    topics_fulltext: Option<CheckBox>,

    /// `Closure` values that must keep firing for the lifetime of the widget.
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

    // SetAppDisplayName("wxWidgets Sample") -- it becomes the preferences
    // dialog's title on the generic implementation.
    if let Some(app) = App::get_app() {
        app.set_app_display_name(APP_DISPLAY_NAME);
    }

    // m_frame = new MyFrame(); m_frame->Show(true);
    let state = build_frame();
    let frame = state.borrow().frame;
    app_host::set_top_window(&frame);
    frame.show();

    let exit_code = app_host::main_loop();

    // ~MyApp(): the editor owns the pages, so releasing it releases them too.
    let editor = state.borrow_mut().editor.take();
    if let Some(editor) = editor {
        editor.delete();
    }

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

    // wxMenu *fileMenu = new wxMenu;
    let file_menu = Menu::new("");
    // fileMenu->Append(wxID_PREFERENCES);  -- empty text selects the stock label
    file_menu.append(constants::ID_PREFERENCES(), "", "", false);
    // fileMenu->Append(wxID_EXIT);
    file_menu.append(constants::ID_EXIT(), "", "", false);

    let menu_bar = MenuBar::new();
    menu_bar.append(&file_menu, "&File");
    frame.set_menu_bar(&menu_bar);

    let panel_style = constants::TAB_TRAVERSAL() | constants::BORDER_NONE();
    let panel = Panel::new_full(&frame, constants::ID_ANY(), Rect::default(), panel_style);

    let text_markdown = StaticText::new_full(&panel, constants::ID_ANY(), "", Rect::default(), 0);
    let text_spellcheck = StaticText::new_full(&panel, constants::ID_ANY(), "", Rect::default(), 0);

    // wxFlexGridSizer(2, wxSize(5, 5)) -- two columns, 5x5 gaps.
    let sizer = FlexGridSizer::new(0, 2, 5, 5);

    let label_markdown = StaticText::new_full(
        &panel,
        constants::ID_ANY(),
        "Markdown syntax:",
        Rect::default(),
        0,
    );
    let label_spellcheck = StaticText::new_full(
        &panel,
        constants::ID_ANY(),
        "Spell checking:",
        Rect::default(),
        0,
    );

    // wxSizerFlags().Center().Right()
    add_with_flags(&sizer, &label_markdown, centered_right_flags());
    // wxSizerFlags().Center()
    add_with_flags(&sizer, &text_markdown, centered_flags());
    add_with_flags(&sizer, &label_spellcheck, centered_right_flags());
    add_with_flags(&sizer, &text_spellcheck, centered_flags());

    panel.set_sizer(&sizer, false);

    let settings = Settings::default();
    let state = State {
        frame,
        text_markdown,
        text_spellcheck,
        settings,
        current: settings,
        editor: None,
        general_page: None,
        topics_page: None,
        general_use_markdown: None,
        general_spellcheck: None,
        topics_fulltext: None,
        closures: Vec::new(),
    };
    let state = Rc::new(RefCell::new(state));
    connect_events(&state);

    // UpdateSettings() -- show the initial values.
    let mut state_ref = state.borrow_mut();
    let settings = state_ref.settings;
    apply_settings(&mut state_ref, settings);
    drop(state_ref);

    state
}

// ---------------------------------------------------------------------------
// Event table
// ---------------------------------------------------------------------------

fn connect_events(state: &Rc<RefCell<State>>) {
    let ctx = state_ctx(state);
    let frame = state.borrow().frame;

    let pref = closure_of(handle_pref, ctx);
    let exit = closure_of(handle_exit, ctx);
    let close = closure_of(handle_close, ctx);

    let menu_event = events::EVT_MENU();

    // Bind(wxEVT_MENU, &MyFrame::OnPref, this, wxID_PREFERENCES);
    let pref_id = constants::ID_PREFERENCES();
    frame.bind(pref_id, pref_id, menu_event, &pref);

    // Bind(wxEVT_MENU, &MyFrame::OnExit, this, wxID_EXIT);
    let exit_id = constants::ID_EXIT();
    frame.bind(exit_id, exit_id, menu_event, &exit);

    // Bind(wxEVT_CLOSE_WINDOW, &MyFrame::OnClose, this);
    let any = constants::ID_ANY();
    frame.bind(any, any, events::EVT_CLOSE_WINDOW(), &close);

    state.borrow_mut().closures = vec![pref, exit, close];
}

// ---------------------------------------------------------------------------
// MyFrame handlers
// ---------------------------------------------------------------------------

/// `MyFrame::OnPref` -- `wxGetApp().ShowPreferencesEditor(this)`.
extern "C" fn handle_pref(_fun: *mut c_void, data: *mut c_void, event: *mut c_void) {
    // `wxClosure` also makes a final call with a null event when it is
    // destroyed; that call must not re-open the editor.
    if event.is_null() {
        return;
    }
    // Build (or reuse) the editor without holding the state borrow: the dialog
    // is modal on MSW and its own events need the same state.
    let pair = with_state(data, |state| (ensure_editor(state, data), state.frame));
    if let Some((editor, frame)) = pair {
        editor.show(&frame);
    }
}

/// `MyFrame::OnExit` -- `Close()`.
extern "C" fn handle_exit(_fun: *mut c_void, data: *mut c_void, event: *mut c_void) {
    // `wxClosure` also makes a final call with a null event when it is
    // destroyed; that call must not touch the frame.
    if event.is_null() {
        return;
    }
    // `Close()` dispatches the close event synchronously, so the borrow must be
    // released first: the close handler reads the same state.
    let frame = with_state(data, |state| state.frame);
    if let Some(frame) = frame {
        frame.close(false);
    }
}

/// `MyFrame::OnClose` -- `DismissPreferencesEditor(); e.Skip();`
extern "C" fn handle_close(_fun: *mut c_void, data: *mut c_void, event: *mut c_void) {
    // `wxClosure` also makes a final call with a null event when it is
    // destroyed; that call must not touch the editor.
    if event.is_null() {
        return;
    }

    let editor = with_state(data, |state| state.editor).flatten();
    if let Some(editor) = editor {
        editor.dismiss();
    }

    let event = unsafe { Event::from_ptr(event) };
    event.skip(true);
}

/// `PrefsPageGeneralPanel::ChangedUseMarkdown`.
extern "C" fn handle_use_markdown(_fun: *mut c_void, data: *mut c_void, event: *mut c_void) {
    if event.is_null() {
        return;
    }
    let checked = unsafe { CommandEvent::from_ptr(event) }.is_checked();

    with_state(data, |state| {
        state.current.use_markdown = checked;
        update_settings_if_necessary(state);
    });
}

/// `PrefsPageGeneralPanel::ChangedSpellcheck`.
extern "C" fn handle_spellcheck(_fun: *mut c_void, data: *mut c_void, event: *mut c_void) {
    if event.is_null() {
        return;
    }
    let checked = unsafe { CommandEvent::from_ptr(event) }.is_checked();

    with_state(data, |state| {
        state.current.spellcheck = checked;
        update_settings_if_necessary(state);
    });
}

/// `PrefsPageTopicsPanel::ChangedFulltext` -- the C++ body is empty.
extern "C" fn handle_fulltext(_fun: *mut c_void, _data: *mut c_void, _event: *mut c_void) {}

// ---------------------------------------------------------------------------
// MyApp
// ---------------------------------------------------------------------------

/// `MyApp::ShowPreferencesEditor`.
fn ensure_editor(state: &mut State, ctx: *mut c_void) -> PreferencesEditor {
    if let Some(editor) = state.editor {
        return editor;
    }

    // m_prefEditor.reset(new wxPreferencesEditor); -- the default (empty)
    // title makes the implementation build "<display name> Preferences".
    let editor = PreferencesEditor::new("");

    // m_prefEditor->AddPage(new PrefsPageGeneral());
    // The stock page carries the state pointer itself; its create-window
    // callback receives it as its first argument.
    let general = StockPreferencesPage::create_with_state(KIND_GENERAL, ctx);
    general.set_on_create_window(general_page_create_window as *const () as *mut c_void);
    state.general_page = Some(general);
    editor.add_stock_page(&general);

    // m_prefEditor->AddPage(new PrefsPageTopics());
    let topics = PreferencesPage::new(ctx);
    topics.set_on_get_name(topics_page_get_name as *const () as *mut c_void);
    topics.set_on_create_window(topics_page_create_window as *const () as *mut c_void);
    topics.set_on_get_icon(topics_page_get_icon as *const () as *mut c_void);
    state.topics_page = Some(topics);
    editor.add_page(&topics);

    state.editor = Some(editor);
    editor
}

/// `MyApp::UpdateSettings` + `MyFrame::UpdateSettings`.
fn apply_settings(state: &mut State, settings: Settings) {
    state.settings = settings;

    state
        .text_markdown
        .set_label(if settings.use_markdown { "yes" } else { "no" });
    state
        .text_spellcheck
        .set_label(if settings.spellcheck { "on" } else { "off" });
}

/// `PrefsPageGeneralPanel::UpdateSettingsIfNecessary`.
fn update_settings_if_necessary(state: &mut State) {
    if PreferencesEditor::should_apply_changes_immediately() {
        let current = state.current;
        apply_settings(state, current);
    }
}

// ---------------------------------------------------------------------------
// Preferences pages
// ---------------------------------------------------------------------------

/// `PrefsPageTopics::GetName` -- publishes "Topics" from the layout callback.
unsafe extern "C" fn topics_page_get_name(ctx: *mut c_void) {
    with_state(ctx, |state| {
        if let Some(page) = state.topics_page {
            page.set_on_get_name_result("Topics");
        }
    });
}

/// `PrefsPageTopics::GetIcon` --
/// `wxArtProvider::GetBitmapBundle(wxART_HELP, wxART_TOOLBAR)`.
///
/// The published bundle is allocated with `new` and deleted by the page, which
/// is exactly what `wxArtProvider_GetBitmapBundle` returns.
unsafe extern "C" fn topics_page_get_icon(ctx: *mut c_void) {
    let size = Size {
        width: -1,
        height: -1,
    };
    let Some(bundle) = ArtProvider::get_bitmap_bundle("wxART_HELP", "wxART_TOOLBAR", size) else {
        return;
    };

    with_state(ctx, |state| {
        if let Some(page) = state.topics_page {
            page.set_on_get_icon_result(&bundle);
        }
    });
}

/// `PrefsPageGeneral::CreateWindow` -- `new PrefsPageGeneralPanel(parent)`.
unsafe extern "C" fn general_page_create_window(ctx: *mut c_void, parent: *mut c_void) {
    // The editor hands out the notebook it creates as a raw `wxWindow*`.
    let parent = unsafe { Window::from_ptr(parent) };
    with_state(ctx, |state| {
        let panel = build_general_page(state, &parent, ctx);
        if let Some(page) = state.general_page {
            page.set_on_create_window_result(&panel);
        }
    });
}

/// `PrefsPageTopics::CreateWindow` -- `new PrefsPageTopicsPanel(parent)`.
unsafe extern "C" fn topics_page_create_window(ctx: *mut c_void, parent: *mut c_void) {
    let parent = unsafe { Window::from_ptr(parent) };
    with_state(ctx, |state| {
        let panel = build_topics_page(state, &parent, ctx);
        if let Some(page) = state.topics_page {
            page.set_on_create_window_result(&panel);
        }
    });
}

/// `PrefsPageGeneralPanel::PrefsPageGeneralPanel`.
fn build_general_page(state: &mut State, parent: &impl WxWindow, ctx: *mut c_void) -> Panel {
    let panel = Panel::new_full(
        parent,
        constants::ID_ANY(),
        Rect::default(),
        constants::TAB_TRAVERSAL() | constants::BORDER_NONE(),
    );

    let use_markdown = CheckBox::new_full(
        &panel,
        constants::ID_ANY(),
        "Use Markdown syntax",
        Rect::default(),
        0,
    );
    let spellcheck = CheckBox::new_full(
        &panel,
        constants::ID_ANY(),
        "Check spelling",
        Rect::default(),
        0,
    );

    // wxBoxSizer(wxVERTICAL); Add(..., wxSizerFlags().Border()) twice.
    let sizer = BoxSizer::new(constants::VERTICAL());
    add_with_flags(&sizer, &use_markdown, border_flags());
    add_with_flags(&sizer, &spellcheck, border_flags());
    panel.set_sizer_and_fit(&sizer, false);

    let use_markdown_closure = closure_of(handle_use_markdown, ctx);
    let spellcheck_closure = closure_of(handle_spellcheck, ctx);
    let any = constants::ID_ANY();
    let checkbox_event = events::EVT_CHECKBOX();
    use_markdown.bind(any, any, checkbox_event, &use_markdown_closure);
    spellcheck.bind(any, any, checkbox_event, &spellcheck_closure);

    state.closures.push(use_markdown_closure);
    state.closures.push(spellcheck_closure);

    // TransferDataToWindow() / TransferDataFromWindow() overrides.
    attach_general_transfer_hooks(&panel, ctx);

    state.general_use_markdown = Some(use_markdown);
    state.general_spellcheck = Some(spellcheck);

    panel
}

/// `PrefsPageTopicsPanel::PrefsPageTopicsPanel`.
fn build_topics_page(state: &mut State, parent: &impl WxWindow, ctx: *mut c_void) -> Panel {
    let panel = Panel::new_full(
        parent,
        constants::ID_ANY(),
        Rect::default(),
        constants::TAB_TRAVERSAL() | constants::BORDER_NONE(),
    );

    let label = StaticText::new_full(
        &panel,
        constants::ID_ANY(),
        "Search in these topics:",
        Rect::default(),
        0,
    );

    // wxListBox(this, wxID_ANY) -- never populated.
    let list_box = ListBox::new_full(
        &panel,
        constants::ID_ANY(),
        Rect::default(),
        0,
        std::ptr::null_mut(),
        0,
    );
    // box->SetMinSize(wxSize(400, 300));
    list_box.set_min_size(Size {
        width: 400,
        height: 300,
    });

    let fulltext = CheckBox::new_full(
        &panel,
        constants::ID_ANY(),
        "Automatically build fulltext index",
        Rect::default(),
        0,
    );

    let sizer = BoxSizer::new(constants::VERTICAL());
    add_with_flags(&sizer, &label, border_flags());
    // wxSizerFlags(1).Border().Expand()
    add_with_flags(&sizer, &list_box, border_expand_flags(1));
    add_with_flags(&sizer, &fulltext, border_flags());
    panel.set_sizer_and_fit(&sizer, false);

    // The C++ sample binds the handler only when changes apply immediately.
    if PreferencesEditor::should_apply_changes_immediately() {
        let closure = closure_of(handle_fulltext, ctx);
        let any = constants::ID_ANY();
        fulltext.bind(any, any, events::EVT_CHECKBOX(), &closure);
        state.closures.push(closure);
    }

    attach_topics_transfer_hooks(&panel, ctx);

    state.topics_fulltext = Some(fulltext);

    panel
}

// ---------------------------------------------------------------------------
// TransferData{To,From}Window overrides
//
// `wxWindowBase::TransferDataToWindow()` / `TransferDataFromWindow()` walk the
// dialog's children and call each child's `GetValidator()->TransferToWindow()`
// / `TransferFromWindow()`. A wxValidator therefore reaches the same points in
// the dialog lifecycle as the C++ sample's panel overrides.
//
// `Validator::new()` creates a plain, callback-less wxValidator;
// `Validator::new_with_state()` creates the `kwxValidator` hook object that
// carries the callbacks.
// ---------------------------------------------------------------------------

fn attach_general_transfer_hooks(panel: &Panel, ctx: *mut c_void) {
    let validator = make_general_validator(ctx);
    panel.set_validator(&validator);
    // SetValidator() keeps a clone; the original is ours to release.
    validator.delete();
}

fn attach_topics_transfer_hooks(panel: &Panel, ctx: *mut c_void) {
    let validator = make_topics_validator(ctx);
    panel.set_validator(&validator);
    validator.delete();
}

/// Creates a `kwxValidator` carrying the general page's transfer callbacks.
fn make_general_validator(ctx: *mut c_void) -> Validator {
    let validator = Validator::new_with_state(ctx);
    validator.set_on_clone(clone_general_validator as *const () as *mut c_void);
    validator.set_on_transfer_to_window(general_transfer_to_window as *const () as *mut c_void);
    validator.set_on_transfer_from_window(general_transfer_from_window as *const () as *mut c_void);
    validator
}

/// Creates a `kwxValidator` carrying the topics page's transfer callbacks.
fn make_topics_validator(ctx: *mut c_void) -> Validator {
    let validator = Validator::new_with_state(ctx);
    validator.set_on_clone(clone_topics_validator as *const () as *mut c_void);
    validator.set_on_transfer_to_window(topics_transfer_to_window as *const () as *mut c_void);
    validator.set_on_transfer_from_window(topics_transfer_from_window as *const () as *mut c_void);
    validator
}

/// `kwxValidator::Clone()` -- wxWidgets copies validators by cloning.
unsafe extern "C" fn clone_general_validator(ctx: *mut c_void) -> *mut c_void {
    make_general_validator(ctx).as_ptr()
}

unsafe extern "C" fn clone_topics_validator(ctx: *mut c_void) -> *mut c_void {
    make_topics_validator(ctx).as_ptr()
}

/// `PrefsPageGeneralPanel::TransferDataToWindow`.
unsafe extern "C" fn general_transfer_to_window(ctx: *mut c_void) -> c_int {
    with_state(ctx, |state| {
        state.current = state.settings;
        let current = state.current;
        if let Some(checkbox) = state.general_use_markdown {
            checkbox.set_value(current.use_markdown);
        }
        if let Some(checkbox) = state.general_spellcheck {
            checkbox.set_value(current.spellcheck);
        }
    });

    1
}

/// `PrefsPageGeneralPanel::TransferDataFromWindow`.
unsafe extern "C" fn general_transfer_from_window(ctx: *mut c_void) -> c_int {
    with_state(ctx, |state| {
        let current = state.current;
        apply_settings(state, current);
    });

    1
}

/// `PrefsPageTopicsPanel::TransferDataToWindow`.
unsafe extern "C" fn topics_transfer_to_window(ctx: *mut c_void) -> c_int {
    with_state(ctx, |state| {
        if let Some(checkbox) = state.topics_fulltext {
            checkbox.set_value(true);
        }
    });

    1
}

/// `PrefsPageTopicsPanel::TransferDataFromWindow` -- calls the empty
/// `ChangedFulltext(dummy)` handler.
unsafe extern "C" fn topics_transfer_from_window(ctx: *mut c_void) -> c_int {
    // wxCommandEvent dummy; ChangedFulltext(dummy);
    handle_fulltext(std::ptr::null_mut(), ctx, std::ptr::null_mut());
    1
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn closure_of(handler: Handler, ctx: *mut c_void) -> Closure {
    Closure::new(handler as *const () as *mut c_void, ctx)
}

/// `sizer->Add(window, flags)` -- `wxSizer::Add(window, const wxSizerFlags&)`
/// copies the flags, so `flags` is released here.
fn add_with_flags(sizer: &Sizer, window: &impl WxWindow, flags: SizerFlags) {
    sizer.add_window_flags(window, &flags);
    flags.delete();
}

/// `wxSizerFlags().Center().Right()`.
fn centered_right_flags() -> SizerFlags {
    let flags = SizerFlags::new(0);
    flags.center();
    flags.right();
    flags
}

/// `wxSizerFlags().Center()`.
fn centered_flags() -> SizerFlags {
    let flags = SizerFlags::new(0);
    flags.center();
    flags
}

/// `wxSizerFlags().Border()` -- `Border()` uses the DPI-scaled default border.
fn border_flags() -> SizerFlags {
    let flags = SizerFlags::new(0);
    flags.border(constants::ALL());
    flags
}

/// `wxSizerFlags(proportion).Border().Expand()`.
fn border_expand_flags(proportion: i32) -> SizerFlags {
    let flags = SizerFlags::new(proportion);
    flags.border(constants::ALL());
    flags.expand();
    flags
}

/// Hands the state to a C callback. The `Rc` is intentionally leaked so that
/// every connection keeps a valid strong reference for the process lifetime.
fn state_ctx(state: &Rc<RefCell<State>>) -> *mut c_void {
    Rc::into_raw(Rc::clone(state)) as *mut c_void
}

/// Runs `f` with the borrowed state. Never panics: if the state is already
/// borrowed the callback is skipped.
fn with_state<R>(data: *mut c_void, f: impl FnOnce(&mut State) -> R) -> Option<R> {
    if data.is_null() {
        return None;
    }
    let cell = unsafe { &*(data as *const RefCell<State>) };
    match cell.try_borrow_mut() {
        Ok(mut state) => Some(f(&mut state)),
        Err(_) => None,
    }
}
