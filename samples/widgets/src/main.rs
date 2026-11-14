#![windows_subsystem = "windows"]

//! Port of the wxWidgets `widgets` sample
//! (`C:/rwCode/wxSamples/src/widgets`).
//!
//! Reproduces the sample's single frame:
//!
//! * `WidgetsFrame` -- `wxMSW widgets demo`, the sample icon, the `&Widget`,
//!   `&Text` and `&Page` menus, a `wxTreebook` of control pages created lazily
//!   from the page registry, and the `&Log window` static box holding the log
//!   list box plus the `Clear &log` / `E&xit` buttons.
//!
//! Conversion notes:
//!
//! * The C++ sample implements every page as a `WidgetsPage` subclass with a
//!   `CreateContent()` / `RecreateWidget()` pair. Rust has no subclassing, so
//!   each page is a plain `wxScrolledWindow` plus a `PageWidgets` value holding
//!   the handles the page's handlers need (`GetWidget()` in the original).
//! * `wxLog`'s `LboxLogger` target (a `wxLog` subclass) cannot be subclassed
//!   from Rust; every `wxLogMessage()` is routed through `log_msg()`, which
//!   appends the same text to the log list box.
//! * Event-table entries become `Closure` + `bind`. Menu ranges
//!   (`EVT_MENU_RANGE`, `EVT_UPDATE_UI_RANGE`) have no generated event type, so
//!   each id in the range is bound individually -- same dispatch, same order.
//! * `wxPersistentRegisterAndRestore` and the XDG `wxFileConfig` migration in
//!   `WidgetsApp`'s constructor have no binding and are not reproduced.
//!
//! Known gaps are listed in the port report; nothing here emulates a missing
//! facility.

use std::cell::RefCell;
use std::ffi::CString;
use std::os::raw::{c_char, c_void};
use std::rc::Rc;

use kwxffi_sys::activity_indicator::ActivityIndicator;
use kwxffi_sys::bitmap_bundle::BitmapBundle;
use kwxffi_sys::bitmap_bundle_array::BitmapBundleArray;
use kwxffi_sys::box_sizer::BoxSizer;
use kwxffi_sys::button::Button;
use kwxffi_sys::check_box::CheckBox;
use kwxffi_sys::choice::Choice;
use kwxffi_sys::closure::Closure;
use kwxffi_sys::command_event::CommandEvent;
use kwxffi_sys::constants;
use kwxffi_sys::context_menu_event::ContextMenuEvent;
use kwxffi_sys::event::Event;
use kwxffi_sys::events;
use kwxffi_sys::frame::Frame;
use kwxffi_sys::freefuncs;
use kwxffi_sys::gauge::Gauge;
use kwxffi_sys::icon::Icon;
use kwxffi_sys::list_box::ListBox;
use kwxffi_sys::menu::Menu;
use kwxffi_sys::menu_bar::MenuBar;
use kwxffi_sys::panel::Panel;
use kwxffi_sys::scrolled_window::ScrolledWindow;
use kwxffi_sys::sizer::Sizer;
use kwxffi_sys::sizer_flags::SizerFlags;
use kwxffi_sys::static_box::StaticBox;
use kwxffi_sys::static_box_sizer::StaticBoxSizer;
use kwxffi_sys::static_text::StaticText;
use kwxffi_sys::support::{Rect, Size, NO_PARENT};
use kwxffi_sys::text_ctrl::TextCtrl;
use kwxffi_sys::text_entry_dialog::TextEntryDialog;
use kwxffi_sys::timer::Timer;
use kwxffi_sys::traits::{WxSizer, WxWindow};
use kwxffi_sys::treebook::Treebook;
use kwxffi_sys::update_ui_event::UpdateUIEvent;

mod app_host;

/// `title = "wxMSW" + " widgets demo"` (`__WXMSW__` build).
const FRAME_TITLE: &str = "wxMSW widgets demo";

/// Directory holding the sample's own assets (`data/` next to `Cargo.toml`).
const ASSET_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/data/");

/// The C callback shape expected by `wxClosure`: `(closureFun, data, event)`.
type Handler = extern "C" fn(*mut c_void, *mut c_void, *mut c_void);

// The generated `Wx*` traits are empty markers, so a `&impl WxWindow` argument
// exposes no methods. These two macros expand the `Window`/`EvtHandler` calls
// on the concrete control type instead (reached through its `Deref` chain).

/// `WidgetsPage::SetUpWidget()` body for one window.
macro_rules! apply_attrs {
    ($window:expr, $attrs:expr) => {{
        let w = $window;
        let a = $attrs;
        w.set_tool_tip(a.tooltip);
        w.enable(a.enabled);
        if a.show {
            w.show();
        } else {
            w.hide();
        }
        w.set_window_variant(a.variant);
        if a.dir != constants::Layout_Default() {
            w.set_layout_direction(a.dir);
        }
    }};
}

/// `WidgetsApp::ConnectToWidgetEvents()` for one widget.
macro_rules! connect_events {
    ($window:expr, $ctx:expr, $closures:expr) => {{
        let w = $window;
        let class = w
            .get_class_info()
            .map(|info| info.get_class_name_ex())
            .unwrap_or_default();
        let class: &'static str = Box::leak(class.into_boxed_str());
        let hook = Box::into_raw(Box::new(HookCtx { state: $ctx, class })) as *mut c_void;
        let any = constants::ID_ANY();
        for (type_, handler) in [
            (events::EVT_SET_FOCUS(), handle_focus as Handler),
            (events::EVT_KILL_FOCUS(), handle_focus),
            (events::EVT_ENTER_WINDOW(), handle_enter),
            (events::EVT_LEAVE_WINDOW(), handle_leave),
            (events::EVT_CONTEXT_MENU(), handle_context_menu),
        ] {
            let closure = Closure::new(handler as *const () as *mut c_void, hook);
            w.bind(any, any, type_, &closure);
            $closures.push(closure);
        }
    }};
}

// ---------------------------------------------------------------------------
// Control ids (widgets.cpp lines 68-121, `wxUSE_TOOLTIPS` on)
// ---------------------------------------------------------------------------

const WIDGETS_CLEAR_LOG: i32 = 100;
const WIDGETS_QUIT: i32 = 101;
const WIDGETS_BOOKCTRL: i32 = 102;

const WIDGETS_SET_TOOLTIP: i32 = 103;
const WIDGETS_SET_FG_COLOUR: i32 = 104;
const WIDGETS_SET_BG_COLOUR: i32 = 105;
const WIDGETS_SET_PAGE_BG: i32 = 106;
const WIDGETS_SET_FONT: i32 = 107;
const WIDGETS_ENABLE: i32 = 108;
const WIDGETS_SHOW: i32 = 109;

const WIDGETS_BORDER_NONE: i32 = 110;
const WIDGETS_BORDER_STATIC: i32 = 111;
const WIDGETS_BORDER_SIMPLE: i32 = 112;
const WIDGETS_BORDER_RAISED: i32 = 113;
const WIDGETS_BORDER_SUNKEN: i32 = 114;
const WIDGETS_BORDER_DOUBLE: i32 = 115;
const WIDGETS_BORDER_DEFAULT: i32 = 116;

const WIDGETS_VARIANT_NORMAL: i32 = 117;
const WIDGETS_VARIANT_SMALL: i32 = 118;
const WIDGETS_VARIANT_MINI: i32 = 119;
const WIDGETS_VARIANT_LARGE: i32 = 120;

const WIDGETS_LAYOUT_DIRECTION: i32 = 121;
const WIDGETS_GLOBAL_BUSY_CURSOR: i32 = 122;
#[allow(dead_code)]
const WIDGETS_BUSY_CURSOR: i32 = 123;

const WIDGETS_GO_TO_PAGE: i32 = 124;

const TEXT_ENTRY_DISABLE_AUTOCOMPLETE: i32 = 225;
const TEXT_ENTRY_AUTOCOMPLETE_FIXED: i32 = 226;
const TEXT_ENTRY_AUTOCOMPLETE_FILENAMES: i32 = 227;
const TEXT_ENTRY_AUTOCOMPLETE_DIRECTORIES: i32 = 228;
#[allow(dead_code)]
const TEXT_ENTRY_AUTOCOMPLETE_CUSTOM: i32 = 229;
#[allow(dead_code)]
const TEXT_ENTRY_AUTOCOMPLETE_KEY_LENGTH: i32 = 230;
const TEXT_ENTRY_SET_HINT: i32 = 231;

#[allow(dead_code)]
const THEME_LIGHT: i32 = 233;
#[allow(dead_code)]
const THEME_DARK: i32 = 234;
#[allow(dead_code)]
const THEME_SYSTEM: i32 = 235;

// ---------------------------------------------------------------------------
// Page ids: every page .cpp restarts its enum at `wxID_HIGHEST`, and the ids
// are only meaningful inside that page's own handlers, so each page keeps the
// C++ relative ids and binds them on its own page window.
// ---------------------------------------------------------------------------

fn page_id(offset: i32) -> i32 {
    constants::ID_HIGHEST() + offset
}

// ---------------------------------------------------------------------------
// Page registry
// ---------------------------------------------------------------------------

// Category bits (`widgets.h` lines 67-78).
const CAT_NATIVE: i32 = 1 << 0;
const CAT_WITH_ITEMS: i32 = 1 << 4;
const CAT_ALL: i32 = 1 << 7;

/// `const wxChar *WidgetsCategories[MAX_PAGES]`.
const CATEGORIES: [&str; 8] = [
    "Native",
    "Generic",
    "Pickers",
    "Comboboxes",
    "With items",
    "Editable",
    "Books",
    "All controls",
];

/// The page kinds this port implements. Pages the port does not implement yet
/// are absent from `PAGE_DEFS` and reported as gaps.
#[derive(Copy, Clone, PartialEq, Eq)]
enum Kind {
    ActivityIndicator,
    Choice,
    Gauge,
}

/// `WidgetsPageInfo(label, categories)` plus the page icon.
struct PageDef {
    label: &'static str,
    /// `ALL_CTRLS | categories`.
    categories: i32,
    icon: &'static str,
    kind: Kind,
}

/// The registry, ordered by label exactly as `WidgetsPage::ms_widgetPages`
/// insertion-sorts it with `wxStrcmp`.
static PAGE_DEFS: &[PageDef] = &[
    PageDef {
        label: "ActivityIndicator",
        categories: CAT_ALL | CAT_NATIVE,
        icon: "activityindicator",
        kind: Kind::ActivityIndicator,
    },
    PageDef {
        label: "Choice",
        categories: CAT_ALL | CAT_NATIVE | CAT_WITH_ITEMS,
        icon: "choice",
        kind: Kind::Choice,
    },
    PageDef {
        label: "Gauge",
        categories: CAT_ALL | CAT_NATIVE,
        icon: "gauge",
        kind: Kind::Gauge,
    },
];

