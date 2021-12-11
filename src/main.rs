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
use std::sync::mpsc::{channel, Sender, TryRecvError};
use std::thread;
use std::time::Duration;

mod icon;
mod monitor;
mod power;
mod resources;
mod xaml;

#[cfg(debug_assertions)]
const ICON_GUID: GUID = GUID::from_u128(0xe8bd1019_8a41_421d_bb5c_7b4ade6bbd9b);

#[cfg(not(debug_assertions))]
const ICON_GUID: GUID = GUID::from_u128(0x58eac4e5_34c3_49bb_90b6_fd86751edbad);

const MAIN_WINDOW_CLASS: PSTR = PSTR(b"SoftwareBrightness\0".as_ptr() as *mut u8);
const EMPTY_STRING: PSTR = PSTR(b"\0".as_ptr() as *mut u8);
const WMAPP_NOTIFYCALLBACK: u32 = WM_APP + 1;
const TIMER_LOST_FOCUS: usize = 2;
const TIMER_BRIGHTNESS_RESET: usize = 3;

enum ChannelData {
    Set(usize, u32),
    Refresh,
    Quit,
}

unsafe extern "system" fn delayed_brightness_reset(
    hwnd: HWND,
    _umsg: u32,
    _idtimer: usize,
    _dwtime: u32,
) {
    KillTimer(hwnd, TIMER_BRIGHTNESS_RESET);
    let sender = get_window_proc_data(&hwnd);
    sender
        .send(ChannelData::Refresh)
        .unwrap_or(( /*ignore err*/ ));
}

unsafe extern "system" fn window_procedure(
    hwnd: HWND,
    umsg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
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
            if timer_id == TIMER_LOST_FOCUS {
                KillTimer(hwnd, TIMER_LOST_FOCUS);
                LOST_FOCUS = false;
            }
            LRESULT(0)
        }
        WM_DESTROY => {
            let sender = get_window_proc_data(&hwnd);
            sender.send(ChannelData::Quit).unwrap_or(( /*ignore err*/ ));
            PostQuitMessage(0);
            LRESULT(0)
        }
        WMAPP_NOTIFYCALLBACK => {
            let loword = lparam.0 as u32 & 0xffff;
            match loword {
                NIN_SELECT => {
                    if !LOST_FOCUS {
                        if IsWindowVisible(hwnd).as_bool() {
                            ShowWindow(hwnd, SW_HIDE);
                        } else {
                            ShowWindow(hwnd, SW_SHOW);
                            SetForegroundWindow(hwnd);
                        }
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
                            SetTimer(
                                hwnd,
                                TIMER_BRIGHTNESS_RESET,
                                6000,
                                Some(delayed_brightness_reset),
                            );
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
        style: CS_HREDRAW | CS_VREDRAW,
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
    if window.0 == 0 {
        return Err(windows::core::Error::from_win32());
    }
    Ok(window)
}

fn set_window_proc_data(hwnd: HWND, data: &Sender<ChannelData>) {
    unsafe {
        SetWindowLongPtrA(
            hwnd,
            GWLP_USERDATA,
            data as *const Sender<ChannelData> as isize,
        );
    }
}

fn get_window_proc_data<'a>(hwnd: &'a HWND) -> &'a Sender<ChannelData> {
    unsafe {
        let ptr = GetWindowLongPtrA(*hwnd, GWLP_USERDATA) as *const Sender<ChannelData>;
        &*ptr
    }
}

fn num_to_hstring(num: u32) -> HSTRING {
    let mut buf: [u8; 11] = [0; 11];
    write!(&mut buf[..], "{}", num).unwrap();
    let s = std::str::from_utf8(&buf).unwrap();
    HSTRING::from(s)
}

fn main() -> Result<()> {
    let width = 360;
    let height = 100;
    let (x, y) = window_position(width, height);
    let window = create_window(x, y, width, height)?;
    icon::add_icon(window)?;
    let power_notify_handle = power::register_for_power_notification(window)?;

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
        icon::modify_icon_tooltip(brightness)?;
        monitor_name.SetText(HSTRING::from(monitor.get_name()))?;
        brightness_number.SetText(num_to_hstring(brightness))?;
        slider.SetValue2(brightness as f64)?;
    }

    let (sender, receiver) = channel::<ChannelData>();
    let window_proc_sender = sender.clone();

    set_window_proc_data(window, &window_proc_sender);

    let thread_handle = thread::spawn(move || -> Result<()> {
        let mut monitors = monitors;
        let mut last_brightness: Vec<u32> = monitors.iter().map(|m| m.get_brightness()).collect();

        fn parse_msg_and_continue(
            receiver: &std::sync::mpsc::Receiver<ChannelData>,
            msg: ChannelData,
            monitors: &mut Vec<monitor::Monitor>,
            last_brightness: &mut Vec<u32>,
            recurse: bool,
        ) -> bool {
            match msg {
                ChannelData::Set(index, brightness) => {
                    if let Some(val) = last_brightness.get_mut(index) {
                        *val = brightness;
                    }
                    if recurse {
                        let recv = receiver.try_recv();
                        loop {
                            match recv {
                                Ok(msg) => {
                                    return parse_msg_and_continue(
                                        receiver,
                                        msg,
                                        monitors,
                                        last_brightness,
                                        false,
                                    )
                                }
                                Err(TryRecvError::Empty) => break,
                                Err(TryRecvError::Disconnected) => return false,
                            }
                        }

                        let expo_backoff = [10, 20, 40, 80, 160];
                        for (monitor, brightness) in monitors.iter_mut().zip(last_brightness.iter())
                        {
                            if monitor.get_brightness() != *brightness {
                                for duration in expo_backoff {
                                    if monitor.set_brightness(*brightness).is_ok() {
                                        break;
                                    }
                                    thread::sleep(Duration::from_millis(duration));
                                }
                            }
                        }
                    }
                    true
                }
                ChannelData::Refresh => {
                    for monitor in monitors.iter_mut() {
                        let brightness = monitor.get_brightness();
                        while monitor.set_brightness(brightness).is_err() {}
                    }
                    true
                }
                ChannelData::Quit => false,
            }
        }

        while let Ok(msg) = receiver.recv() {
            if !parse_msg_and_continue(&receiver, msg, &mut monitors, &mut last_brightness, true) {
                break;
            }
        }
        Ok(())
    });

    let slider_callback = xaml::XamlControls::create_slider_callback(move |_caller, args| {
        if let Some(args) = args {
            let brightness = args.NewValue()? as u32;
            sender
                .send(ChannelData::Set(0, brightness))
                .unwrap_or(( /*ignore err*/ ));
            brightness_number.SetText(num_to_hstring(brightness))?;
            icon::modify_icon_tooltip(brightness)?;
        }
        Ok(())
    });
    let event_token = slider.ValueChanged(slider_callback)?;

    let mut msg = MSG::default();
    unsafe {
        while GetMessageA(&mut msg, HWND::default(), 0, 0).as_bool() {
            if !xaml_controls.filter_message(&msg) {
                TranslateMessage(&msg);
                DispatchMessageA(&msg);
            }
        }
    }

    thread_handle.join().unwrap().unwrap();
    slider.RemoveValueChanged(event_token)?;
    power::unregister_for_power_notification(power_notify_handle)?;
    icon::delete_icon();
    Ok(())
}
