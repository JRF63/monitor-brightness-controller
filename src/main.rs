#![windows_subsystem = "windows"]

use windows::core::{Result, GUID, HSTRING};
use windows::Win32::Foundation::*;
use windows::Win32::Graphics::Gdi::*;
use windows::Win32::System::LibraryLoader::GetModuleHandleA;
use windows::Win32::System::Power::POWERBROADCAST_SETTING;
use windows::Win32::System::WinRT::{RoInitialize, RO_INIT_SINGLETHREADED};
use windows::Win32::UI::Controls::RichEdit::WM_CONTEXTMENU;
use windows::Win32::UI::Shell::*;
use windows::Win32::UI::WindowsAndMessaging::*;

use std::io::Write;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::thread::{self, Thread};
use std::time::Duration;

mod icon;
mod monitor;
mod power;
mod xaml;

#[cfg(debug_assertions)]
const ICON_GUID: GUID = GUID::from_u128(0xe8bd1019_8a41_421d_bb5c_7b4ade6bbd9b);

#[cfg(not(debug_assertions))]
const ICON_GUID: GUID = GUID::from_u128(0x58eac4e5_34c3_49bb_90b6_fd86751edbad);

const MAIN_WINDOW_CLASS: PSTR = PSTR(b"SoftwareBrightness\0".as_ptr() as *mut u8);
const EMPTY_STRING: PSTR = PSTR(b"\0".as_ptr() as *mut u8);
const WM_APP_NOTIFYCALLBACK: u32 = WM_APP + 1;

static UNPARKED: AtomicBool = AtomicBool::new(false);

unsafe extern "system" fn window_procedure(
    hwnd: HWND,
    umsg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    const TIMER_LOST_FOCUS: usize = 2;
    const TIMER_BRIGHTNESS_RESET: usize = 3;

    static mut LOST_FOCUS: bool = false;
    static mut MONITOR_TURNED_OFF: bool = false;

    match umsg {
        WM_ACTIVATEAPP => {
            if wparam.0 == 0 {
                ShowWindow(hwnd, SW_HIDE);
                SetTimer(hwnd, TIMER_LOST_FOCUS, 200, None);
                LOST_FOCUS = true;
            }
            LRESULT(0)
        }
        WM_TIMER => {
            let timer_id = wparam.0;
            match timer_id {
                TIMER_LOST_FOCUS => {
                    KillTimer(hwnd, TIMER_LOST_FOCUS);
                    LOST_FOCUS = false;
                }
                TIMER_BRIGHTNESS_RESET => {
                    KillTimer(hwnd, TIMER_BRIGHTNESS_RESET);
                    let thread = get_window_proc_data(&hwnd);
                    UNPARKED.store(true, Ordering::Relaxed);
                    thread.unpark();
                }
                _ => (),
            }
            LRESULT(0)
        }
        WM_DESTROY => {
            PostQuitMessage(0);
            LRESULT(0)
        }
        WM_APP_NOTIFYCALLBACK => {
            let loword = lparam.0 as u32 & 0xffff;
            match loword {
                NIN_SELECT => {
                    if !LOST_FOCUS {
                        ShowWindow(hwnd, SW_SHOW);
                        SetForegroundWindow(hwnd);
                    }
                }
                WM_CONTEXTMENU => {
                    // TODO: create context menu
                    SendMessageA(hwnd, WM_CLOSE, WPARAM(0), LPARAM(0));
                }
                _ => (),
            }
            LRESULT(0)
        }
        WM_POWERBROADCAST => {
            if wparam.0 as u32 == PBT_POWERSETTINGCHANGE {
                const OFF: u8 = 0;
                const ON: u8 = 1;
                const DIMMED: u8 = 2;

                let setting = &*(lparam.0 as *const POWERBROADCAST_SETTING);
                let monitor_state = setting.Data[0];
                match monitor_state {
                    OFF | DIMMED => {
                        MONITOR_TURNED_OFF = true;
                    }
                    ON => {
                        if MONITOR_TURNED_OFF {
                            MONITOR_TURNED_OFF = false;
                            SetTimer(hwnd, TIMER_BRIGHTNESS_RESET, 5000, None);
                        }
                    }
                    _ => (),
                }
            }
            LRESULT(1)
        }
        _ => DefWindowProcA(hwnd, umsg, wparam, lparam),
    }
}

fn window_position(width: i32, height: i32) -> (i32, i32) {
    let mut pabd = APPBARDATA {
        cbSize: std::mem::size_of::<APPBARDATA>() as u32,
        ..Default::default()
    };
    let ret = unsafe { SHAppBarMessage(ABM_GETTASKBARPOS, &mut pabd) };
    if ret != 0 {
        match pabd.uEdge {
            ABE_BOTTOM => return (pabd.rc.right - width, pabd.rc.top - height),
            ABE_LEFT => return (pabd.rc.right, pabd.rc.bottom - height),
            ABE_RIGHT => return (pabd.rc.left - width, pabd.rc.bottom - height),
            ABE_TOP => return (pabd.rc.right - width, pabd.rc.bottom),
            _ => (),
        }
    }
    (0, 0)
}