// ---------------------------------------------------------------------------
// State
// ---------------------------------------------------------------------------

/// `WidgetAttributes` (`widgets.h` lines 87-118).
#[derive(Copy, Clone)]
struct Attrs {
    tooltip: &'static str,
    enabled: bool,
    show: bool,
    /// `wxLayoutDirection`; `wxLayout_Default` until toggled.
    dir: i32,
    variant: i32,
    /// `m_defaultFlags` -- only the border flags are ever changed.
    default_flags: i32,
}

impl Default for Attrs {
    fn default() -> Self {
        Self {
            tooltip: "This is a tooltip",
            enabled: true,
            show: true,
            dir: constants::Layout_Default(),
            variant: constants::WINDOW_VARIANT_NORMAL(),
            default_flags: constants::BORDER_DEFAULT(),
        }
    }
}

/// The handles a page's handlers need; stands in for the page subclass's
/// member fields and its `GetWidgets()`.
enum PageWidgets {
    None,
    ActivityIndicator {
        indicator: ActivityIndicator,
        status: StaticText,
    },
    Choice {
        choice: Choice,
        sort: CheckBox,
        add_text: TextCtrl,
        change_text: TextCtrl,
        delete_text: TextCtrl,
    },
    Gauge {
        gauge: Gauge,
        timer: Timer,
        /// The progress timer drives either the progress or the indeterminate
        /// simulation, as the sample's two timers do.
        indeterminate: bool,
        value_text: TextCtrl,
        range_text: TextCtrl,
        cur_value_text: TextCtrl,
        progress_btn: Button,
        indeterminate_btn: Button,
    },
}

impl PageWidgets {
    /// `WidgetsPage::SetUpWidget()` for this page's widgets.
    fn apply_attrs(&self, attrs: &Attrs) {
        match self {
            PageWidgets::None => {}
            PageWidgets::ActivityIndicator { indicator, status } => {
                apply_attrs!(indicator, attrs);
                apply_attrs!(status, attrs);
            }
            PageWidgets::Choice {
                choice,
                sort,
                add_text,
                change_text,
                delete_text,
            } => {
                apply_attrs!(choice, attrs);
                apply_attrs!(sort, attrs);
                apply_attrs!(add_text, attrs);
                apply_attrs!(change_text, attrs);
                apply_attrs!(delete_text, attrs);
            }
            PageWidgets::Gauge {
                gauge,
                value_text,
                range_text,
                cur_value_text,
                progress_btn,
                indeterminate_btn,
                ..
            } => {
                apply_attrs!(gauge, attrs);
                apply_attrs!(value_text, attrs);
                apply_attrs!(range_text, attrs);
                apply_attrs!(cur_value_text, attrs);
                apply_attrs!(progress_btn, attrs);
                apply_attrs!(indeterminate_btn, attrs);
            }
        }
    }
}

/// One instantiated page: the book page window plus its lazy content.
struct Page {
    kind: Kind,
    label: &'static str,
    /// The `wxScrolledWindow` added to the book.
    panel: ScrolledWindow,
    /// `CreateContent()` has run.
    created: bool,
    widgets: PageWidgets,
}

/// `WidgetsFrame`'s member fields.
struct State {
    frame: Frame,
    #[allow(dead_code)]
    panel: Panel,
    book: Treebook,
    lbox_log: ListBox,
    attrs: Attrs,
    /// Page instances in book-node order; `node_page[i]` maps a book node to
    /// its index here (`None` for the category parent nodes).
    pages: Vec<Page>,
    node_page: Vec<Option<usize>>,
    /// Currently selected page (`WidgetsFrame::CurrentPage()`).
    current: Option<usize>,
    /// `Closure` values that must keep firing for the lifetime of the frame.
    closures: Vec<Closure>,
    /// `State` handed to every callback (a leaked `Rc` clone).
    ctx: *mut c_void,
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

fn main() {
    std::process::exit(run());
}

fn run() -> i32 {
    // WidgetsApp::OnInit(): wxEntryStart() + the hidden app's OnInit().
    if app_host::initialize() == 0 {
        return 1;
    }

    let state = build_frame();
    {
        let mut state_ref = state.borrow_mut();
        init_book(&mut state_ref);
    }
    let frame = state.borrow().frame;
    app_host::set_top_window(&frame);
    frame.show();

    // Select the first page with contents (`USE_TREEBOOK`).
    let book = state.borrow().book;
    book.set_selection(1);

    let exit_code = app_host::main_loop();

    app_host::shutdown();
    exit_code
}

// ---------------------------------------------------------------------------
// WidgetsFrame::WidgetsFrame()
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
        icon.delete();
    }

    let menu_bar = MenuBar::new();

    // ---- &Widget ----------------------------------------------------------
    let widget_menu = Menu::new("");

    // #if wxUSE_TOOLTIPS
    widget_menu.append(WIDGETS_SET_TOOLTIP, "Set &tooltip...\tCtrl-T", "", false);
    widget_menu.append_separator();
    widget_menu.append(
        WIDGETS_SET_FG_COLOUR,
        "Set &foreground...\tCtrl-F",
        "",
        false,
    );
    widget_menu.append(
        WIDGETS_SET_BG_COLOUR,
        "Set &background...\tCtrl-B",
        "",
        false,
    );
    widget_menu.append(
        WIDGETS_SET_PAGE_BG,
        "Set &page background...\tShift-Ctrl-B",
        "",
        false,
    );
    widget_menu.append(WIDGETS_SET_FONT, "Set f&ont...\tCtrl-O", "", false);
    widget_menu.append_check_item(WIDGETS_ENABLE, "&Enable/disable\tCtrl-E", "");
    widget_menu.append_check_item(WIDGETS_SHOW, "Show/Hide", "");

    let border_menu = Menu::new("");
    border_menu.append_radio_item(WIDGETS_BORDER_DEFAULT, "De&fault\tCtrl-Shift-9", "");
    border_menu.append_radio_item(WIDGETS_BORDER_NONE, "&None\tCtrl-Shift-0", "");
    border_menu.append_radio_item(WIDGETS_BORDER_SIMPLE, "&Simple\tCtrl-Shift-1", "");
    border_menu.append_radio_item(WIDGETS_BORDER_DOUBLE, "&Double\tCtrl-Shift-2", "");
    border_menu.append_radio_item(WIDGETS_BORDER_STATIC, "Stati&c\tCtrl-Shift-3", "");
    border_menu.append_radio_item(WIDGETS_BORDER_RAISED, "&Raised\tCtrl-Shift-4", "");
    border_menu.append_radio_item(WIDGETS_BORDER_SUNKEN, "S&unken\tCtrl-Shift-5", "");
    widget_menu.append_sub_menu(&border_menu, "Set &border", "");

    let variant_menu = Menu::new("");
    variant_menu.append_radio_item(WIDGETS_VARIANT_MINI, "&Mini\tCtrl-Shift-6", "");
    variant_menu.append_radio_item(WIDGETS_VARIANT_SMALL, "&Small\tCtrl-Shift-7", "");
    variant_menu.append_radio_item(WIDGETS_VARIANT_NORMAL, "&Normal\tCtrl-Shift-8", "");
    variant_menu.append_radio_item(WIDGETS_VARIANT_LARGE, "&Large\tCtrl-Shift-9", "");
    widget_menu.append_sub_menu(&variant_menu, "Set &variant", "");

    widget_menu.append_separator();
    widget_menu.append_check_item(
        WIDGETS_LAYOUT_DIRECTION,
        "Toggle &layout direction\tCtrl-L",
        "",
    );
    widget_menu.append_separator();
    widget_menu.append_check_item(
        WIDGETS_GLOBAL_BUSY_CURSOR,
        "Toggle &global busy cursor\tCtrl-Shift-U",
        "",
    );
    widget_menu.append_check_item(WIDGETS_BUSY_CURSOR, "Toggle b&usy cursor\tCtrl-U", "");
    widget_menu.append_separator();
    widget_menu.append(constants::ID_EXIT(), "&Quit\tCtrl-Q", "", false);

    menu_bar.append(&widget_menu, "&Widget");

    // ---- &Text ------------------------------------------------------------
    let text_menu = Menu::new("");
    text_menu.append_radio_item(
        TEXT_ENTRY_DISABLE_AUTOCOMPLETE,
        "&Disable auto-completion",
        "",
    );
    text_menu.append_radio_item(
        TEXT_ENTRY_AUTOCOMPLETE_FIXED,
        "Fixed-&list auto-completion",
        "",
    );
    text_menu.append_radio_item(
        TEXT_ENTRY_AUTOCOMPLETE_FILENAMES,
        "&Files names auto-completion",
        "",
    );
    text_menu.append_radio_item(
        TEXT_ENTRY_AUTOCOMPLETE_DIRECTORIES,
        "&Directories names auto-completion",
        "",
    );
    text_menu.append_radio_item(
        TEXT_ENTRY_AUTOCOMPLETE_CUSTOM,
        "&Custom auto-completion",
        "",
    );
    text_menu.append_radio_item(
        TEXT_ENTRY_AUTOCOMPLETE_KEY_LENGTH,
        "Custom with &min length",
        "",
    );
    text_menu.append_separator();
    text_menu.append(TEXT_ENTRY_SET_HINT, "Set help &hint", "", false);
    menu_bar.append(&text_menu, "&Text");

    frame.set_menu_bar(&menu_bar);

    menu_bar.check(WIDGETS_ENABLE, true);
    menu_bar.check(WIDGETS_SHOW, true);
    menu_bar.check(WIDGETS_VARIANT_NORMAL, true);

    // ---- controls ---------------------------------------------------------
    let panel = Panel::new_full(
        &frame,
        constants::ID_ANY(),
        Rect::default(),
        constants::TAB_TRAVERSAL(),
    );

    // m_book = new WidgetsBookCtrl(m_panel, Widgets_BookCtrl,
    //                              wxDefaultPosition, FromDIP(wxSize(900, 500)),
    //                              wxBK_DEFAULT, "Widgets");
    let book = Treebook::new_full(
        &panel,
        WIDGETS_BOOKCTRL,
        Rect::new(-1, -1, 900, 500),
        constants::BK_DEFAULT(),
    );

    // sizerDown = new wxStaticBoxSizer(wxVERTICAL, m_panel, "&Log window");
    let down_box = StaticBox::new_full(
        &panel,
        constants::ID_ANY(),
        "&Log window",
        Rect::default(),
        0,
    );
    let sizer_down = StaticBoxSizer::new(&down_box, constants::VERTICAL());
    let down_parent = static_box_parent(&sizer_down);

    // m_lboxLog = new wxListBox(sizerDownBox, wxID_ANY);
    let lbox_log = ListBox::new_full(
        &down_parent,
        constants::ID_ANY(),
        Rect::default(),
        0,
        std::ptr::null_mut(),
        0,
    );
    add_window_flags(&sizer_down, &lbox_log, expand_border_prop(1));
    sizer_down.set_min_size(Size {
        width: 100,
        height: 150,
    });

    let sizer_btns = BoxSizer::new(constants::HORIZONTAL());
    let btn_clear = Button::new_full(
        &down_parent,
        WIDGETS_CLEAR_LOG,
        "Clear &log",
        Rect::default(),
        0,
    );
    add_window_plain(&sizer_btns, &btn_clear);
    sizer_btns.add_spacer(10);
    let btn_quit = Button::new_full(&down_parent, WIDGETS_QUIT, "E&xit", Rect::default(), 0);
    add_window_plain(&sizer_btns, &btn_quit);
    {
        let f = SizerFlags::new(0);
        f.border(constants::ALL());
        f.right();
        add_sizer_flags(&sizer_down, &sizer_btns, f);
    }

    // sizerTop->Add(m_book, wxSizerFlags(1).Expand().DoubleBorder(
    //                   wxALL & ~(wxTOP | wxBOTTOM)));
    let sizer_top = BoxSizer::new(constants::VERTICAL());
    {
        let f = SizerFlags::new(1);
        f.expand();
        f.double_border(constants::LEFT() | constants::RIGHT());
        add_window_flags(&sizer_top, &book, f);
    }
    sizer_top.add_spacer(5);
    {
        let f = SizerFlags::new(0);
        f.expand();
        f.double_border(constants::LEFT() | constants::RIGHT() | constants::BOTTOM());
        add_sizer_flags(&sizer_top, &sizer_down, f);
    }

    panel.set_sizer(&sizer_top, false);

    let state = State {
        frame,
        panel,
        book,
        lbox_log,
        attrs: Attrs::default(),
        pages: Vec::new(),
        node_page: Vec::new(),
        current: None,
        closures: Vec::new(),
        ctx: std::ptr::null_mut(),
    };
    let state = Rc::new(RefCell::new(state));

    // The context every callback receives. The `Rc` is intentionally leaked so
    // that each connection keeps a valid strong reference for the process
    // lifetime.
    let ctx = Rc::into_raw(Rc::clone(&state)) as *mut c_void;
    state.borrow_mut().ctx = ctx;

    connect_frame_events(&state);

    state
}

