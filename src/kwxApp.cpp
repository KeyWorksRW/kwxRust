/*
 * kwxApp - C interface for wxWidgets application for kwxRust
 *
 * Adapted from kwxFFI examples/CApp/kwxApp.cpp with Rust-specific exports.
 */

#include "kwx_wrapper.h"

#include <wx/cmdline.h>
#include <wx/tooltip.h>
#include <wx/wx.h>

#include <cstdlib>
#include <cstring>

/*-----------------------------------------------------------------------------
    Export macro for Rust FFI
-----------------------------------------------------------------------------*/
#ifdef _WIN32
#define RUST_EXPORT extern "C" __declspec(dllexport)
#else
#define RUST_EXPORT extern "C" __attribute__((visibility("default")))
#endif

/*-----------------------------------------------------------------------------
    Internal state
-----------------------------------------------------------------------------*/
namespace
{
    bool g_initialized = false;
    bool g_terminating = false;

    // Idle timer support
    typedef void (*kwxApp_IdleCallback)(void *data);
    kwxApp_IdleCallback g_idleCallback = nullptr;
    void *g_idleCallbackData = nullptr;

    // Cached strings for Get functions (to avoid returning dangling pointers)
    wxString g_cachedAppName;
    wxString g_cachedVendorName;

    // Helper to convert wxString to C string (caller must free)
    char *wxStringToNewCStr(const wxString &str)
    {
        const char *utf8 = str.utf8_str();
        size_t len = strlen(utf8) + 1;
        char *result = static_cast<char *>(malloc(len));
        if (result)
        {
            memcpy(result, utf8, len);
        }
        return result;
    }
} // namespace

/*-----------------------------------------------------------------------------
    Idle Timer Class
-----------------------------------------------------------------------------*/
#define KWXAPP_MIN_IDLE_INTERVAL_MS 100

class kwxIdleTimer : public wxTimer
{
public:
    void Notify() override
    {
        if (g_idleCallback)
        {
            g_idleCallback(g_idleCallbackData);
        }
    }
};

static kwxIdleTimer *g_idleTimer = nullptr;

/*-----------------------------------------------------------------------------
    The wxApp subclass (hidden from Rust callers)
-----------------------------------------------------------------------------*/
class kwxAppImpl : public wxApp
{
public:
    bool OnInit() override
    {
        if (!wxApp::OnInit())
        {
            return false;
        }
        g_idleTimer = new kwxIdleTimer();
        return true;
    }

    int OnExit() override
    {
        g_terminating = true;
        if (g_idleTimer)
        {
            g_idleTimer->Stop();
            delete g_idleTimer;
            g_idleTimer = nullptr;
        }
        return wxApp::OnExit();
    }

    // Event handler that routes through closure system
    void HandleEvent(wxEvent &evt)
    {
        wxCallback *callback = (wxCallback *)(evt.m_callbackUserData);
        if (callback)
        {
            callback->Invoke(&evt);
        }
    }

    // Prevent wxApp from failing on foreign language command line args
    void OnInitCmdLine(wxCmdLineParser &parser) override { parser.SetCmdLine(""); }

    bool OnCmdLineParsed(wxCmdLineParser &) override { return true; }
};

// This is the actual wxApp instance
wxIMPLEMENT_APP_NO_MAIN(kwxAppImpl);

