#![windows_subsystem = "windows"]

mod guid;
mod icon;
mod monitor;
mod power;
mod window;
mod xaml;

use std::{
    sync::mpsc::{self, Receiver, TryRecvError},
    thread,
};

use windows::{
    core::Result,
    Win32::{
        Foundation::HWND,
        System::WinRT::{RoInitialize, RO_INIT_SINGLETHREADED},
        UI::WindowsAndMessaging::{DispatchMessageA, GetMessageA, TranslateMessage, MSG},
    },
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

    'outer: while let Ok(mut msg) = rx.recv() {
        // Once a message is received, repeatedly `try_recv` until there is no more.
        // This is done so that it will not try to set the brightness one by one for each
        // value sent by the callback.
        while let BrightnessEvent::Change(i, brightness) = msg {
            brightness_vals[i] = brightness;
            msg = match rx.try_recv() {
                Ok(msg) => msg,
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => break 'outer,
            }
        }

        for (monitor, brightness) in monitors.iter_mut().zip(brightness_vals.iter()) {
            let _ = monitor.set_brightness(*brightness);
        }
    }
}

fn main() -> Result<()> {
    // Initialize WinRT
    unsafe {
        RoInitialize(RO_INIT_SINGLETHREADED)?;
    }

    let (tx, rx) = mpsc::channel();
    let tx1 = tx.clone();
    let tx2 = tx;

    let monitors = monitor::Monitor::get_monitors()?;

    let window = Window::new(&tx1)?;
    let mut notification_icon = NotificationIcon::new(window.as_handle())?;
    let _power_notify_handle = PowerNotifyHandle::new(window.as_handle())?;

    if let Some(monitor) = monitors.first() {
        let brightness = monitor.get_brightness();
        notification_icon.modify_tooltip(brightness)?;
    }

    let xaml_controls = xaml::XamlControls::new(&window, &monitors, tx2, notification_icon)?;

    thread::spawn(move || {
        brightness_controller_loop(monitors, rx);
    });

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
