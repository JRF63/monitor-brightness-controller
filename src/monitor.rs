use std::{mem::MaybeUninit, thread, time::Duration};

use windows::{
    core::Result,
    Win32::{
        Devices::Display::{
            DestroyPhysicalMonitor, GetMonitorBrightness, GetNumberOfPhysicalMonitorsFromHMONITOR,
            GetPhysicalMonitorsFromHMONITOR, SetMonitorBrightness, PHYSICAL_MONITOR,
        },
        Foundation::{BOOL, LPARAM, RECT},
        Graphics::Gdi::{EnumDisplayMonitors, HDC, HMONITOR},
    },
};

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
    pub fn try_set_brightness(&mut self, brightness: u32) -> Result<()> {
        unsafe {
            let result = SetMonitorBrightness(
                self.physical_monitor.hPhysicalMonitor,
                brightness.clamp(self.min_brightness, self.max_brightness),
            );
            if result != 0 {
                // TODO: Maybe store brightness in Windows registry to allow persistence
                self.current_brightness = brightness;
                Ok(())
            } else {
                Err(windows::core::Error::from_win32())
            }
        }
    }

    pub fn set_brightness(&mut self, brightness: u32) -> Result<()> {
        // 10ms, 20ms, 40ms, 80ms, etc.
        let expo_backoff: [_; 8] = std::array::from_fn(|i| Duration::from_millis(10 * (1 << i)));

        // Setting the brightness sometimes fail (i.e., when it's done repeatedly without
        // sleeping). This loop retries it, waiting for increasingly long periods after
        // each failure.
        let mut result = Ok(());
        for duration in expo_backoff {
            result = self.try_set_brightness(brightness);
            if result.is_ok() {
                return Ok(());
            }
            thread::sleep(duration);
        }
        // Return last result of `try_set_brightness`
        result
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
            for physical_monitor in get_physical_monitors(monitor_handle)? {
                let mut min_brightness: MaybeUninit<u32> = MaybeUninit::uninit();
                let mut current_brightness: MaybeUninit<u32> = MaybeUninit::uninit();
                let mut max_brightness: MaybeUninit<u32> = MaybeUninit::uninit();

                let device_name = {
                    // Copy the `[u16; 128]` to a stack variable to avoid dealing with a reference to
                    // a packed struct member and be forced to use unaligned pointer reads
                    let desc = physical_monitor.szPhysicalMonitorDescription;
                    string_from_raw_utf16(&desc)
                };

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

/// Get handles to all connected monitors. The returned handles does not need to be manually freed.
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

/// Return the number of physical monitors associated with a `HMONITOR` handle. A single `HMONITOR`
/// can have multiple physical monitors when extending or duplicating displays.
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

/// Return all physical monitors of `monitor_handle`.
fn get_physical_monitors(monitor_handle: HMONITOR) -> Result<Vec<PHYSICAL_MONITOR>> {
    let num_physical_monitors = get_num_physical_monitors(monitor_handle)?;
    unsafe {
        let mut physical_monitors =
            vec![PHYSICAL_MONITOR::default(); num_physical_monitors as usize];

        let result = GetPhysicalMonitorsFromHMONITOR(monitor_handle, &mut physical_monitors);
        if result != 0 {
            Ok(physical_monitors)
        } else {
            Err(windows::core::Error::from_win32())
        }
    }
}

/// Create a `String` from a null-terminated UTF-16 string where the location of the null is not
/// known.
fn string_from_raw_utf16(array: &[u16; 128]) -> String {
    let mut zero_loc = 0;
    for (i, &v) in array.iter().enumerate() {
        zero_loc = i;
        if v == 0 {
            break;
        }
    }
    String::from_utf16_lossy(&array[..zero_loc])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_brightness() {
        use std::{thread, time};

        let duration = time::Duration::from_secs(5);

        let mut monitors = Monitor::get_monitors().unwrap();
        let mut brightnesses = Vec::new();
        for monitor in &mut monitors {
            brightnesses.push(monitor.get_brightness());
        }

        for monitor in &mut monitors {
            monitor.try_set_brightness(0).unwrap();
        }

        thread::sleep(duration);

        for monitor in &mut monitors {
            monitor.try_set_brightness(100).unwrap();
        }

        thread::sleep(duration);

        for (monitor, &brightness) in monitors.iter_mut().zip(brightnesses.iter()) {
            monitor.try_set_brightness(brightness).unwrap();
        }
    }
}