// ---------------------------------------------------------------------------
// WidgetsFrame::InitBook()
// ---------------------------------------------------------------------------

fn init_book(state: &mut State) {
    // wxVector<wxBitmapBundle> imageList; imageList.push_back(sample)
    let images = BitmapBundleArray::new();
    if let Some(bundle) = load_bundle("sample", 32, 32) {
        images.add(&bundle);
        bundle.delete();
    }
    let mut menu_pages = Menu::new("");
    let mut n_page = 0;

    // First pass: construct the pages (one instance per category, exactly as
    // the C++ per-category constructor call does) and build the Page menu.
    for (cat, _label) in CATEGORIES.iter().enumerate() {
        n_page += 1; // increase for the parent page
        let bit = 1 << cat;
        let mut defs: Vec<&PageDef> = PAGE_DEFS
            .iter()
            .filter(|def| def.categories & bit != 0)
            .collect();
        // `ms_widgetPages` is kept sorted by label with wxStrcmp.
        defs.sort_by(|a, b| a.label.cmp(b.label));

        for def in defs {
            let panel = ScrolledWindow::new_full(
                &state.book,
                constants::ID_ANY(),
                Rect::default(),
                constants::CLIP_CHILDREN() | constants::TAB_TRAVERSAL(),
            );
            state.pages.push(Page {
                kind: def.kind,
                label: def.label,
                panel,
                created: false,
                widgets: PageWidgets::None,
            });

            // The `WidgetsPage` constructor pushes its icon into the image
            // vector: one bundle per page instance, in construction order.
            if let Some(bundle) = load_bundle(def.icon, 16, 16) {
                images.add(&bundle);
                bundle.delete();
            }

            if cat == CATEGORIES.len() - 1 {
                menu_pages.append_radio_item(WIDGETS_GO_TO_PAGE + n_page, def.label, "");
            }
            n_page += 1;
        }
    }

    // Second pass: add the category parent nodes and their pages, recording
    // the book-node -> page mapping the page-changed handler needs.
    let mut node_order: Vec<Option<usize>> = Vec::new();
    let mut page_index = 0usize;
    let mut image = 1;
    for (cat, label) in CATEGORIES.iter().enumerate() {
        state.book.add_page(&NO_PARENT, *label, false, 0);
        node_order.push(None);

        let bit = 1 << cat;
        let count = PAGE_DEFS
            .iter()
            .filter(|def| def.categories & bit != 0)
            .count();
        for _ in 0..count {
            let (panel, page_label) = {
                let page = &state.pages[page_index];
                (page.panel, page.label)
            };
            image += 1;
            state.book.add_sub_page(&panel, page_label, false, image);
            node_order.push(Some(page_index));
            page_index += 1;
        }
    }
    state.node_page = node_order;

    state.book.set_images(&images);
    images.delete();

    if let Some(menu_bar) = state.frame.get_menu_bar() {
        menu_bar.append(&menu_pages, "&Page");
    }

    // ---- T&heme -----------------------------------------------------------
    let theme_menu = Menu::new("");
    theme_menu.append_radio_item(THEME_SYSTEM, "&System", "");
    theme_menu.append_radio_item(THEME_LIGHT, "&Light", "");
    theme_menu.append_radio_item(THEME_DARK, "&Dark", "");
    if let Some(menu_bar) = state.frame.get_menu_bar() {
        menu_bar.append(&theme_menu, "T&heme");
    }

    // wxTreebook: select the first page with contents and make sure the top of
    // the tree is visible.
    if state.book.get_current_page().is_none() {
        state.book.set_selection(1);
    }
    if let Some(tree) = state.book.get_tree_ctrl() {
        if let Some(size) = tree.get_best_size() {
            tree.set_min_size(Size {
                width: size.get_width() * 9 / 8,
                height: -1,
            });
        }
    }

    connect_book_events(state);

    // Force content creation for the initial selection (the C++ relies on the
    // page-changed event; doing it here as well makes the first page appear).
    on_book_selection(state);
}

// ---------------------------------------------------------------------------
// Frame event table
// ---------------------------------------------------------------------------

fn connect_frame_events(state: &Rc<RefCell<State>>) {
    let ctx = state.borrow().ctx;
    let frame = state.borrow().frame;
    let mut closures = Vec::new();

    let menu_event = events::EVT_MENU();
    let button_event = events::EVT_BUTTON();

    // Buttons in the log pane.
    let clear_log = Closure::new(handle_clear_log as *const () as *mut c_void, ctx);
    frame.bind(
        WIDGETS_CLEAR_LOG,
        WIDGETS_CLEAR_LOG,
        button_event,
        &clear_log,
    );
    closures.push(clear_log);

    let quit = Closure::new(handle_exit as *const () as *mut c_void, ctx);
    frame.bind(WIDGETS_QUIT, WIDGETS_QUIT, button_event, &quit);
    closures.push(quit);

    // Menu items: one binding per id.
    let ids: [(i32, Handler); 25] = [
        (constants::ID_EXIT(), handle_exit),
        (WIDGETS_SET_TOOLTIP, handle_set_tooltip),
        (WIDGETS_SET_FG_COLOUR, handle_set_fg),
        (WIDGETS_SET_BG_COLOUR, handle_set_bg),
        (WIDGETS_SET_PAGE_BG, handle_set_page_bg),
        (WIDGETS_SET_FONT, handle_set_font),
        (WIDGETS_ENABLE, handle_enable),
        (WIDGETS_SHOW, handle_show),
        (WIDGETS_BORDER_NONE, handle_set_border),
        (WIDGETS_BORDER_STATIC, handle_set_border),
        (WIDGETS_BORDER_SIMPLE, handle_set_border),
        (WIDGETS_BORDER_RAISED, handle_set_border),
        (WIDGETS_BORDER_SUNKEN, handle_set_border),
        (WIDGETS_BORDER_DOUBLE, handle_set_border),
        (WIDGETS_BORDER_DEFAULT, handle_set_border),
        (WIDGETS_VARIANT_NORMAL, handle_set_variant),
        (WIDGETS_VARIANT_SMALL, handle_set_variant),
        (WIDGETS_VARIANT_MINI, handle_set_variant),
        (WIDGETS_VARIANT_LARGE, handle_set_variant),
        (WIDGETS_LAYOUT_DIRECTION, handle_toggle_layout),
        (WIDGETS_GLOBAL_BUSY_CURSOR, handle_toggle_global_busy),
        (TEXT_ENTRY_DISABLE_AUTOCOMPLETE, handle_autocomplete_disable),
        (TEXT_ENTRY_AUTOCOMPLETE_FIXED, handle_autocomplete_fixed),
        (
            TEXT_ENTRY_AUTOCOMPLETE_FILENAMES,
            handle_autocomplete_filenames,
        ),
        (
            TEXT_ENTRY_AUTOCOMPLETE_DIRECTORIES,
            handle_autocomplete_directories,
        ),
    ];
    for (id, handler) in ids {
        let closure = Closure::new(handler as *const () as *mut c_void, ctx);
        frame.bind(id, id, menu_event, &closure);
        closures.push(closure);
    }

    // Page menu: Widgets_GoToPage .. +100.
    for n in 0..=100 {
        let id = WIDGETS_GO_TO_PAGE + n;
        let closure = Closure::new(handle_go_to_page as *const () as *mut c_void, ctx);
        frame.bind(id, id, menu_event, &closure);
        closures.push(closure);
    }

    // Text hint + theme.
    let hint = Closure::new(handle_set_hint as *const () as *mut c_void, ctx);
    frame.bind(TEXT_ENTRY_SET_HINT, TEXT_ENTRY_SET_HINT, menu_event, &hint);
    closures.push(hint);
    for id in [THEME_SYSTEM, THEME_LIGHT, THEME_DARK] {
        let closure = Closure::new(handle_theme as *const () as *mut c_void, ctx);
        frame.bind(id, id, menu_event, &closure);
        closures.push(closure);
    }

    state.borrow_mut().closures.extend(closures);
}

fn connect_book_events(state: &mut State) {
    let ctx = state.ctx;
    let book = state.book;

    let changed = Closure::new(handle_page_changed as *const () as *mut c_void, ctx);
    let any = constants::ID_ANY();
    book.bind(any, any, events::EVT_TREEBOOK_PAGE_CHANGED(), &changed);
    state.closures.push(changed);
}