fn create_window(x: i32, y: i32, width: i32, height: i32) -> Result<HWND> {
    let instance = unsafe { GetModuleHandleA(PSTR::default()) };
    let cursor = unsafe { LoadCursorW(None, IDC_ARROW) };
    let wcex = WNDCLASSEXA {
        cbSize: std::mem::size_of::<WNDCLASSEXA>() as u32,
        style: CS_DROPSHADOW,
        lpfnWndProc: Some(window_procedure),
        hInstance: instance,
        hCursor: cursor,
        lpszMenuName: EMPTY_STRING,
        lpszClassName: MAIN_WINDOW_CLASS,
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
            WS_POPUP,
            x,
            y,
            width,
            height,
            None,
            None,
            instance,
            std::ptr::null(),
        )
    };
    if window.0 != 0 {
        Ok(window)
    } else {
        Err(windows::core::Error::from_win32())
    }
}

fn num_to_hstring(num: u32) -> HSTRING {
    let mut buf: [u8; 11] = [0; 11];
    write!(&mut buf[..], "{}", num).unwrap();
    let s = std::str::from_utf8(&buf).unwrap();
    HSTRING::from(s)
}

fn set_window_proc_data(hwnd: HWND, data: &Thread) {
    unsafe {
        SetWindowLongPtrA(hwnd, GWLP_USERDATA, data as *const Thread as isize);
    }
}

fn get_window_proc_data<'a>(hwnd: &'a HWND) -> &'a Thread {
    unsafe {
        let ptr = GetWindowLongPtrA(*hwnd, GWLP_USERDATA) as *const Thread;
        &*ptr
    }
}

fn main() -> Result<()> {
    // Initialize WinRT
    unsafe {
        RoInitialize(RO_INIT_SINGLETHREADED)?;
    }

    let width = 360;
    let height = 100;
    let (x, y) = window_position(width, height);
    let window = create_window(x, y, width, height)?;
    let mut notification_icon = icon::create_notification_icon(window)?;
    let _power_notify_handle = power::register_for_power_notification(window)?;

    let xaml_controls = xaml::XamlControls::new(window)?;
    unsafe {
        SetWindowPos(
            xaml_controls.window(),
            HWND(0),
            0,
            0,
            width,
            height,
            SWP_SHOWWINDOW,
        );
        UpdateWindow(window);
    }
    let monitor_name = xaml_controls.monitor_name()?;
    let brightness_number = xaml_controls.brightness_number()?;
    let slider = xaml_controls.slider()?;

    let monitors = monitor::Monitor::get_monitors()?;
    
    if let Some(monitor) = monitors.first() {
        let brightness = monitor.get_brightness();
        notification_icon.modify_tooltip(brightness)?;
        monitor_name.SetText(HSTRING::from(monitor.get_name()))?;
        brightness_number.SetText(num_to_hstring(brightness))?;
        slider.SetValue2(brightness as f64)?;
    }

    static QUIT: AtomicBool = AtomicBool::new(false);
    static mut DATA: Vec<AtomicU32> = Vec::new();

    for monitor in &monitors {
        unsafe {
            DATA.push(AtomicU32::new(monitor.get_brightness()));
        }
    }

    let join_handle = thread::spawn(move || {
        let mut monitors = monitors;

        while !QUIT.load(Ordering::Relaxed) {
            UNPARKED.store(false, Ordering::Relaxed);
            thread::park();
            if UNPARKED.load(Ordering::Acquire) {
                let expo_backoff = [10, 20, 40, 80, 160];
                for (monitor, atomic_val) in monitors.iter_mut().zip(unsafe { DATA.iter() }) {
                    let mut brightness = atomic_val.load(Ordering::Acquire);
                    loop {
                        for duration in expo_backoff {
                            if monitor.set_brightness(brightness).is_ok() {
                                break;
                            }
                            thread::sleep(Duration::from_millis(duration));
                        }
                        brightness = atomic_val.load(Ordering::Acquire);
                        if brightness == monitor.get_brightness() {
                            break;
                        }
                    }
                }
            }
        }
    });

    let worker_thread = join_handle.thread().clone();
    let worker_thread_clone = worker_thread.clone();

    let event_token = slider.ValueChanged(xaml::XamlControls::create_slider_callback(
        move |_caller, args| {
            // Slider's ValueChanged callback is run on the main thread
            if let Some(args) = args {
                const MONITOR_INDEX: usize = 0;
                let brightness = args.NewValue()? as u32;
                unsafe {
                    DATA[MONITOR_INDEX].store(brightness, Ordering::Release);
                }
                UNPARKED.store(true, Ordering::Release);
                worker_thread.unpark();
                brightness_number.SetText(num_to_hstring(brightness))?;
                notification_icon.modify_tooltip(brightness)?;
            }
            Ok(())
        },
    ))?;

    set_window_proc_data(window, &worker_thread_clone);

    let mut msg = MSG::default();
    unsafe {
        while GetMessageA(&mut msg, HWND::default(), 0, 0).as_bool() {
            if !xaml_controls.filter_message(&msg) {
                TranslateMessage(&msg);
                DispatchMessageA(&msg);
            }
        }
    }

    QUIT.store(true, Ordering::Relaxed);
    worker_thread_clone.unpark();
    join_handle.join().unwrap();
    slider.RemoveValueChanged(event_token)?;
    Ok(())
}
