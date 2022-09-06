use std::{marker::PhantomData, ops::Deref, sync::mpsc::Sender};

use windows::{
    core::{Result, PCSTR},
    Win32::{
        Foundation::{HWND, LPARAM, LRESULT, WPARAM},
        System::{LibraryLoader::GetModuleHandleA, Power::POWERBROADCAST_SETTING},
        UI::{
            Shell::{
                SHAppBarMessage, ABE_BOTTOM, ABE_LEFT, ABE_RIGHT, ABE_TOP, ABM_GETTASKBARPOS,
                APPBARDATA, NIN_SELECT,
            },
            WindowsAndMessaging::{
                CreateWindowExA, DefWindowProcA, GetWindowLongPtrA, KillTimer, LoadCursorW,
                PostQuitMessage, RegisterClassExA, SendMessageA, SetForegroundWindow, SetTimer,
                SetWindowLongPtrA, SetWindowPos, ShowWindow, CS_DROPSHADOW, GWLP_USERDATA,
                IDC_ARROW, PBT_POWERSETTINGCHANGE, SWP_SHOWWINDOW, SW_HIDE, WM_ACTIVATEAPP,
                WM_CLOSE, WM_CONTEXTMENU, WM_DESTROY, WM_POWERBROADCAST, WM_TIMER, WNDCLASSEXA,
                WS_EX_NOREDIRECTIONBITMAP, WS_EX_TOOLWINDOW, WS_POPUP,
            },
        },
    },
    UI::Xaml::Controls::ListBox,
};

use crate::{BrightnessEvent, NotificationIcon};

/// Calculate the position where the window would be shown. This should be near where the controls
/// for sound, Wi-Fi, etc.
pub fn window_position(width: i32, height: i32) -> (i32, i32) {
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
            _ => (), // Unknown value; fallthrough the panic
        }
    }
    panic!("Could not get taskbar position")
}

/// Wrapper class for a `HWND`. Indirectly owns a reference to a `Sender<BrightnessEvent>`
pub struct Window<'a> {
    inner: HWND,
    sender: PhantomData<&'a Sender<BrightnessEvent>>,
}

impl<'a> Window<'a> {
    pub const WIDTH: i32 = 360;
    pub const HEIGHT: i32 = 100;

    /// Create a native window that acts as a container for XAML.
    pub fn new(sender: &'a Sender<BrightnessEvent>) -> Result<Self> {
        /// Handles the window events. A function inside a function does not allow the inner
        /// function to access the outer functions variables; this is only placed here to emphasize
        /// that this should only be used inside `Window::new`.
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

                            // SAFETY: `Window` is just a `HWND` with a lifetime
                            let window: Window = std::mem::transmute(hwnd);
                            let _ = window.send(BrightnessEvent::Reset);
                        }
                        _ => (),
                    }
                    LRESULT(0)
                }
                WM_DESTROY => {
                    PostQuitMessage(0);
                    LRESULT(0)
                }
                NotificationIcon::MESSAGE => {
                    let loword = lparam.0 as u32 & 0xffff;
                    match loword {
                        // left clicked
                        NIN_SELECT => {
                            if !LOST_FOCUS {
                                // Recalculate the position in case the taskbar position was
                                // changed
                                let (x, y) = window_position(Window::WIDTH, Window::HEIGHT);
                                SetWindowPos(
                                    hwnd,
                                    HWND(0),
                                    x,
                                    y,
                                    Window::WIDTH,
                                    Window::HEIGHT,
                                    SWP_SHOWWINDOW,
                                );
                                SetForegroundWindow(hwnd);
                            }
                        }
                        // right clicked
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

        const MAIN_WINDOW_CLASS: PCSTR =
            PCSTR(b"MonitorBrightnessController\0".as_ptr() as *mut u8);

        let instance = unsafe { GetModuleHandleA(PCSTR::default())? };
        let cursor = unsafe { LoadCursorW(None, IDC_ARROW)? };

        let wcex = WNDCLASSEXA {
            cbSize: std::mem::size_of::<WNDCLASSEXA>() as u32,
            style: CS_DROPSHADOW,
            lpfnWndProc: Some(window_procedure),
            hInstance: instance,
            hCursor: cursor,
            lpszMenuName: PCSTR::default(),
            lpszClassName: MAIN_WINDOW_CLASS,
            ..Default::default()
        };
        unsafe {
            if RegisterClassExA(&wcex) == 0 {
                return Err(windows::core::Error::from_win32());
            }
        }

        let (x, y) = window_position(Self::WIDTH, Self::HEIGHT);

        let hwnd = unsafe {
            CreateWindowExA(
                WS_EX_NOREDIRECTIONBITMAP | WS_EX_TOOLWINDOW,
                MAIN_WINDOW_CLASS,
                None,
                WS_POPUP,
                x,
                y,
                Self::WIDTH,
                Self::HEIGHT,
                None,
                None,
                instance,
                std::ptr::null(),
            )
        };
        if hwnd.0 != 0 {
            // SAFETY: This stores a `&Sender<BrightnessEvent>` to the `HWND` which can later be
            // referenced through `GetWindowLongPtrA`. The `PhantomData` member of `Window` ensures
            // that it does not exceed the lifetime of the `&Sender<BrightnessEvent>` argument.
            unsafe {
                SetWindowLongPtrA(
                    hwnd,
                    GWLP_USERDATA,
                    sender as *const Sender<BrightnessEvent> as isize,
                );
            }
            Ok(Window {
                inner: hwnd,
                sender: PhantomData,
            })
        } else {
            Err(windows::core::Error::from_win32())
        }
    }

    /// Cast the `Window` as a raw `HWND` that can be used for interacting with Windows API.
    pub fn as_handle(&self) -> HWND {
        self.inner
    }
}

struct WindowData {
    sender: Sender<BrightnessEvent>,
    list_box: ListBox
}

impl<'a> Deref for Window<'a> {
    type Target = Sender<BrightnessEvent>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        // SAFETY: Gets the `*const Sender<BrightnessEvent>` that was stored in `Window::new`.
        // The `Sender` should be valid throughout the lifetime of this `Window` because we
        // required `PhantomData<&'a Sender<BrightnessEvent>>` in its type signature.
        unsafe {
            let ptr = GetWindowLongPtrA(self.inner, GWLP_USERDATA);
            &*(ptr as *const Sender<BrightnessEvent>)
        }
    }
}