/// `WidgetsFrame::OnPageChanged`.
extern "C" fn handle_page_changed(_fun: *mut c_void, data: *mut c_void, event: *mut c_void) {
    if event.is_null() {
        return;
    }
    let event = unsafe { CommandEvent::from_ptr(event) };
    let selection = event.get_selection();

    with_state(data, |state| {
        let index = state
            .node_page
            .get(selection.max(0) as usize)
            .copied()
            .flatten();
        state.current = index;
        if let Some(index) = index {
            ensure_content(state, index);
            if let Some(menu_bar) = state.frame.get_menu_bar() {
                menu_bar.check(WIDGETS_GO_TO_PAGE + selection, true);
            }
        }
        setup_widget(state);
    });

    event.skip(true);
}

/// `OnGoToPage`: `SetSelection(id - Widgets_GoToPage)`.
extern "C" fn handle_go_to_page(_fun: *mut c_void, data: *mut c_void, event: *mut c_void) {
    if event.is_null() {
        return;
    }
    let event = unsafe { CommandEvent::from_ptr(event) };
    let id = event.get_id();
    let book = with_state(data, |state| state.book);
    if let Some(book) = book {
        book.set_selection(id - WIDGETS_GO_TO_PAGE);
    }
}

/// Creates the content of the initially selected page.
fn on_book_selection(state: &mut State) {
    let selection = state.book.get_selection();
    let index = state
        .node_page
        .get(selection.max(0) as usize)
        .copied()
        .flatten();
    state.current = index;
    if let Some(index) = index {
        ensure_content(state, index);
    }
    setup_widget(state);
}

// ---------------------------------------------------------------------------
// Page content
// ---------------------------------------------------------------------------

type Builder = fn(
    panel: &ScrolledWindow,
    attrs: &Attrs,
    ctx: *mut c_void,
    closures: &mut Vec<Closure>,
) -> PageWidgets;

/// `CreateContent()`: builds the page's sizer and controls the first time the
/// page is shown.
fn ensure_content(state: &mut State, index: usize) {
    if index >= state.pages.len() || state.pages[index].created {
        return;
    }

    let panel = state.pages[index].panel;
    let kind = state.pages[index].kind;
    let attrs = state.attrs;
    let ctx = state.ctx;
    let mut closures = Vec::new();
    let build: Builder = match kind {
        Kind::ActivityIndicator => build_activity_indicator,
        Kind::Choice => build_choice,
        Kind::Gauge => build_gauge,
    };
    let widgets = build(&panel, &attrs, ctx, &mut closures);

    // SetScrollRate(10, 10); FitInside();
    panel.set_scroll_rate(10, 10);
    panel.fit_inside();

    state.pages[index].widgets = widgets;
    state.pages[index].created = true;
    state.closures.extend(closures);
}

/// `WidgetsPage::RecreateWidget()`: drop the old content and build it again.
fn recreate_page(state: &mut State, index: usize) {
    if index >= state.pages.len() {
        return;
    }
    // `SetSizer()` with deleteOld deletes the previous sizer and its windows.
    state.pages[index].created = false;
    state.pages[index].widgets = PageWidgets::None;
    ensure_content(state, index);
}

/// `WidgetsPage::SetUpWidget()` -- applies the shared attributes.
fn setup_widget(state: &mut State) {
    let Some(index) = state.current else {
        return;
    };
    if index >= state.pages.len() || !state.pages[index].created {
        return;
    }
    let attrs = state.attrs;
    state.pages[index].widgets.apply_attrs(&attrs);
}

// ---------------------------------------------------------------------------
// ActivityIndicator page
// ---------------------------------------------------------------------------

const AI_START: i32 = 0;
const AI_STOP: i32 = 1;
const AI_IS_RUNNING: i32 = 2;

fn build_activity_indicator(
    panel: &ScrolledWindow,
    attrs: &Attrs,
    ctx: *mut c_void,
    closures: &mut Vec<Closure>,
) -> PageWidgets {
    let oper_box = StaticBox::new_full(
        panel,
        constants::ID_ANY(),
        "&Operations",
        Rect::default(),
        0,
    );
    let sizer_oper = StaticBoxSizer::new(&oper_box, constants::VERTICAL());
    let oper_parent = static_box_parent(&sizer_oper);

    let btn_start = Button::new_full(
        &oper_parent,
        page_id(AI_START),
        "&Start",
        Rect::default(),
        0,
    );
    add_window_flags(&sizer_oper, &btn_start, expand_border());
    let btn_stop = Button::new_full(&oper_parent, page_id(AI_STOP), "&Stop", Rect::default(), 0);
    add_window_flags(&sizer_oper, &btn_stop, expand_border());
    let status = StaticText::new_full(
        &oper_parent,
        page_id(AI_IS_RUNNING),
        "Indicator is initializing...",
        Rect::default(),
        0,
    );
    add_window_flags(&sizer_oper, &status, expand_border());

    // m_sizerIndicator = new wxStaticBoxSizer(wxHORIZONTAL, this,
    //                                         "Activity Indicator");
    let indicator_box = StaticBox::new_full(
        panel,
        constants::ID_ANY(),
        "Activity Indicator",
        Rect::default(),
        0,
    );
    let sizer_indicator = StaticBoxSizer::new(&indicator_box, constants::HORIZONTAL());
    let indicator_parent = static_box_parent(&sizer_indicator);

    // RecreateWidget(): the indicator uses the shared default flags.
    let indicator = ActivityIndicator::new_full(
        &indicator_parent,
        constants::ID_ANY(),
        -1,
        -1,
        -1,
        -1,
        attrs.default_flags,
    );
    sizer_indicator.add_stretch_spacer(0);
    {
        let f = SizerFlags::new(0);
        f.center();
        add_window_flags(&sizer_indicator, &indicator, f);
    }
    sizer_indicator.add_stretch_spacer(0);
    sizer_indicator.layout();

    connect_events!(&indicator, ctx, closures);

    let sizer_top = BoxSizer::new(constants::HORIZONTAL());
    add_sizer_flags(&sizer_top, &sizer_oper, expand_double_border());
    add_sizer_flags(&sizer_top, &sizer_indicator, expand_double_border_prop(1));
    panel.set_sizer(&sizer_top, true);

    // Event table
    let start = Closure::new(handle_ai_start as *const () as *mut c_void, ctx);
    let stop = Closure::new(handle_ai_stop as *const () as *mut c_void, ctx);
    let update = Closure::new(handle_ai_update as *const () as *mut c_void, ctx);
    let button = events::EVT_BUTTON();
    panel.bind(page_id(AI_START), page_id(AI_START), button, &start);
    panel.bind(page_id(AI_STOP), page_id(AI_STOP), button, &stop);
    let is_running = page_id(AI_IS_RUNNING);
    panel.bind(is_running, is_running, events::EVT_UPDATE_UI(), &update);
    closures.push(start);
    closures.push(stop);
    closures.push(update);

    PageWidgets::ActivityIndicator { indicator, status }
}

extern "C" fn handle_ai_start(_fun: *mut c_void, data: *mut c_void, event: *mut c_void) {
    if event.is_null() {
        return;
    }
    with_state(data, |state| {
        if let Some(PageWidgets::ActivityIndicator { indicator, .. }) =
            current_widgets(state, Kind::ActivityIndicator)
        {
            indicator.start();
        }
    });
}

extern "C" fn handle_ai_stop(_fun: *mut c_void, data: *mut c_void, event: *mut c_void) {
    if event.is_null() {
        return;
    }
    with_state(data, |state| {
        if let Some(PageWidgets::ActivityIndicator { indicator, .. }) =
            current_widgets(state, Kind::ActivityIndicator)
        {
            indicator.stop();
        }
    });
}

extern "C" fn handle_ai_update(_fun: *mut c_void, data: *mut c_void, event: *mut c_void) {
    if event.is_null() {
        return;
    }
    let event = unsafe { UpdateUIEvent::from_ptr(event) };
    let running = with_state(data, |state| {
        match current_widgets(state, Kind::ActivityIndicator) {
            Some(PageWidgets::ActivityIndicator { indicator, .. }) => indicator.is_running(),
            _ => false,
        }
    })
    .unwrap_or(false);

    event.set_text(if running {
        "Indicator is running"
    } else {
        "Indicator is stopped"
    });
}

// ---------------------------------------------------------------------------
// Choice page
// ---------------------------------------------------------------------------

const CH_RESET: i32 = 0;
const CH_CHOICE: i32 = 1;
const CH_SORT: i32 = 2;
const CH_ADD: i32 = 3;
const CH_ADD_TEXT: i32 = 4;
const CH_ADD_SEVERAL: i32 = 5;
const CH_ADD_MANY: i32 = 6;
const CH_CHANGE: i32 = 7;
const CH_CHANGE_TEXT: i32 = 8;
const CH_DELETE: i32 = 9;
const CH_DELETE_TEXT: i32 = 10;
const CH_DELETE_SEL: i32 = 11;
const CH_CLEAR: i32 = 12;

