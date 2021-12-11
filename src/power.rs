use windows::core::Result;
use windows::Win32::Foundation::{HANDLE, HWND};
use windows::Win32::System::Power::{
    RegisterPowerSettingNotification, UnregisterPowerSettingNotification, HPOWERNOTIFY,
};
use windows::Win32::System::SystemServices::GUID_CONSOLE_DISPLAY_STATE;

pub fn register_for_power_notification(hwnd: HWND) -> Result<HPOWERNOTIFY> {
    unsafe {
        let handle =
            RegisterPowerSettingNotification(HANDLE(hwnd.0), &GUID_CONSOLE_DISPLAY_STATE, 0);
        if handle.0 != 0 {
            Ok(handle)
        } else {
            Err(windows::core::Error::from_win32())
        }
    }
}

pub fn unregister_for_power_notification(power_notify_handle: HPOWERNOTIFY) -> Result<()> {
    unsafe {
        if UnregisterPowerSettingNotification(power_notify_handle).as_bool() {
            Ok(())
        } else {
            Err(windows::core::Error::from_win32())
        }
    }
}
