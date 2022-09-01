use windows::core::{Result, PCSTR};
use windows::Win32::Foundation::*;
use windows::Win32::System::LibraryLoader::GetModuleHandleA;
use windows::Win32::UI::Shell::*;
use windows::Win32::UI::WindowsAndMessaging::*;

use std::io::Write;

const ICON_RESOURCE: PCSTR = PCSTR(201 as *mut u8);

pub struct NotificationIcon(NOTIFYICONDATAA);

impl Drop for NotificationIcon {
    fn drop(&mut self) {
        self.0.uFlags = NIF_GUID;
        unsafe {
            if !Shell_NotifyIconA(NIM_DELETE, &self.0).as_bool() {
                panic!("{:?}", windows::core::Error::from_win32());
            }
        }
    }
}

pub fn create_notification_icon(window: HWND) -> Result<NotificationIcon> {
    let icon = unsafe {
        LoadImageA(
            GetModuleHandleA(PCSTR::default())?,
            ICON_RESOURCE,
            IMAGE_ICON,
            0,
            0,
            LR_DEFAULTSIZE,
        )
    }?;
    let icon = HICON(icon.0);
    if icon.0 == 0 {
        return Err(windows::core::Error::from_win32());
    }

    let mut nid = NOTIFYICONDATAA {
        cbSize: std::mem::size_of::<NOTIFYICONDATAA>() as u32,
        hWnd: window,
        uFlags: NIF_ICON | NIF_MESSAGE | NIF_GUID,
        uCallbackMessage: crate::WM_APP_NOTIFYCALLBACK,
        hIcon: icon,
        Anonymous: NOTIFYICONDATAA_0 {
            uVersion: NOTIFYICON_VERSION_4,
        },
        guidItem: crate::ICON_GUID,
        ..Default::default()
    };

    unsafe {
        if Shell_NotifyIconA(NIM_ADD, &nid).as_bool() {
            if Shell_NotifyIconA(NIM_SETVERSION, &nid).as_bool() {
                // Set flags for `modify_tooltip`
                nid.uFlags = NIF_TIP | NIF_SHOWTIP | NIF_GUID;
                return Ok(NotificationIcon(nid));
            }
        }
    }
    Err(windows::core::Error::from_win32())
}

impl NotificationIcon {
    pub fn modify_tooltip(&mut self, brightness: u32) -> Result<()> {
        let data: &mut [u8; 128] = unsafe {
            // SAFETY: CHAR is just a #[repr(transparent)] u8
            std::mem::transmute(&mut self.0.szTip)
        };
        if write!(data.as_mut_slice(), "Brightness: {}\0", brightness).is_err() {
            return Err(windows::core::Error::from_win32());
        }

        unsafe {
            if Shell_NotifyIconA(NIM_MODIFY, &self.0).as_bool() {
                Ok(())
            } else {
                Err(windows::core::Error::from_win32())
            }
        }
    }
}
