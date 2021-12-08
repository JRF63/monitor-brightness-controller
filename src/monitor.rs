use windows::core::Result;
use windows::Win32::Devices::Display::{
    DestroyPhysicalMonitor, GetMonitorBrightness, GetNumberOfPhysicalMonitorsFromHMONITOR,
    GetPhysicalMonitorsFromHMONITOR, SetMonitorBrightness, PHYSICAL_MONITOR,
};
use windows::Win32::Foundation::*;
use windows::Win32::Graphics::Gdi::{
    DISPLAY_DEVICEA, EnumDisplayMonitors, GetMonitorInfoA, HDC, HMONITOR, MONITORINFO, MONITORINFOEXA, EnumDisplayDevicesA
};

use std::mem::MaybeUninit;
use std::ffi::CStr;

pub struct Monitor {
    physical_monitor: PHYSICAL_MONITOR,
    device_name: String,
    min_brightness: u32,
    current_brightness: u32,
    max_brightness: u32,
}

impl Drop for Monitor {
    fn drop(&mut self) {
        unsafe {
            DestroyPhysicalMonitor(self.physical_monitor.hPhysicalMonitor);
        }
    }
}

impl Monitor {
    pub fn set_brightness(&mut self, brightness: u32) -> Result<()> {
        unsafe {
            let result = SetMonitorBrightness(
                self.physical_monitor.hPhysicalMonitor,
                brightness.clamp(self.min_brightness, self.max_brightness),
            );
            if result != 0 {
                self.current_brightness = brightness;
                // TODO: save to registry?
                Ok(())
            } else {
                Err(windows::core::Error::from_win32())
            }
        }
    }

    pub fn get_brightness(&self) -> u32 {
        self.current_brightness
    }

    pub fn get_name(&self) -> &str {
        &self.device_name
    }

    pub fn get_monitors() -> Result<Vec<Monitor>> {
        let mut monitors = Vec::new();
        let monitor_handles = get_monitor_handles()?;
        for &monitor_handle in &monitor_handles {
            let device_name = get_monitor_name(monitor_handle);
            for physical_monitor in get_physical_monitors(monitor_handle)? {
                let mut min_brightness: MaybeUninit<u32> = MaybeUninit::uninit();
                let mut current_brightness: MaybeUninit<u32> = MaybeUninit::uninit();
                let mut max_brightness: MaybeUninit<u32> = MaybeUninit::uninit();

                unsafe {
                    let result = GetMonitorBrightness(
                        physical_monitor.hPhysicalMonitor,
                        min_brightness.as_mut_ptr(),
                        current_brightness.as_mut_ptr(),
                        max_brightness.as_mut_ptr(),
                    );

                    let monitor = Monitor {
                        physical_monitor,
                        device_name: device_name.clone(),
                        min_brightness: min_brightness.assume_init(),
                        current_brightness: current_brightness.assume_init(),
                        max_brightness: max_brightness.assume_init(),
                    };

                    if result != 0 {
                        monitors.push(monitor);
                    }
                };
            }
        }
        Ok(monitors)
    }
}

fn get_monitor_handles() -> Result<Vec<HMONITOR>> {
    unsafe extern "system" fn callback(
        monitor_handle: HMONITOR,
        _: HDC,
        _: *mut RECT,
        param: LPARAM,
    ) -> BOOL {
        let monitors: &mut Vec<HMONITOR> = &mut *(param.0 as *mut Vec<HMONITOR>);
        monitors.push(monitor_handle);
        BOOL(1)
    }

    let mut monitors: Vec<HMONITOR> = Vec::new();
    let result = unsafe {
        EnumDisplayMonitors(
            HDC::default(),
            std::ptr::null(),
            Some(callback),
            LPARAM(&mut monitors as *mut _ as isize),
        )
    };
    if result.as_bool() {
        Ok(monitors)
    } else {
        Err(windows::core::Error::from_win32())
    }
}

fn get_num_physical_monitors(monitor_handle: HMONITOR) -> Result<u32> {
    let mut num_physical_monitors: MaybeUninit<u32> = MaybeUninit::uninit();
    unsafe {
        let result = GetNumberOfPhysicalMonitorsFromHMONITOR(
            monitor_handle,
            num_physical_monitors.as_mut_ptr(),
        );
        if result != 0 {
            Ok(num_physical_monitors.assume_init())
        } else {
            Err(windows::core::Error::from_win32())
        }
    }
}

fn get_physical_monitors(monitor_handle: HMONITOR) -> Result<Vec<PHYSICAL_MONITOR>> {
    let num_physical_monitors = get_num_physical_monitors(monitor_handle)?;
    unsafe {
        let mut physical_monitors = Vec::new();
        physical_monitors.reserve(num_physical_monitors as usize);
        physical_monitors.set_len(num_physical_monitors as usize);

        let result = GetPhysicalMonitorsFromHMONITOR(
            monitor_handle,
            num_physical_monitors,
            physical_monitors.as_mut_ptr(),
        );
        if result != 0 {
            Ok(physical_monitors)
        } else {
            Err(windows::core::Error::from_win32())
        }
    }
}

fn get_monitor_name(monitor_handle: HMONITOR) -> String {
    let mut info = MONITORINFOEXA {
        __AnonymousBase_winuser_L13567_C43: MONITORINFO {
            cbSize: std::mem::size_of::<MONITORINFOEXA>() as u32,
            ..Default::default()
        },
        ..Default::default()
    };
    let mut device = DISPLAY_DEVICEA {
        cb: std::mem::size_of::<DISPLAY_DEVICEA>() as u32,
        ..Default::default()
    };
    unsafe {
        GetMonitorInfoA(monitor_handle, &mut info as *mut MONITORINFOEXA as _);
        EnumDisplayDevicesA(
            PSTR(info.szDevice.as_ptr() as _),
            0,
            &mut device,
            0
        );
        let slice = CStr::from_ptr(
            device.DeviceString.as_ptr() as _
        );
        slice.to_str().unwrap().to_string()
    }
}

#[test]
fn test_brightness() {
    use std::{thread, time};

    let duration = time::Duration::from_secs(5);

    let mut monitors = Monitor::get_monitors().unwrap();
    let mut brightnesses = Vec::new();
    for monitor in &mut monitors {
        brightnesses.push(monitor.get_brightness());
    }

    for monitor in &mut monitors {
        monitor.set_brightness(0).unwrap();
    }

    thread::sleep(duration);

    for monitor in &mut monitors {
        monitor.set_brightness(100).unwrap();
    }

    thread::sleep(duration);

    for (monitor, &brightness) in monitors.iter_mut().zip(brightnesses.iter()) {
        monitor.set_brightness(brightness).unwrap();
    }
}