fn build_choice(
    panel: &ScrolledWindow,
    attrs: &Attrs,
    ctx: *mut c_void,
    closures: &mut Vec<Closure>,
) -> PageWidgets {
    // ---- &Set choice parameters -------------------------------------------
    let params_box = StaticBox::new_full(
        panel,
        constants::ID_ANY(),
        "&Set choice parameters",
        Rect::default(),
        0,
    );
    let sizer_params = StaticBoxSizer::new(&params_box, constants::VERTICAL());
    let params_parent = static_box_parent(&sizer_params);

    let chk_sort = CheckBox::new_full(
        &params_parent,
        page_id(CH_SORT),
        "&Sort items",
        Rect::default(),
        0,
    );
    add_window_flags(&sizer_params, &chk_sort, expand_border());
    let btn_reset = Button::new_full(
        &params_parent,
        page_id(CH_RESET),
        "&Reset",
        Rect::default(),
        0,
    );
    add_window_flags(&sizer_params, &btn_reset, expand_border());

    // ---- &Change choice contents ------------------------------------------
    let contents_box = StaticBox::new_full(
        panel,
        constants::ID_ANY(),
        "&Change choice contents",
        Rect::default(),
        0,
    );
    let sizer_contents = StaticBoxSizer::new(&contents_box, constants::VERTICAL());
    let contents_parent = static_box_parent(&sizer_contents);

    let add_row = BoxSizer::new(constants::HORIZONTAL());
    let btn_add = Button::new_full(
        &contents_parent,
        page_id(CH_ADD),
        "&Add this string",
        Rect::default(),
        0,
    );
    add_window_flags(&add_row, &btn_add, center_vertical_right());
    let add_text = TextCtrl::new_full(
        &contents_parent,
        page_id(CH_ADD_TEXT),
        "test item 0",
        Rect::default(),
        constants::TE_PROCESS_ENTER(),
    );
    add_window_flags(&add_row, &add_text, text_entry_flags());
    add_sizer_flags(&sizer_contents, &add_row, expand_border());

    let btn_add_several = Button::new_full(
        &contents_parent,
        page_id(CH_ADD_SEVERAL),
        "&Insert a few strings",
        Rect::default(),
        0,
    );
    add_window_flags(&sizer_contents, &btn_add_several, expand_border());
    let btn_add_many = Button::new_full(
        &contents_parent,
        page_id(CH_ADD_MANY),
        "Add &many strings",
        Rect::default(),
        0,
    );
    add_window_flags(&sizer_contents, &btn_add_many, expand_border());

    let change_row = BoxSizer::new(constants::HORIZONTAL());
    let btn_change = Button::new_full(
        &contents_parent,
        page_id(CH_CHANGE),
        "C&hange current",
        Rect::default(),
        0,
    );
    add_window_flags(&change_row, &btn_change, center_vertical_right());
    let change_text = TextCtrl::new_full(
        &contents_parent,
        page_id(CH_CHANGE_TEXT),
        "",
        Rect::default(),
        constants::TE_PROCESS_ENTER(),
    );
    add_window_flags(&change_row, &change_text, text_entry_flags());
    add_sizer_flags(&sizer_contents, &change_row, expand_border());

    let delete_row = BoxSizer::new(constants::HORIZONTAL());
    let btn_delete = Button::new_full(
        &contents_parent,
        page_id(CH_DELETE),
        "&Delete this item",
        Rect::default(),
        0,
    );
    add_window_flags(&delete_row, &btn_delete, center_vertical_right());
    let delete_text = TextCtrl::new_full(
        &contents_parent,
        page_id(CH_DELETE_TEXT),
        "",
        Rect::default(),
        constants::TE_PROCESS_ENTER(),
    );
    add_window_flags(&delete_row, &delete_text, text_entry_flags());
    add_sizer_flags(&sizer_contents, &delete_row, expand_border());

    let btn_delete_sel = Button::new_full(
        &contents_parent,
        page_id(CH_DELETE_SEL),
        "Delete &selection",
        Rect::default(),
        0,
    );
    add_window_flags(&sizer_contents, &btn_delete_sel, expand_border());
    let btn_clear = Button::new_full(
        &contents_parent,
        page_id(CH_CLEAR),
        "&Clear",
        Rect::default(),
        0,
    );
    add_window_flags(&sizer_contents, &btn_clear, expand_border());

    // ---- the choice itself -------------------------------------------------
    let style = if chk_sort.get_value() {
        attrs.default_flags | constants::CB_SORT()
    } else {
        attrs.default_flags
    };
    let sizer_choice = BoxSizer::new(constants::VERTICAL());
    let choice = Choice::new_full(
        panel,
        page_id(CH_CHOICE),
        Rect::default(),
        0,
        std::ptr::null_mut(),
        style,
    );
    choice.set_min_size(Size {
        width: 150,
        height: -1,
    });
    add_window_flags(&sizer_choice, &choice, expand_border());

    connect_events!(&choice, ctx, closures);

    let sizer_top = BoxSizer::new(constants::HORIZONTAL());
    add_sizer_flags(&sizer_top, &sizer_params, expand_double_border());
    add_sizer_flags(&sizer_top, &sizer_contents, expand_double_border());
    add_sizer_flags(&sizer_top, &sizer_choice, expand_double_border_prop(1));
    panel.set_sizer(&sizer_top, true);

    // ---- event table -------------------------------------------------------
    let button = events::EVT_BUTTON();
    for (id, handler) in [
        (CH_RESET, handle_choice_reset as Handler),
        (CH_ADD, handle_choice_add),
        (CH_ADD_SEVERAL, handle_choice_add_several),
        (CH_ADD_MANY, handle_choice_add_many),
        (CH_CHANGE, handle_choice_change),
        (CH_DELETE, handle_choice_delete),
        (CH_DELETE_SEL, handle_choice_delete_sel),
        (CH_CLEAR, handle_choice_clear),
    ] {
        let closure = Closure::new(handler as *const () as *mut c_void, ctx);
        panel.bind(page_id(id), page_id(id), button, &closure);
        closures.push(closure);
    }
    let on_check = Closure::new(handle_choice_style as *const () as *mut c_void, ctx);
    panel.bind(
        page_id(CH_SORT),
        page_id(CH_SORT),
        events::EVT_CHECKBOX(),
        &on_check,
    );
    closures.push(on_check);
    let on_select = Closure::new(handle_choice_select as *const () as *mut c_void, ctx);
    panel.bind(
        page_id(CH_CHOICE),
        page_id(CH_CHOICE),
        events::EVT_CHOICE(),
        &on_select,
    );
    closures.push(on_select);

    PageWidgets::Choice {
        choice,
        sort: chk_sort,
        add_text,
        change_text,
        delete_text,
    }
}

extern "C" fn handle_choice_add(_fun: *mut c_void, data: *mut c_void, event: *mut c_void) {
    if event.is_null() {
        return;
    }
    with_state(data, |state| {
        let pair = match current_widgets(state, Kind::Choice) {
            Some(PageWidgets::Choice {
                choice, add_text, ..
            }) => Some((*choice, *add_text)),
            _ => None,
        };
        if let Some((choice, add_text)) = pair {
            choice.append(add_text.get_value().as_str());
        }
    });
}

extern "C" fn handle_choice_add_several(_fun: *mut c_void, data: *mut c_void, event: *mut c_void) {
    if event.is_null() {
        return;
    }
    with_state(data, |state| {
        if let Some(PageWidgets::Choice { choice, .. }) = current_widgets(state, Kind::Choice) {
            // m_choice->Insert(items, 0) with the sample's three strings.
            for (pos, item) in [
                "First",
                "another one",
                "and the last (very very very very very very very very very very long) one",
            ]
            .iter()
            .enumerate()
            {
                choice.insert(item, pos as u32);
            }
        }
    });
}

extern "C" fn handle_choice_add_many(_fun: *mut c_void, data: *mut c_void, event: *mut c_void) {
    if event.is_null() {
        return;
    }
    with_state(data, |state| {
        if let Some(PageWidgets::Choice { choice, .. }) = current_widgets(state, Kind::Choice) {
            // "many" means 1000 here.
            for n in 0..1000 {
                choice.append(format!("item #{n}").as_str());
            }
        }
    });
}

extern "C" fn handle_choice_change(_fun: *mut c_void, data: *mut c_void, event: *mut c_void) {
    if event.is_null() {
        return;
    }
    with_state(data, |state| {
        let pair = match current_widgets(state, Kind::Choice) {
            Some(PageWidgets::Choice {
                choice,
                change_text,
                ..
            }) => Some((*choice, *change_text)),
            _ => None,
        };
        if let Some((choice, change_text)) = pair {
            let sel = choice.get_selection();
            if sel != -1 {
                choice.set_string(sel, change_text.get_value().as_str());
            }
        }
    });
}

extern "C" fn handle_choice_delete(_fun: *mut c_void, data: *mut c_void, event: *mut c_void) {
    if event.is_null() {
        return;
    }
    with_state(data, |state| {
        let pair = match current_widgets(state, Kind::Choice) {
            Some(PageWidgets::Choice {
                choice,
                delete_text,
                ..
            }) => Some((*choice, *delete_text)),
            _ => None,
        };
        if let Some((choice, delete_text)) = pair {
            // The entry is an index: !GetValue().ToULong(&n) || n >= count.
            let Ok(index) = delete_text.get_value().trim().parse::<i32>() else {
                return;
            };
            if index < choice.get_count() {
                choice.delete(index);
            }
        }
    });
}

extern "C" fn handle_choice_delete_sel(_fun: *mut c_void, data: *mut c_void, event: *mut c_void) {
    if event.is_null() {
        return;
    }
    with_state(data, |state| {
        if let Some(PageWidgets::Choice { choice, .. }) = current_widgets(state, Kind::Choice) {
            let sel = choice.get_selection();
            if sel != -1 {
                choice.delete(sel);
            }
        }
    });
}

extern "C" fn handle_choice_clear(_fun: *mut c_void, data: *mut c_void, event: *mut c_void) {
    if event.is_null() {
        return;
    }
    with_state(data, |state| {
        if let Some(PageWidgets::Choice { choice, .. }) = current_widgets(state, Kind::Choice) {
            choice.clear();
        }
    });
}

extern "C" fn handle_choice_reset(_fun: *mut c_void, data: *mut c_void, event: *mut c_void) {
    if event.is_null() {
        return;
    }
    with_state(data, |state| {
        if let Some(PageWidgets::Choice { sort, .. }) = current_widgets(state, Kind::Choice) {
            sort.set_value(false);
        }
        rebuild_current(state, Kind::Choice);
    });
}

extern "C" fn handle_choice_style(_fun: *mut c_void, data: *mut c_void, event: *mut c_void) {
    if event.is_null() {
        return;
    }
    with_state(data, |state| rebuild_current(state, Kind::Choice));
}

extern "C" fn handle_choice_select(_fun: *mut c_void, data: *mut c_void, event: *mut c_void) {
    if event.is_null() {
        return;
    }
    let event = unsafe { CommandEvent::from_ptr(event) };
    let index = event.get_selection();
    with_state(data, |state| {
        log_msg(state, &format!("Choice item {index} selected."));
        if let Some(PageWidgets::Choice { delete_text, .. }) = current_widgets(state, Kind::Choice)
        {
            delete_text.set_value(format!("{index}").as_str());
        }
    });
}

// ---------------------------------------------------------------------------
// Gauge page
// ---------------------------------------------------------------------------

const GA_RESET: i32 = 0;
const GA_GAUGE: i32 = 1;
const GA_CUR_VALUE_TEXT: i32 = 2;
const GA_VALUE_TEXT: i32 = 3;
const GA_RANGE_TEXT: i32 = 4;
const GA_SET_RANGE: i32 = 5;
const GA_PROGRESS: i32 = 6;
const GA_INDETERMINATE: i32 = 7;
const GA_CLEAR: i32 = 8;
const GA_TIMER: i32 = 9;
const GA_VERTICAL: i32 = 10;
const GA_SMOOTH: i32 = 11;
const GA_PROGRESS_STYLE: i32 = 12;
const GA_SET_VALUE: i32 = 13;

