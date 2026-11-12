//! Port of the wxWidgets `minimal` sample (`C:/rwCode/wxSamples/src/minimal/minimal.cpp`).
//!
//! Faithful to the C++ original:
//! - `MyFrame` with the title `"Minimal wxWidgets App"`, default position/size and
//!   `wxDEFAULT_FRAME_STYLE` (the C++ constructor passes neither position nor size).
//! - `File -> Exit` (`E&xit\tAlt-X`, help `"Quit this program"`) and
//!   `Help -> About` (`&About\tF1`, help `"Show about dialog"`).
//! - `EVT_MENU(Minimal_Quit)` -> `Close(true)`, `EVT_MENU(Minimal_About)` -> the
//!   `wxMessageBox` about dialog, parented to the frame.
//! - `CreateStatusBar(2)` followed by `SetStatusText("Welcome to wxWidgets!")` in pane 0.
//!
//! Known gap: `SetIcon(wxICON(sample))` is not reproduced. On MSW the icon comes
//! from the sample's `.rc` resource, and this port ships no resource script; the
//! frame therefore uses the default wxWidgets icon.

// Use the Windows GUI subsystem (no console window), as the C++ sample is GUI-only.
#![windows_subsystem = "windows"]

use kwxrust::prelude::*;

/// Menu command ID for `Minimal_Quit = wxID_EXIT`, resolved at runtime.
fn minimal_quit() -> i32 {
    kwxrust::sys::wx_id_exit()
}

/// Menu command ID for `Minimal_About = wxID_ABOUT`, resolved at runtime.
fn minimal_about() -> i32 {
    kwxrust::sys::wx_id_about()
}

fn main() {
    // MyApp::OnInit() -> wxApp::OnInit()
    app::init();

    // MyFrame::MyFrame(title) : wxFrame(nullptr, wxID_ANY, title)
    let frame = Frame::builder("Minimal wxWidgets App").build();

    app::set_top_window(&frame);

    // #if wxUSE_MENUBAR
    //
    // wxMenu *fileMenu = new wxMenu;
    // wxMenu *helpMenu = new wxMenu;
    // helpMenu->Append(Minimal_About, "&About\tF1", "Show about dialog");
    // fileMenu->Append(Minimal_Quit, "E&xit\tAlt-X", "Quit this program");
    let file_menu = Menu::new();
    let help_menu = Menu::new();
    help_menu.append(minimal_about(), "&About\tF1", "Show about dialog");
    file_menu.append(minimal_quit(), "E&xit\tAlt-X", "Quit this program");

    // wxMenuBar *menuBar = new wxMenuBar();
    // menuBar->Append(fileMenu, "&File");
    // menuBar->Append(helpMenu, "&Help");
    // SetMenuBar(menuBar);
    let menu_bar = MenuBar::new();
    menu_bar.append(&file_menu, "&File");
    menu_bar.append(&help_menu, "&Help");
    frame.set_menu_bar(&menu_bar);

    // wxBEGIN_EVENT_TABLE(MyFrame, wxFrame)
    //     EVT_MENU(Minimal_Quit,  MyFrame::OnQuit)
    //     EVT_MENU(Minimal_About, MyFrame::OnAbout)
    // wxEND_EVENT_TABLE()
    let menu_selected = unsafe { kwxrust::sys::exp_wxEVT_MENU() };
    event::bind(&frame, minimal_quit(), menu_selected, |_evt| on_quit()).forget();
    event::bind(&frame, minimal_about(), menu_selected, |_evt| on_about()).forget();

    // #if wxUSE_STATUSBAR : CreateStatusBar(2); SetStatusText("Welcome to wxWidgets!");
    frame.create_status_bar(2);
    frame.set_status_text("Welcome to wxWidgets!", 0);

    // frame->Show(true)
    frame.show();

    let exit_code = app::run();
    app::shutdown();
    std::process::exit(exit_code);
}

/// `MyFrame::OnQuit` — `Close(true)`: close the frame and force the close.
fn on_quit() {
    app::close_top_window(true);
}

/// `MyFrame::OnAbout`
///
/// ```cpp
/// wxMessageBox(wxString::Format("Welcome to %s!\n\nThis is the minimal wxWidgets sample\n"
///                               "running under %s.",
///                               wxVERSION_STRING, wxGetOsDescription()),
///              "About wxWidgets minimal sample", wxOK | wxICON_INFORMATION, this);
/// ```
fn on_about() {
    let message = format!(
        "Welcome to {}!\n\nThis is the minimal wxWidgets sample\nrunning under {}.",
        app::version_string(),
        app::os_description()
    );

    dialog::info_modal(&message, "About wxWidgets minimal sample");
}
