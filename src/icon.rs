use windows::core::Result;
use windows::Win32::Foundation::*;
use windows::Win32::System::LibraryLoader::GetModuleHandleA;
use windows::Win32::UI::Shell::*;
use windows::Win32::UI::WindowsAndMessaging::*;

use std::io::Write;

pub fn add_icon(window: HWND) -> Result<()> {
    let icon = unsafe {
        LoadImageA(
            GetModuleHandleA(PSTR::default()),
            crate::resources::ICON_RESOURCE,
            IMAGE_ICON,
            0,
            0,
            LR_DEFAULTSIZE,
        )
    };
    let icon = HICON(icon.0);
    if icon.0 == 0 {
        return Err(windows::core::Error::from_win32());
    }

    let nid = NOTIFYICONDATAA {
        cbSize: std::mem::size_of::<NOTIFYICONDATAA>() as u32,
        hWnd: window,
        uFlags: NIF_ICON | NIF_MESSAGE | NIF_GUID,
        uCallbackMessage: crate::WMAPP_NOTIFYCALLBACK,
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
                return Ok(());
            }
        }
    }
    Err(windows::core::Error::from_win32())
}

pub fn delete_icon() {
    let nid = NOTIFYICONDATAA {
        cbSize: std::mem::size_of::<NOTIFYICONDATAA>() as u32,
        uFlags: NIF_GUID,
        guidItem: crate::ICON_GUID,
        ..Default::default()
    };
    unsafe {
        Shell_NotifyIconA(NIM_DELETE, &nid);
    }
}

pub fn modify_icon_tooltip(brightness: u32) -> Result<()> {
    let mut nid = NOTIFYICONDATAA {
        cbSize: std::mem::size_of::<NOTIFYICONDATAA>() as u32,
        uFlags: NIF_TIP | NIF_SHOWTIP | NIF_GUID,
        guidItem: crate::ICON_GUID,
        ..Default::default()
    };

    let data: &mut [u8; 128] = unsafe {
        // SAFETY: CHAR is just a #[repr(transparent)] u8
        std::mem::transmute(&mut nid.szTip)
    };
    if write!(data.as_mut_slice(), "Brightness: {}\0", brightness).is_err() {
        return Err(windows::core::Error::from_win32());
    }

    unsafe {
        if Shell_NotifyIconA(NIM_MODIFY, &nid).as_bool() {
            Ok(())
        } else {
            Err(windows::core::Error::from_win32())
        }
    }
}
