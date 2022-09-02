#![windows_subsystem = "windows"]

mod guid;
mod icon;
mod monitor;
mod power;
mod window;
mod xaml;

use std::{
    io::Write,
    sync::mpsc::{self, Receiver, TryRecvError},
    thread,
    time::Duration,
};

use windows::{
    core::{Result, HSTRING},
    Win32::{
        Foundation::HWND,
        System::WinRT::{RoInitialize, RO_INIT_SINGLETHREADED},
        UI::WindowsAndMessaging::{DispatchMessageA, GetMessageA, TranslateMessage, MSG},
    },
    UI::Xaml::Controls::Primitives::RangeBase,
};

use guid::ICON_GUID;
use icon::NotificationIcon;
use monitor::Monitor;
use power::PowerNotifyHandle;
use window::Window;

pub enum BrightnessEvent {
    Change(usize, u32),
    Reset,
}

/// Event loop that handles directly setting the brightness of the monitors. Should be used in a
/// separate thread since setting the brightness can stall the GUI.
#[inline]
fn brightness_controller_loop(mut monitors: Vec<Monitor>, rx: Receiver<BrightnessEvent>) {
    let mut brightness_vals = monitors
        .iter()
        .map(|m| m.get_brightness())
        .collect::<Vec<_>>();

    'outer: loop {
        let mut msg = match rx.recv() {
            Ok(msg) => msg,
            Err(_) => break,
        };

        // Once a message is received, repeatedly `try_recv` until there is no more.
        // This is done so that it will not try to set the brightness one by one for each
        // value sent by the callback.
        loop {
            match msg {
                BrightnessEvent::Change(i, brightness) => {
                    brightness_vals[i] = brightness;
                    msg = match rx.try_recv() {
                        Ok(msg) => msg,
                        Err(TryRecvError::Empty) => break,
                        Err(TryRecvError::Disconnected) => break 'outer,
                    }
                }
                BrightnessEvent::Reset => break,
            }
        }

        for (monitor, brightness) in monitors.iter_mut().zip(brightness_vals.iter()) {
            let expo_backoff = [
                Duration::from_millis(10),
                Duration::from_millis(20),
                Duration::from_millis(40),
                Duration::from_millis(80),
                Duration::from_millis(160),
            ];

            // Setting the brightness sometimes fail (i.e., when it's done repeatedly without
            // sleeping). This loop retries it, waiting for increasingly long periods after
            // each failure.
            for duration in expo_backoff {
                if monitor.set_brightness(*brightness).is_ok() {
                    break;
                }
                thread::sleep(duration);
            }
        }
    }
}

/// Helper function for creating a `HSTRING` from an integer.
fn num_to_hstring(num: u32) -> HSTRING {
    let mut buf: [u8; 11] = [0; 11];
    write!(&mut buf[..], "{}", num).unwrap();
    let s = std::str::from_utf8(&buf).unwrap();
    HSTRING::from(s)
}

fn main() -> Result<()> {
    // Initialize WinRT
    unsafe {
        RoInitialize(RO_INIT_SINGLETHREADED)?;
    }

    let (tx, rx) = mpsc::channel();
    let tx1 = tx.clone();
    let tx2 = tx;

    let window = Window::new(&tx2)?;
    let mut notification_icon = NotificationIcon::new(window.as_handle())?;
    let _power_notify_handle = PowerNotifyHandle::new(window.as_handle())?;

    let xaml_controls = xaml::XamlControls::new(&window)?;
    let monitor_name = xaml_controls.monitor_name()?;
    let brightness_number = xaml_controls.brightness_number()?;
    let slider = xaml_controls.slider()?;

    let monitors = monitor::Monitor::get_monitors()?;

    if let Some(monitor) = monitors.first() {
        let brightness = monitor.get_brightness();
        notification_icon.modify_tooltip(brightness)?;
        monitor_name.SetText(HSTRING::from(monitor.get_name()))?;
        brightness_number.SetText(num_to_hstring(brightness))?;
        RangeBase::from(&slider).SetValue(brightness as f64)?;
    }

    thread::spawn(move || {
        brightness_controller_loop(monitors, rx);
    });

    // Purposely ignoring the returned event token; It shouldn't be necessary to manually free the
    // callback
    RangeBase::from(&slider).ValueChanged(xaml::XamlControls::create_slider_callback(
        move |_caller, args| {
            // TODO: Handle multiple monitors.
            // Slider's ValueChanged callback is run on the main thread
            if let Some(args) = args {
                const MONITOR_INDEX: usize = 0;
                let brightness = args.NewValue()? as u32;
                let _ = tx1.send(BrightnessEvent::Change(MONITOR_INDEX, brightness));

                brightness_number.SetText(num_to_hstring(brightness))?;
                notification_icon.modify_tooltip(brightness)?;
            }
            Ok(())
        },
    ))?;

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