fn build_gauge(
    panel: &ScrolledWindow,
    attrs: &Attrs,
    ctx: *mut c_void,
    closures: &mut Vec<Closure>,
) -> PageWidgets {
    // ---- &Set style --------------------------------------------------------
    let style_box =
        StaticBox::new_full(panel, constants::ID_ANY(), "&Set style", Rect::default(), 0);
    let sizer_style = StaticBoxSizer::new(&style_box, constants::VERTICAL());
    let style_parent = static_box_parent(&sizer_style);

    let chk_vertical = CheckBox::new_full(
        &style_parent,
        page_id(GA_VERTICAL),
        "&Vertical",
        Rect::default(),
        0,
    );
    add_window_flags(&sizer_style, &chk_vertical, expand_border());
    let chk_smooth = CheckBox::new_full(
        &style_parent,
        page_id(GA_SMOOTH),
        "&Smooth",
        Rect::default(),
        0,
    );
    add_window_flags(&sizer_style, &chk_smooth, expand_border());
    let chk_progress = CheckBox::new_full(
        &style_parent,
        page_id(GA_PROGRESS_STYLE),
        "&Progress",
        Rect::default(),
        0,
    );
    add_window_flags(&sizer_style, &chk_progress, expand_border());
    let btn_reset = Button::new_full(
        &style_parent,
        page_id(GA_RESET),
        "&Reset",
        Rect::default(),
        0,
    );
    add_window_flags(&sizer_style, &btn_reset, expand_border());

    // ---- &Change gauge value ----------------------------------------------
    let value_box = StaticBox::new_full(
        panel,
        constants::ID_ANY(),
        "&Change gauge value",
        Rect::default(),
        0,
    );
    let sizer_value = StaticBoxSizer::new(&value_box, constants::VERTICAL());
    let value_parent = static_box_parent(&sizer_value);

    let cur_row = BoxSizer::new(constants::HORIZONTAL());
    let cur_label = StaticText::new_full(
        &value_parent,
        constants::ID_ANY(),
        "Current value",
        Rect::default(),
        0,
    );
    add_window_flags(&cur_row, &cur_label, center_vertical_right());
    let cur_value_text = TextCtrl::new_full(
        &value_parent,
        page_id(GA_CUR_VALUE_TEXT),
        "0",
        Rect::default(),
        constants::TE_READONLY(),
    );
    add_window_flags(&cur_row, &cur_value_text, text_entry_flags());
    add_sizer_flags(&sizer_value, &cur_row, expand_border());

    let set_row = BoxSizer::new(constants::HORIZONTAL());
    let btn_set = Button::new_full(
        &value_parent,
        page_id(GA_SET_VALUE),
        "Set &value",
        Rect::default(),
        0,
    );
    add_window_flags(&set_row, &btn_set, center_vertical_right());
    let value_text = TextCtrl::new_full(
        &value_parent,
        page_id(GA_VALUE_TEXT),
        "0",
        Rect::default(),
        constants::TE_PROCESS_ENTER(),
    );
    add_window_flags(&set_row, &value_text, text_entry_flags());
    add_sizer_flags(&sizer_value, &set_row, expand_border());

    let range_row = BoxSizer::new(constants::HORIZONTAL());
    let btn_range = Button::new_full(
        &value_parent,
        page_id(GA_SET_RANGE),
        "Set &range",
        Rect::default(),
        0,
    );
    add_window_flags(&range_row, &btn_range, center_vertical_right());
    let range_text = TextCtrl::new_full(
        &value_parent,
        page_id(GA_RANGE_TEXT),
        "100",
        Rect::default(),
        constants::TE_PROCESS_ENTER(),
    );
    add_window_flags(&range_row, &range_text, text_entry_flags());
    add_sizer_flags(&sizer_value, &range_row, expand_border());

    let progress_btn = Button::new_full(
        &value_parent,
        page_id(GA_PROGRESS),
        "Simulate &progress",
        Rect::default(),
        0,
    );
    add_window_flags(&sizer_value, &progress_btn, expand_border());
    let indeterminate_btn = Button::new_full(
        &value_parent,
        page_id(GA_INDETERMINATE),
        "Simulate &indeterminate job",
        Rect::default(),
        0,
    );
    add_window_flags(&sizer_value, &indeterminate_btn, expand_border());
    let btn_clear = Button::new_full(
        &value_parent,
        page_id(GA_CLEAR),
        "&Clear",
        Rect::default(),
        0,
    );
    add_window_flags(&sizer_value, &btn_clear, expand_border());

    // ---- the gauge ---------------------------------------------------------
    let sizer_gauge = BoxSizer::new(constants::VERTICAL());
    let mut flags = attrs.default_flags;
    flags |= if chk_vertical.get_value() {
        constants::GA_VERTICAL()
    } else {
        constants::GA_HORIZONTAL()
    };
    if chk_smooth.get_value() {
        flags |= constants::GA_SMOOTH();
    }
    if chk_progress.get_value() {
        flags |= constants::GA_PROGRESS();
    }
    let gauge = Gauge::new_full(panel, page_id(GA_GAUGE), 100, Rect::default(), flags);
    gauge.set_min_size(Size {
        width: 150,
        height: -1,
    });
    add_window_flags(&sizer_gauge, &gauge, expand_border());

    connect_events!(&gauge, ctx, closures);

    let sizer_top = BoxSizer::new(constants::HORIZONTAL());
    add_sizer_flags(&sizer_top, &sizer_style, expand_double_border());
    add_sizer_flags(&sizer_top, &sizer_value, expand_double_border());
    add_sizer_flags(&sizer_top, &sizer_gauge, expand_double_border_prop(1));
    panel.set_sizer(&sizer_top, true);

    // The progress timer; the sample keeps two timers, this port keeps one and
    // an `indeterminate` mode flag on the page.
    let timer = Timer::new_full(panel, page_id(GA_TIMER));

    let button = events::EVT_BUTTON();
    for (id, handler) in [
        (GA_RESET, handle_gauge_reset as Handler),
        (GA_SET_VALUE, handle_gauge_set_value),
        (GA_PROGRESS, handle_gauge_progress),
        (GA_INDETERMINATE, handle_gauge_indeterminate),
        (GA_CLEAR, handle_gauge_clear),
        (GA_SET_RANGE, handle_gauge_set_range),
    ] {
        let closure = Closure::new(handler as *const () as *mut c_void, ctx);
        panel.bind(page_id(id), page_id(id), button, &closure);
        closures.push(closure);
    }
    let on_check = Closure::new(handle_gauge_style as *const () as *mut c_void, ctx);
    for id in [GA_VERTICAL, GA_SMOOTH, GA_PROGRESS_STYLE] {
        panel.bind(page_id(id), page_id(id), events::EVT_CHECKBOX(), &on_check);
    }
    closures.push(on_check);

    let on_timer = Closure::new(handle_gauge_timer as *const () as *mut c_void, ctx);
    panel.bind(
        page_id(GA_TIMER),
        page_id(GA_TIMER),
        events::EVT_TIMER(),
        &on_timer,
    );
    closures.push(on_timer);

    PageWidgets::Gauge {
        gauge,
        timer,
        indeterminate: false,
        value_text,
        range_text,
        cur_value_text,
        progress_btn,
        indeterminate_btn,
    }
}

/// `CreateGauge()` after a style checkbox changed: the range and value survive.
extern "C" fn handle_gauge_style(_fun: *mut c_void, data: *mut c_void, event: *mut c_void) {
    if event.is_null() {
        return;
    }
    with_state(data, |state| {
        let (value, range) = match current_widgets(state, Kind::Gauge) {
            Some(PageWidgets::Gauge { gauge, .. }) => (gauge.get_value(), gauge.get_range()),
            _ => (0, 100),
        };
        rebuild_current(state, Kind::Gauge);
        if let Some(PageWidgets::Gauge {
            gauge, range_text, ..
        }) = current_widgets(state, Kind::Gauge)
        {
            gauge.set_range(range);
            gauge.set_value(value);
            range_text.set_value(format!("{range}").as_str());
        }
    });
}

extern "C" fn handle_gauge_reset(_fun: *mut c_void, data: *mut c_void, event: *mut c_void) {
    if event.is_null() {
        return;
    }
    with_state(data, |state| {
        let parts = match current_widgets(state, Kind::Gauge) {
            Some(PageWidgets::Gauge {
                timer,
                value_text,
                range_text,
                gauge,
                progress_btn,
                indeterminate_btn,
                indeterminate,
                ..
            }) => Some((
                *timer,
                *value_text,
                *range_text,
                *gauge,
                *progress_btn,
                *indeterminate_btn,
                indeterminate,
            )),
            _ => None,
        };
        if let Some((timer, value_text, range_text, gauge, progress_btn, indeterminate_btn, mode)) =
            parts
        {
            timer.stop();
            *mode = false;
            progress_btn.set_label("Simulate &progress");
            indeterminate_btn.set_label("Simulate &indeterminate job");
            indeterminate_btn.enable(true);
            value_text.set_value("0");
            range_text.set_value("100");
            gauge.set_range(100);
            gauge.set_value(0);
        }
    });
}

extern "C" fn handle_gauge_set_value(_fun: *mut c_void, data: *mut c_void, event: *mut c_void) {
    if event.is_null() {
        return;
    }
    with_state(data, |state| {
        let pair = match current_widgets(state, Kind::Gauge) {
            Some(PageWidgets::Gauge {
                gauge, value_text, ..
            }) => Some((*gauge, *value_text)),
            _ => None,
        };
        if let Some((gauge, value_text)) = pair {
            if let Ok(value) = value_text.get_value().trim().parse::<i32>() {
                gauge.set_value(value);
            }
        }
    });
}

extern "C" fn handle_gauge_set_range(_fun: *mut c_void, data: *mut c_void, event: *mut c_void) {
    if event.is_null() {
        return;
    }
    with_state(data, |state| {
        let pair = match current_widgets(state, Kind::Gauge) {
            Some(PageWidgets::Gauge {
                gauge, range_text, ..
            }) => Some((*gauge, *range_text)),
            _ => None,
        };
        if let Some((gauge, range_text)) = pair {
            if let Ok(range) = range_text.get_value().trim().parse::<i32>() {
                gauge.set_range(range);
            }
        }
    });
}

extern "C" fn handle_gauge_clear(_fun: *mut c_void, data: *mut c_void, event: *mut c_void) {
    if event.is_null() {
        return;
    }
    with_state(data, |state| {
        if let Some(PageWidgets::Gauge { gauge, .. }) = current_widgets(state, Kind::Gauge) {
            gauge.set_value(0);
        }
    });
}