/*-----------------------------------------------------------------------------
    C Interface Implementation
-----------------------------------------------------------------------------*/
extern "C"
{
    /*-------------------------------------------------------------------------
        Application Lifecycle
    -------------------------------------------------------------------------*/
    RUST_EXPORT int kwxApp_Initialize(int argc, char **argv)
    {
        if (g_initialized)
        {
            return 1; // Already initialized
        }

        // wxWidgets requires valid argc/argv
        static char *dummy_argv[] = {const_cast<char *>("kwxRust"), nullptr};
        if (argc <= 0 || argv == nullptr)
        {
            argc = 1;
            argv = dummy_argv;
        }

        if (!wxEntryStart(argc, argv))
        {
            return 0;
        }

        if (!wxTheApp || !wxTheApp->CallOnInit())
        {
            wxEntryCleanup();
            return 0;
        }

        g_initialized = true;
        return 1;
    }

    RUST_EXPORT int kwxApp_MainLoop()
    {
        if (!g_initialized || !wxTheApp)
        {
            return -1;
        }
        return wxTheApp->OnRun();
    }

    RUST_EXPORT void kwxApp_ExitMainLoop()
    {
        if (wxTheApp)
        {
            wxTheApp->ExitMainLoop();
        }
    }

    RUST_EXPORT void kwxApp_Shutdown()
    {
        if (g_initialized)
        {
            g_terminating = true;
            if (wxTheApp)
            {
                wxTheApp->OnExit();
            }
            wxEntryCleanup();
            g_initialized = false;
        }
    }

    RUST_EXPORT int kwxApp_IsTerminating()
    {
        return g_terminating ? 1 : 0;
    }

    /*-------------------------------------------------------------------------
        Application Properties
    -------------------------------------------------------------------------*/
    RUST_EXPORT const char *kwxApp_GetAppName()
    {
        if (!wxTheApp)
            return "";
        g_cachedAppName = wxTheApp->GetAppName();
        return g_cachedAppName.utf8_str();
    }

    RUST_EXPORT void kwxApp_SetAppName(const char *name)
    {
        if (wxTheApp && name)
        {
            wxTheApp->SetAppName(wxString::FromUTF8(name));
        }
    }

    RUST_EXPORT const char *kwxApp_GetVendorName()
    {
        if (!wxTheApp)
            return "";
        g_cachedVendorName = wxTheApp->GetVendorName();
        return g_cachedVendorName.utf8_str();
    }

    RUST_EXPORT void kwxApp_SetVendorName(const char *name)
    {
        if (wxTheApp && name)
        {
            wxTheApp->SetVendorName(wxString::FromUTF8(name));
        }
    }

    RUST_EXPORT void *kwxApp_GetTopWindow()
    {
        return wxTheApp ? wxTheApp->GetTopWindow() : nullptr;
    }

    RUST_EXPORT void kwxApp_SetTopWindow(void *window)
    {
        if (wxTheApp)
        {
            wxTheApp->SetTopWindow(static_cast<wxWindow *>(window));
        }
    }

    RUST_EXPORT void kwxApp_SetExitOnFrameDelete(int flag)
    {
        if (wxTheApp)
        {
            wxTheApp->SetExitOnFrameDelete(flag != 0);
        }
    }

    RUST_EXPORT int kwxApp_GetExitOnFrameDelete()
    {
        return (wxTheApp && wxTheApp->GetExitOnFrameDelete()) ? 1 : 0;
    }

    /*-------------------------------------------------------------------------
        System Information
    -------------------------------------------------------------------------*/
    RUST_EXPORT void kwxApp_GetDisplaySize(int *width, int *height)
    {
        wxSize size = wxGetDisplaySize();
        if (width)
            *width = size.GetWidth();
        if (height)
            *height = size.GetHeight();
    }

    RUST_EXPORT void kwxApp_GetMousePosition(int *x, int *y)
    {
        wxPoint pos = wxGetMousePosition();
        if (x)
            *x = pos.x;
        if (y)
            *y = pos.y;
    }

    RUST_EXPORT int kwxApp_GetOsVersion(int *major, int *minor)
    {
        return wxGetOsVersion(major, minor);
    }

    RUST_EXPORT char *kwxApp_GetOsDescription()
    {
        return wxStringToNewCStr(wxGetOsDescription());
    }

    RUST_EXPORT char *kwxApp_GetUserId()
    {
        return wxStringToNewCStr(wxGetUserId());
    }

    RUST_EXPORT char *kwxApp_GetUserName()
    {
        return wxStringToNewCStr(wxGetUserName());
    }

    /*-------------------------------------------------------------------------
        Event Processing
    -------------------------------------------------------------------------*/
    RUST_EXPORT int kwxApp_Pending()
    {
        return (wxTheApp && wxTheApp->Pending()) ? 1 : 0;
    }

    RUST_EXPORT void kwxApp_Dispatch()
    {
        if (wxTheApp)
        {
            wxTheApp->Dispatch();
        }
    }

    RUST_EXPORT int kwxApp_Yield()
    {
        return wxYield() ? 1 : 0;
    }

    RUST_EXPORT int kwxApp_SafeYield(void *window)
    {
        return wxSafeYield(static_cast<wxWindow *>(window)) ? 1 : 0;
    }

    /*-------------------------------------------------------------------------
        Event Connection (Rust-specific: uses wxClosure directly)
    -------------------------------------------------------------------------*/
    RUST_EXPORT int kwxApp_ConnectClosure(void *obj, int first, int last, int event_type,
                                          void *closure)
    {
        wxCallback *callback = new wxCallback(static_cast<wxClosure *>(closure));
        static_cast<wxEvtHandler *>(obj)->Connect(
            first, last, event_type,
            (wxObjectEventFunction)&kwxAppImpl::HandleEvent,
            callback);
        return 0;
    }

    RUST_EXPORT int kwxApp_Disconnect(void *obj, int first, int last, int event_type,
                                      void *data)
    {
        return static_cast<wxEvtHandler *>(obj)->Disconnect(
                   first, last, event_type,
                   (wxObjectEventFunction)&kwxAppImpl::HandleEvent,
                   static_cast<wxObject *>(data))
                   ? 1
                   : 0;
    }

    /*-------------------------------------------------------------------------
        Idle Timer
    -------------------------------------------------------------------------*/
    RUST_EXPORT void kwxApp_SetIdleCallback(int interval_ms, void (*callback_func)(void *),
                                            void *callback_data)
    {
        g_idleCallback = callback_func;
        g_idleCallbackData = callback_data;

        if (g_idleTimer)
        {
            if (g_idleTimer->IsRunning())
            {
                g_idleTimer->Stop();
            }

            if (callback_func && interval_ms >= KWXAPP_MIN_IDLE_INTERVAL_MS)
            {
                g_idleTimer->Start(interval_ms, false);
            }
        }
    }

    RUST_EXPORT int kwxApp_GetIdleInterval()
    {
        if (g_idleTimer && g_idleTimer->IsRunning())
        {
            return g_idleTimer->GetInterval();
        }
        return 0;
    }

    /*-------------------------------------------------------------------------
        Utilities
    -------------------------------------------------------------------------*/
    RUST_EXPORT void kwxApp_InitAllImageHandlers()
    {
        wxInitAllImageHandlers();
    }

    RUST_EXPORT void kwxApp_EnableTooltips(int enable)
    {
        wxToolTip::Enable(enable != 0);
    }

    RUST_EXPORT void kwxApp_SetTooltipDelay(int milliseconds)
    {
        wxToolTip::SetDelay(milliseconds);
    }

    RUST_EXPORT void kwxApp_Bell()
    {
        wxBell();
    }

    RUST_EXPORT void kwxApp_Sleep(int seconds)
    {
        wxSleep(seconds);
    }

    RUST_EXPORT void kwxApp_MilliSleep(int milliseconds)
    {
        wxMilliSleep(milliseconds);
    }

    RUST_EXPORT void *kwxApp_FindWindowById(int id, void *parent)
    {
        return wxWindow::FindWindowById(static_cast<long>(id), static_cast<wxWindow *>(parent));
    }

    RUST_EXPORT void *kwxApp_FindWindowByLabel(const char *label, void *parent)
    {
        if (!label)
            return nullptr;
        return wxFindWindowByLabel(wxString::FromUTF8(label), static_cast<wxWindow *>(parent));
    }

    RUST_EXPORT void *kwxApp_FindWindowByName(const char *name, void *parent)
    {
        if (!name)
            return nullptr;
        return wxFindWindowByName(wxString::FromUTF8(name), static_cast<wxWindow *>(parent));
    }

    RUST_EXPORT void kwxApp_FreeString(char *str)
    {
        free(str);
    }

} // extern "C"
