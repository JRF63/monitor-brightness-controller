use windows::{
    core::Result,
    Win32::{
        Foundation::{HANDLE, HWND},
        System::{Power::{
            RegisterPowerSettingNotification, UnregisterPowerSettingNotification, HPOWERNOTIFY,
        }, SystemServices::GUID_CONSOLE_DISPLAY_STATE},
    },
};

pub struct PowerNotifyHandle(HPOWERNOTIFY);

impl Drop for PowerNotifyHandle {
    fn drop(&mut self) {
        unsafe {
            if !UnregisterPowerSettingNotification(self.0).as_bool() {
                panic!("{:?}", windows::core::Error::from_win32());
            }
        }
    }
}

pub fn register_for_power_notification(hwnd: HWND) -> Result<PowerNotifyHandle> {
    unsafe {
        let handle =
            RegisterPowerSettingNotification(HANDLE(hwnd.0), &GUID_CONSOLE_DISPLAY_STATE, 0)?;
        Ok(PowerNotifyHandle(handle))
    }
}