extern "C" fn handle_gauge_progress(_fun: *mut c_void, data: *mut c_void, event: *mut c_void) {
    if event.is_null() {
        return;
    }
    with_state(data, |state| {
        let running = match current_widgets(state, Kind::Gauge) {
            Some(PageWidgets::Gauge { timer, .. }) => timer.is_running(),
            _ => false,
        };
        if running {
            if let Some(PageWidgets::Gauge {
                timer,
                progress_btn,
                indeterminate_btn,
                ..
            }) = current_widgets(state, Kind::Gauge)
            {
                timer.stop();
                progress_btn.set_label("Simulate &progress");
                indeterminate_btn.enable(true);
                log_msg(state, "Stopped the timer.");
            }
            return;
        }
        let parts = match current_widgets(state, Kind::Gauge) {
            Some(PageWidgets::Gauge {
                timer,
                gauge,
                progress_btn,
                indeterminate_btn,
                indeterminate,
                ..
            }) => Some((
                *timer,
                *gauge,
                *progress_btn,
                *indeterminate_btn,
                indeterminate,
            )),
            _ => None,
        };
        if let Some((timer, gauge, progress_btn, indeterminate_btn, mode)) = parts {
            *mode = false;
            gauge.set_value(0);
            timer.start(300, false);
            progress_btn.set_label("&Stop timer");
            indeterminate_btn.enable(false);
            log_msg(state, "Launched progress timer (interval = 300 ms).");
        }
    });
}

extern "C" fn handle_gauge_indeterminate(_fun: *mut c_void, data: *mut c_void, event: *mut c_void) {
    if event.is_null() {
        return;
    }
    with_state(data, |state| {
        let running = match current_widgets(state, Kind::Gauge) {
            Some(PageWidgets::Gauge { timer, .. }) => timer.is_running(),
            _ => false,
        };
        if running {
            if let Some(PageWidgets::Gauge {
                timer,
                gauge,
                indeterminate_btn,
                progress_btn,
                ..
            }) = current_widgets(state, Kind::Gauge)
            {
                timer.stop();
                gauge.set_value(0);
                indeterminate_btn.set_label("Simulate &indeterminate job");
                progress_btn.enable(true);
                log_msg(state, "Stopped the timer.");
            }
            return;
        }
        let parts = match current_widgets(state, Kind::Gauge) {
            Some(PageWidgets::Gauge {
                timer,
                indeterminate_btn,
                progress_btn,
                indeterminate,
                ..
            }) => Some((*timer, *indeterminate_btn, *progress_btn, indeterminate)),
            _ => None,
        };
        if let Some((timer, indeterminate_btn, progress_btn, mode)) = parts {
            *mode = true;
            timer.start(300, false);
            indeterminate_btn.set_label("&Stop timer");
            progress_btn.enable(false);
            log_msg(state, "Launched progress timer (interval = 300 ms).");
        }
    });
}

extern "C" fn handle_gauge_timer(_fun: *mut c_void, data: *mut c_void, event: *mut c_void) {
    if event.is_null() {
        return;
    }
    with_state(data, |state| {
        let parts = match current_widgets(state, Kind::Gauge) {
            Some(PageWidgets::Gauge {
                gauge,
                cur_value_text,
                timer,
                indeterminate_btn,
                progress_btn,
                indeterminate,
                ..
            }) => Some((
                *gauge,
                *cur_value_text,
                *timer,
                *indeterminate_btn,
                *progress_btn,
                *indeterminate,
            )),
            _ => None,
        };
        let Some((gauge, cur_value_text, timer, indeterminate_btn, progress_btn, mode)) = parts
        else {
            return;
        };

        if mode {
            gauge.pulse();
            return;
        }

        let value = gauge.get_value() + 1;
        let range = gauge.get_range();
        if value >= range {
            timer.stop();
            progress_btn.set_label("Simulate &progress");
            indeterminate_btn.enable(true);
            gauge.set_value(range);
            log_msg(state, "Progress finished.");
        } else {
            gauge.set_value(value);
        }
        cur_value_text.set_value(format!("{}", gauge.get_value()).as_str());
    });
}

// ---------------------------------------------------------------------------
// Widget event hooks (`WidgetsApp::ConnectToWidgetEvents`)
// ---------------------------------------------------------------------------

/// Per-widget context for the four logging hooks: the shared state plus the
/// class name the C++ reports through `GetClassInfo()->GetClassName()`.
struct HookCtx {
    state: *mut c_void,
    class: &'static str,
}

extern "C" fn handle_focus(_fun: *mut c_void, data: *mut c_void, event: *mut c_void) {
    if event.is_null() || data.is_null() {
        return;
    }
    let hook = unsafe { &*(data as *const HookCtx) };
    let event = unsafe { Event::from_ptr(event) };
    let got = event.get_event_type() == events::EVT_SET_FOCUS();
    with_state(hook.state, |state| {
        log_msg(
            state,
            &format!(
                "Widget '{}' {} focus",
                hook.class,
                if got { "got" } else { "lost" }
            ),
        );
    });
    event.skip(true);
}

extern "C" fn handle_enter(_fun: *mut c_void, data: *mut c_void, event: *mut c_void) {
    if event.is_null() || data.is_null() {
        return;
    }
    let hook = unsafe { &*(data as *const HookCtx) };
    with_state(hook.state, |state| {
        log_msg(state, &format!("Mouse entered into '{}'", hook.class));
    });
    unsafe { Event::from_ptr(event) }.skip(true);
}

extern "C" fn handle_leave(_fun: *mut c_void, data: *mut c_void, event: *mut c_void) {
    if event.is_null() || data.is_null() {
        return;
    }
    let hook = unsafe { &*(data as *const HookCtx) };
    with_state(hook.state, |state| {
        log_msg(state, &format!("Mouse left '{}'", hook.class));
    });
    unsafe { Event::from_ptr(event) }.skip(true);
}

extern "C" fn handle_context_menu(_fun: *mut c_void, data: *mut c_void, event: *mut c_void) {
    if event.is_null() || data.is_null() {
        return;
    }
    let hook = unsafe { &*(data as *const HookCtx) };
    let event = unsafe { ContextMenuEvent::from_ptr(event) };
    let mut x = 0;
    let mut y = 0;
    event.get_position(&mut x, &mut y);
    with_state(hook.state, |state| {
        log_msg(
            state,
            &format!("Context menu event for '{}' at {}x{}", hook.class, x, y),
        );
    });
    event.skip(true);
}

// ---------------------------------------------------------------------------
// Frame handlers
// ---------------------------------------------------------------------------

extern "C" fn handle_clear_log(_fun: *mut c_void, data: *mut c_void, event: *mut c_void) {
    if event.is_null() {
        return;
    }
    with_state(data, |state| state.lbox_log.clear());
}

extern "C" fn handle_exit(_fun: *mut c_void, data: *mut c_void, event: *mut c_void) {
    if event.is_null() {
        return;
    }
    let frame = with_state(data, |state| state.frame);
    if let Some(frame) = frame {
        frame.close(false);
    }
}

extern "C" fn handle_set_tooltip(_fun: *mut c_void, data: *mut c_void, event: *mut c_void) {
    if event.is_null() {
        return;
    }
    let pair = with_state(data, |state| (state.frame, state.attrs.tooltip));
    let Some((frame, tooltip)) = pair else {
        return;
    };

    // wxTextEntryDialog(this, "Tooltip text (may use \\n, leave empty to
    // remove): ", "Widgets sample", attrs.m_tooltip)
    let dialog = TextEntryDialog::new_full(
        &frame,
        "Tooltip text (may use \\n, leave empty to remove): ",
        "Widgets sample",
        tooltip,
        0,
        0,
        0,
        Size::default(),
    );
    let result = dialog.show_modal();
    let value = dialog.get_value();
    if result == constants::ID_OK() {
        let value = value.replace("\\n", "\n");
        with_state(data, |state| {
            state.attrs.tooltip = Box::leak(value.into_boxed_str());
            if let Some(index) = state.current {
                recreate_page(state, index);
            }
            setup_widget(state);
        });
    }
}

extern "C" fn handle_set_fg(_fun: *mut c_void, data: *mut c_void, event: *mut c_void) {
    if event.is_null() {
        return;
    }
    with_state(data, |state| {
        log_msg(
            state,
            "Set foreground colour is not available in this binding.",
        )
    });
}

extern "C" fn handle_set_bg(_fun: *mut c_void, data: *mut c_void, event: *mut c_void) {
    if event.is_null() {
        return;
    }
    with_state(data, |state| {
        log_msg(
            state,
            "Set background colour is not available in this binding.",
        )
    });
}

extern "C" fn handle_set_page_bg(_fun: *mut c_void, data: *mut c_void, event: *mut c_void) {
    if event.is_null() {
        return;
    }
    with_state(data, |state| {
        log_msg(
            state,
            "Set page background colour is not available in this binding.",
        )
    });
}

extern "C" fn handle_set_font(_fun: *mut c_void, data: *mut c_void, event: *mut c_void) {
    if event.is_null() {
        return;
    }
    with_state(data, |state| {
        log_msg(state, "Set font is not available in this binding.")
    });
}

/// `OnEnable`: `m_enabled = event.IsChecked(); SetUpWidget();`
extern "C" fn handle_enable(_fun: *mut c_void, data: *mut c_void, event: *mut c_void) {
    if event.is_null() {
        return;
    }
    let event = unsafe { CommandEvent::from_ptr(event) };
    let checked = event.is_checked();
    with_state(data, |state| {
        state.attrs.enabled = checked;
        setup_widget(state);
    });
}

/// `OnShow`: `m_show = event.IsChecked(); SetUpWidget();`
extern "C" fn handle_show(_fun: *mut c_void, data: *mut c_void, event: *mut c_void) {
    if event.is_null() {
        return;
    }
    let event = unsafe { CommandEvent::from_ptr(event) };
    let checked = event.is_checked();
    with_state(data, |state| {
        state.attrs.show = checked;
        setup_widget(state);
    });
}

/// `OnSetBorder`: replaces the border flags in `m_defaultFlags` and recreates
/// the widget.
extern "C" fn handle_set_border(_fun: *mut c_void, data: *mut c_void, event: *mut c_void) {
    if event.is_null() {
        return;
    }
    let event = unsafe { CommandEvent::from_ptr(event) };
    let id = event.get_id();
    let border = match id {
        WIDGETS_BORDER_NONE => constants::BORDER_NONE(),
        WIDGETS_BORDER_STATIC => constants::BORDER_STATIC(),
        WIDGETS_BORDER_SIMPLE => constants::BORDER_SIMPLE(),
        WIDGETS_BORDER_RAISED => constants::BORDER_RAISED(),
        WIDGETS_BORDER_SUNKEN => constants::BORDER_SUNKEN(),
        WIDGETS_BORDER_DOUBLE => constants::BORDER_DOUBLE(),
        _ => constants::BORDER_DEFAULT(),
    };
    with_state(data, |state| {
        // The menu items are the only source of border flags, and they are
        // mutually exclusive, so assigning the flag is exact here.
        state.attrs.default_flags = border;
        if let Some(index) = state.current {
            recreate_page(state, index);
            setup_widget(state);
        }
    });
}

