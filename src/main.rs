use windows::core::{Result, GUID};
use windows::Win32::Foundation::*;
use windows::Win32::Graphics::Gdi::*;
use windows::Win32::System::LibraryLoader::GetModuleHandleA;
use windows::Win32::System::WinRT::{RoInitialize, RO_INIT_SINGLETHREADED};
use windows::Win32::UI::Controls::RichEdit::WM_CONTEXTMENU;
use windows::Win32::UI::Shell::NIN_SELECT;
use windows::Win32::UI::WindowsAndMessaging::*;

mod icon;
mod monitor;
mod resources;
mod xaml;

#[cfg(debug_assertions)]
const ICON_GUID: GUID = GUID::from_u128(0xe8bd1019_8a41_421d_bb5c_7b4ade6bbd9b);

#[cfg(not(debug_assertions))]
const ICON_GUID: GUID = GUID::from_u128(0x58eac4e5_34c3_49bb_90b6_fd86751edbad);

const MAIN_WINDOW_CLASS: PSTR = PSTR(b"SoftwareBrightness\0".as_ptr() as *mut u8);
const EMPTY_STRING: PSTR = PSTR(b"\0".as_ptr() as *mut u8);
const WMAPP_NOTIFYCALLBACK: u32 = WM_APP + 1;
const WINDOW_WIDTH: i32 = 360;
const WINDOW_HEIGHT: i32 = 100;

static mut MONITORS: Vec<monitor::Monitor> = Vec::new();

pub unsafe extern "system" fn window_procedure(
    hwnd: HWND,
    umsg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match umsg {
        // WM_CREATE => {
        //     DefWindowProcA(hwnd, umsg, wparam, lparam)
        // }
        WM_DESTROY => {
            icon::delete_icon();
            PostQuitMessage(0);
            LRESULT(0)
        }
        WMAPP_NOTIFYCALLBACK => {
            let loword = lparam.0 as u32 & 0xffff;
            match loword {
                NIN_SELECT => {
                    // super::toggle_window(hwnd);
                }
                WM_CONTEXTMENU => {
                    // TODO: create context menu
                    SendMessageA(hwnd, WM_CLOSE, WPARAM(0), LPARAM(0));
                }
                _ => (),
            }
            LRESULT(0)
        }
        _ => DefWindowProcA(hwnd, umsg, wparam, lparam),
    }
}

fn create_window() -> Result<HWND> {
    let instance = unsafe { GetModuleHandleA(PSTR::default()) };
    let cursor = unsafe { LoadCursorW(None, IDC_ARROW) };
    let wcex = WNDCLASSEXA {
        cbSize: std::mem::size_of::<WNDCLASSEXA>() as u32,
        style: CS_HREDRAW | CS_VREDRAW,
        lpfnWndProc: Some(window_procedure),
        hInstance: instance,
        // hIcon: meow
        hCursor: cursor,
        lpszMenuName: EMPTY_STRING,
        lpszClassName: MAIN_WINDOW_CLASS,
        // hIconSm: meow
        ..Default::default()
    };
    unsafe {
        if RegisterClassExA(&wcex) == 0 {
            return Err(windows::core::Error::from_win32());
        }
    }

    let window = unsafe {
        CreateWindowExA(
            WS_EX_NOREDIRECTIONBITMAP | WS_EX_TOOLWINDOW,
            MAIN_WINDOW_CLASS,
            EMPTY_STRING,
            WS_POPUP | WS_VISIBLE,
            360 + 62,
            980,
            WINDOW_WIDTH,
            WINDOW_HEIGHT,
            None,
            None,
            instance,
            std::ptr::null(),
        )
    };
    if window.0 == 0 {
        return Err(windows::core::Error::from_win32());
    }

    icon::add_icon(window)?;
    Ok(window)
}

fn set_monitor_brightness(index: usize, brightness: u32) {
    let expo_backoff = [10, 20, 40, 80, 160];
    if let Some(monitor) = unsafe { MONITORS.get_mut(index) } {
        for duration in expo_backoff {
            if monitor.set_brightness(brightness).is_ok() {
                break;
            } else {
                std::thread::sleep(std::time::Duration::from_millis(duration));
            }
        }
    }
}

fn main() -> Result<()> {
    let window = create_window()?;

    unsafe {
        RoInitialize(RO_INIT_SINGLETHREADED)?;
    }

    let xaml_controls = xaml::XamlControls::new(window)?;
    unsafe {
        SetWindowPos(
            xaml_controls.window(),
            HWND(0),
            0,
            0,
            360,
            100,
            SWP_SHOWWINDOW,
        );
        ShowWindow(window, SW_SHOW);
        UpdateWindow(window);
    }

    unsafe {
        MONITORS = monitor::Monitor::get_monitors()?;
    }
    if let Some(monitor) = unsafe { MONITORS.first() } {
        let brightness = monitor.get_brightness();
        icon::modify_icon_tooltip(brightness)?;
    }

    xaml_controls.slider_value_changed(|_caller, args| {
        if let Some(args) = args {
            let brightness = args.NewValue()? as u32;
            // set_monitor_brightness(0, brightness);
            icon::modify_icon_tooltip(brightness)?;
        }
        Ok(())
    })?;

    let mut msg = MSG::default();
    unsafe {
        while GetMessageA(&mut msg, HWND::default(), 0, 0).as_bool() {
            if !xaml_controls.filter_message(&msg) {
                TranslateMessage(&msg);
                DispatchMessageA(&msg);
            }
        }
    }

    Ok(())
}