/// `OnSetVariant`.
extern "C" fn handle_set_variant(_fun: *mut c_void, data: *mut c_void, event: *mut c_void) {
    if event.is_null() {
        return;
    }
    let event = unsafe { CommandEvent::from_ptr(event) };
    let variant = match event.get_id() {
        WIDGETS_VARIANT_MINI => constants::WINDOW_VARIANT_MINI(),
        WIDGETS_VARIANT_SMALL => constants::WINDOW_VARIANT_SMALL(),
        WIDGETS_VARIANT_LARGE => constants::WINDOW_VARIANT_LARGE(),
        _ => constants::WINDOW_VARIANT_NORMAL(),
    };
    with_state(data, |state| {
        state.attrs.variant = variant;
        if let Some(index) = state.current {
            recreate_page(state, index);
        }
        setup_widget(state);
    });
}

/// `OnToggleLayoutDirection`.
extern "C" fn handle_toggle_layout(_fun: *mut c_void, data: *mut c_void, event: *mut c_void) {
    if event.is_null() {
        return;
    }
    with_state(data, |state| {
        // wxLayout_Default flips to RTL, RTL flips back to the default.
        state.attrs.dir = if state.attrs.dir == constants::Layout_Default() {
            constants::Layout_RightToLeft()
        } else {
            constants::Layout_Default()
        };
        setup_widget(state);
    });
}

/// `OnToggleGlobalBusyCursor`.
extern "C" fn handle_toggle_global_busy(_fun: *mut c_void, _data: *mut c_void, event: *mut c_void) {
    if event.is_null() {
        return;
    }
    // OnToggleGlobalBusyCursor: Begin on check, End on uncheck.
    let event = unsafe { CommandEvent::from_ptr(event) };
    if event.is_checked() {
        freefuncs::wxBeginBusyCursor();
    } else {
        freefuncs::wxEndBusyCursor();
    }
}

/// `OnDisableAutoComplete`.
extern "C" fn handle_autocomplete_disable(
    _fun: *mut c_void,
    data: *mut c_void,
    event: *mut c_void,
) {
    if event.is_null() {
        return;
    }
    with_state(data, |state| {
        let entry = match current_text_entry(state) {
            Some(entry) => entry,
            None => return,
        };
        if entry.auto_complete(0, std::ptr::null_mut()) {
            log_msg(state, "Disabled auto completion.");
        } else {
            log_msg(state, "AutoComplete() failed.");
        }
    });
}

/// `OnAutoCompleteFixed`.
extern "C" fn handle_autocomplete_fixed(_fun: *mut c_void, data: *mut c_void, event: *mut c_void) {
    if event.is_null() {
        return;
    }
    // wxString items[] = { "aa" .. "yy", "is this string for test?",
    // "this is a test string", "this is another test string",
    // "this string is for test" };
    let mut items: Vec<String> = ('a'..'y').map(|c| format!("{c}{c}")).collect();
    for s in [
        "is this string for test?",
        "this is a test string",
        "this is another test string",
        "this string is for test",
    ] {
        items.push(s.to_string());
    }
    let refs: Vec<&str> = items.iter().map(|s| s.as_str()).collect();
    let (_owned, ptrs) = c_strings(&refs);

    with_state(data, |state| {
        let entry = match current_text_entry(state) {
            Some(entry) => entry,
            None => return,
        };
        if entry.auto_complete(refs.len() as i32, ptrs.as_ptr() as *mut c_void) {
            log_msg(state, "Enabled auto completion of a set of fixed strings.");
        } else {
            log_msg(state, "AutoComplete() failed.");
        }
    });
}

/// `OnAutoCompleteFilenames`.
extern "C" fn handle_autocomplete_filenames(
    _fun: *mut c_void,
    data: *mut c_void,
    event: *mut c_void,
) {
    if event.is_null() {
        return;
    }
    with_state(data, |state| {
        let entry = match current_text_entry(state) {
            Some(entry) => entry,
            None => return,
        };
        if entry.auto_complete_file_names() {
            log_msg(state, "Enabled auto completion of file names.");
        } else {
            log_msg(state, "AutoCompleteFileNames() failed.");
        }
    });
}

/// `OnAutoCompleteDirectories`.
extern "C" fn handle_autocomplete_directories(
    _fun: *mut c_void,
    data: *mut c_void,
    event: *mut c_void,
) {
    if event.is_null() {
        return;
    }
    with_state(data, |state| {
        let entry = match current_text_entry(state) {
            Some(entry) => entry,
            None => return,
        };
        if entry.auto_complete_directories() {
            log_msg(state, "Enabled auto completion of directories.");
        } else {
            log_msg(state, "AutoCompleteDirectories() failed.");
        }
    });
}

/// `OnSetHint`.
extern "C" fn handle_set_hint(_fun: *mut c_void, data: *mut c_void, event: *mut c_void) {
    if event.is_null() {
        return;
    }
    let frame = with_state(data, |state| state.frame);
    let Some(frame) = frame else {
        return;
    };
    let hint = freefuncs::wxGetTextFromUserEx(
        "Text hint:",
        "Widgets sample",
        "Type here",
        &frame,
        -1,
        -1,
        1,
    );
    if hint.is_empty() {
        return;
    }
    with_state(data, |state| {
        let entry = match current_text_entry(state) {
            Some(entry) => entry,
            None => return,
        };
        if entry.set_hint(hint.as_str()) {
            log_msg(state, &format!("Set hint to \"{hint}\"."));
        } else {
            log_msg(state, "Text hints not supported.");
        }
    });
}

/// `OnTheme`: `wxTheApp->SetAppearance(...)`. The appearance values have no
/// generated constant, so the menu item reports the gap instead of guessing.
extern "C" fn handle_theme(_fun: *mut c_void, data: *mut c_void, event: *mut c_void) {
    if event.is_null() {
        return;
    }
    with_state(data, |state| {
        log_msg(state, "SetAppearance is not available in this binding.")
    });
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// `WidgetsPage::GetTextEntry()` for the pages that expose a text entry.
fn current_text_entry(state: &mut State) -> Option<TextCtrl> {
    // None of the pages ported so far overrides `GetTextEntry()` in C++
    // (`WidgetsPage::GetTextEntry()` returns nullptr), so the `&Text` menu
    // commands are disabled there by `OnUpdateTextUI`.
    let _ = state;
    None
}

/// The widgets of the currently shown page when it is of `kind`.
fn current_widgets(state: &mut State, kind: Kind) -> Option<&mut PageWidgets> {
    let index = state.current?;
    let page = state.pages.get_mut(index)?;
    if page.kind != kind {
        return None;
    }
    Some(&mut page.widgets)
}

/// Rebuilds the current page after a style change.
fn rebuild_current(state: &mut State, kind: Kind) {
    if let Some(index) = state.current {
        if state.pages[index].kind == kind {
            recreate_page(state, index);
            setup_widget(state);
        }
    }
}

/// `wxLogMessage()` into the log list box.
fn log_msg(state: &mut State, message: &str) {
    state.lbox_log.append(message);
}

/// `wxStaticBoxSizer::GetStaticBox()`.
fn static_box_parent(sizer: &StaticBoxSizer) -> StaticBox {
    sizer
        .get_static_box()
        .expect("wxStaticBoxSizer always owns a static box")
}

fn add_window_plain(sizer: &Sizer, window: &impl WxWindow) {
    sizer.add_window(window, 0, 0, 0, std::ptr::null_mut());
}

fn add_window_flags(sizer: &Sizer, window: &impl WxWindow, flags: SizerFlags) {
    sizer.add_window(
        window,
        flags.get_proportion(),
        flags.get_flags(),
        flags.get_border_in_pixels(),
        std::ptr::null_mut(),
    );
    flags.delete();
}

fn add_sizer_flags(sizer: &Sizer, child: &impl WxSizer, flags: SizerFlags) {
    sizer.add_sizer(
        child,
        flags.get_proportion(),
        flags.get_flags(),
        flags.get_border_in_pixels(),
        std::ptr::null_mut(),
    );
    flags.delete();
}

fn expand_border_prop(proportion: i32) -> SizerFlags {
    let f = SizerFlags::new(proportion);
    f.expand();
    f.border(constants::ALL());
    f
}

fn expand_border() -> SizerFlags {
    let f = SizerFlags::new(0);
    f.expand();
    f.border(constants::ALL());
    f
}

fn expand_double_border() -> SizerFlags {
    let f = SizerFlags::new(0);
    f.expand();
    f.double_border(constants::ALL());
    f
}

fn expand_double_border_prop(proportion: i32) -> SizerFlags {
    let f = SizerFlags::new(proportion);
    f.expand();
    f.double_border(constants::ALL());
    f
}

fn center_vertical_right() -> SizerFlags {
    let f = SizerFlags::new(0);
    f.center_vertical();
    f.border(constants::RIGHT());
    f
}

fn text_entry_flags() -> SizerFlags {
    let f = SizerFlags::new(1);
    f.center_vertical();
    f.border(constants::LEFT());
    f
}

/// Builds the `char**` array the kwxFFI `*_Create` / `AutoComplete` shims read.
fn c_strings(items: &[&str]) -> (Vec<CString>, Vec<*mut c_char>) {
    let owned: Vec<CString> = items
        .iter()
        .map(|s| CString::new(*s).unwrap_or_default())
        .collect();
    let ptrs: Vec<*mut c_char> = owned.iter().map(|s| s.as_ptr() as *mut c_char).collect();
    (owned, ptrs)
}

/// `WidgetsPage::CreateBitmapBundle(icon)`: a fixed-size bundle from the SVG
/// conversion of the sample's XPM.
fn load_bundle(name: &str, width: i32, height: i32) -> Option<BitmapBundle> {
    let path = format!("{ASSET_DIR}{name}.svg");
    BitmapBundle::from_svg_file(path.as_str(), Size { width, height })
}

/// `SetIcon(wxICON(sample))`.
fn load_sample_icon() -> Option<Icon> {
    let size = Size {
        width: 32,
        height: 32,
    };
    let bundle = load_bundle("sample", 32, 32)?;
    let Some(bitmap) = bundle.get_bitmap(size) else {
        bundle.delete();
        return None;
    };
    let icon = Icon::create_default();
    icon.copy_from_bitmap(&bitmap);
    bitmap.delete();
    bundle.delete();
    Some(icon)
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
