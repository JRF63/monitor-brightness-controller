mod image;

use std::{io::Write, marker::PhantomData, sync::mpsc::Sender};

use windows::{
    core::{Interface, Result, HSTRING},
    Win32::{
        Foundation::{BOOL, HWND},
        System::WinRT::Xaml::{IDesktopWindowXamlSourceNative, IDesktopWindowXamlSourceNative2},
        UI::WindowsAndMessaging::{SetWindowPos, MSG, SWP_SHOWWINDOW},
    },
    UI::{
        Text::FontWeights,
        Xaml::{
            Controls::{
                Button, ContentControl, Control, ItemsControl, ListBox, Orientation, Panel,
                Primitives::{ButtonBase, RangeBase, RangeBaseValueChangedEventHandler, Selector},
                SelectionMode, Slider, StackPanel, TextBlock,
            },
            FrameworkElement, HorizontalAlignment,
            Hosting::{DesktopWindowXamlSource, WindowsXamlManager},
            Media::{AcrylicBackgroundSource, AcrylicBrush},
            RoutedEventHandler, TextAlignment, Thickness, UIElement, VerticalAlignment, Visibility,
        },
    },
};

use crate::{window::window_position, BrightnessEvent, Monitor, NotificationIcon, Window};

pub struct XamlControls<'a> {
    manager: WindowsXamlManager,
    source: IDesktopWindowXamlSourceNative2,
    parent: PhantomData<&'a HWND>,
}

impl<'a> Drop for XamlControls<'a> {
    fn drop(&mut self) {
        if let Ok(source) = self.source.cast::<DesktopWindowXamlSource>() {
            source.Close().unwrap();
        }
        self.manager.Close().unwrap();
    }
}

impl<'a> XamlControls<'a> {
    const TEXTBLOCK_FONT_SIZE: f64 = 15.0;
    const TEXTBLOCK_PADDING: Thickness = Thickness {
        Left: 6.0,
        Top: 6.0,
        Right: 6.0,
        Bottom: 6.0,
    };
    const SELECTOR_HEIGHT: i32 = 45;
    const CONTROLS_HEIGHT: i32 = Window::HEIGHT - XamlControls::SELECTOR_HEIGHT;
    const SLIDER_WIDTH: i32 = 232;
    const SLIDER_HEIGHT: i32 = 28;
    const BRIGHTNESS_TEXT_FONT_SIZE: f64 = 23.5;

    pub fn new(
        parent: &'a Window,
        monitors: &[Monitor],
        tx: Sender<BrightnessEvent>,
        notification_icon: NotificationIcon,
    ) -> Result<Self> {
        let manager = WindowsXamlManager::InitializeForCurrentThread()?;
        let xaml_source = DesktopWindowXamlSource::new()?;
        let interop: IDesktopWindowXamlSourceNative = xaml_source.cast()?;
        let window = unsafe {
            interop.AttachToWindow(parent.as_handle())?;
            interop.WindowHandle()?
        };

        let controls = XamlControls::create_controls(
            window,
            parent.as_handle(),
            monitors,
            tx,
            notification_icon,
        )?;
        xaml_source.SetContent(&controls)?;
        let source: IDesktopWindowXamlSourceNative2 = xaml_source.cast()?;

        // Sets the XAML window's position on its parent
        unsafe {
            SetWindowPos(
                window,
                HWND(0),
                0,
                0,
                Window::WIDTH,
                Window::HEIGHT,
                SWP_SHOWWINDOW,
            );
        }

        Ok(XamlControls {
            manager,
            source,
            parent: PhantomData,
        })
    }

    /// Intercept Windows message events. Used in a `GetMessage` loop.
    pub fn filter_message(&self, message: *const MSG) -> bool {
        let mut processed = BOOL(0);
        unsafe {
            if let Ok(_) = self.source.PreTranslateMessage(message, &mut processed) {
                return processed.as_bool();
            }
        }
        false
    }

    /// Builds the XAML controls.
    fn create_controls(
        window: HWND,
        parent: HWND,
        monitors: &[Monitor],
        tx: Sender<BrightnessEvent>,
        notification_icon: NotificationIcon,
    ) -> Result<StackPanel> {
        let brush = AcrylicBrush::new()?;
        brush.SetBackgroundSource(AcrylicBackgroundSource::HostBackdrop)?;
        brush.SetTintColor(windows::UI::Colors::Black()?)?;

        let xaml_container = StackPanel::new()?;
        Panel::from(&xaml_container).SetBackground(brush.clone())?;

        let selected_monitor = monitors.first().unwrap();
        let init_brightness = selected_monitor.get_brightness();

        let button = create_selector(&brush, selected_monitor.get_name())?;
        let list_box = create_selector_choices(monitors)?;
        let slider_container = create_slider_control(
            &brush,
            list_box.clone(),
            init_brightness,
            tx,
            notification_icon,
        )?;

        set_button_click_event(window, parent, &button, list_box.clone())?;

        Panel::from(&xaml_container).Children()?.Append(button)?;
        Panel::from(&xaml_container).Children()?.Append(list_box)?;
        Panel::from(&xaml_container)
            .Children()?
            .Append(slider_container)?;
        UIElement::from(&xaml_container).UpdateLayout()?;

        Ok(xaml_container)
    }
}

/// Monitor selector at the top of the window.
fn create_selector(brush: &AcrylicBrush, init_text: &str) -> Result<Button> {
    let button = Button::new()?;
    FrameworkElement::from(&button).SetWidth(Window::WIDTH as f64)?;
    FrameworkElement::from(&button).SetHeight(XamlControls::SELECTOR_HEIGHT as f64)?;

    let text_block = TextBlock::new()?;
    text_block.SetPadding(XamlControls::TEXTBLOCK_PADDING)?;
    text_block.SetFontSize(XamlControls::TEXTBLOCK_FONT_SIZE)?;
    text_block.SetText(HSTRING::from(init_text))?;

    ContentControl::from(&button).SetContent(text_block)?;
    Control::from(&button).SetBackground(brush)?;
    Control::from(&button).SetBorderThickness(Thickness::default())?; // Disable border
    Control::from(&button).SetHorizontalContentAlignment(HorizontalAlignment::Left)?;
    Ok(button)
}

/// Selection of monitors; initially hidden.
fn create_selector_choices(monitors: &[Monitor]) -> Result<ListBox> {
    let list_box = ListBox::new()?;
    // Because there is only one slider control
    list_box.SetSelectionMode(SelectionMode::Single)?;

    let items = ItemsControl::from(&list_box).Items()?;
    for monitor in monitors {
        let text_block = TextBlock::new()?;
        text_block.SetPadding(XamlControls::TEXTBLOCK_PADDING)?;
        text_block.SetFontSize(XamlControls::TEXTBLOCK_FONT_SIZE)?;
        text_block.SetText(HSTRING::from(monitor.get_name()))?;
        items.Append(text_block)?;
    }
    if monitors.len() > 0 {
        Selector::from(&list_box).SetSelectedIndex(0)?;
    }
    let num_items = items.Size()? as i32;
    let height = XamlControls::SELECTOR_HEIGHT * num_items;
    FrameworkElement::from(&list_box).SetHeight(height as f64)?;
    // Hide the `ListBox`
    UIElement::from(&list_box).SetVisibility(Visibility::Collapsed)?;
    Ok(list_box)
}

/// Consists of a brightness icon, a slider, and a text for the currently selected monitor's
/// brightness.
fn create_slider_control(
    brush: &AcrylicBrush,
    list_box: ListBox,
    init_brightness: u32,
    tx: Sender<BrightnessEvent>,
    mut notification_icon: NotificationIcon,
) -> Result<StackPanel> {
    let slider_container = StackPanel::new()?;
    Panel::from(&slider_container).SetBackground(brush)?;
    slider_container.SetOrientation(Orientation::Horizontal)?;
    FrameworkElement::from(&slider_container).SetHeight(XamlControls::CONTROLS_HEIGHT as f64)?;

    let width = (Window::WIDTH - XamlControls::SLIDER_WIDTH) as f64 / 2.0;

    let image = image::create_image()?;
    FrameworkElement::from(&image).SetWidth(width)?;
    FrameworkElement::from(&image).SetMargin(Thickness {
        Left: 0.0,
        Top: 4.0,
        Right: 0.0,
        Bottom: 0.0,
    })?;

    let brightness_number = TextBlock::new()?;
    FrameworkElement::from(&brightness_number).SetWidth(width)?;
    brightness_number.SetTextAlignment(TextAlignment::Center)?;
    FrameworkElement::from(&brightness_number).SetVerticalAlignment(VerticalAlignment::Center)?;
    brightness_number.SetFontSize(XamlControls::BRIGHTNESS_TEXT_FONT_SIZE)?;
    brightness_number.SetText(num_to_hstring(init_brightness))?;

    let slider = Slider::new()?;
    FrameworkElement::from(&slider).SetWidth(XamlControls::SLIDER_WIDTH as f64)?;
    FrameworkElement::from(&slider).SetHeight(XamlControls::SLIDER_HEIGHT as f64)?;
    RangeBase::from(&slider).SetMaximum(100.0)?;
    RangeBase::from(&slider).SetMinimum(0.0)?;
    RangeBase::from(&slider).SetValue(init_brightness as f64)?;

    let brightness_number_clone = brightness_number.clone();

    // `Slider::ValueChanged` callback is run on the main/UI thread. This should return immediately
    // to prevent GUI lagging hence the use of a separate thread to update the monitor brightness
    RangeBase::from(&slider).ValueChanged(RangeBaseValueChangedEventHandler::new(
        move |_caller, args| {
            if let Some(args) = args {
                let index = Selector::from(&list_box).SelectedIndex()? as usize;
                let brightness = args.NewValue()? as u32;
                let _ = tx.send(BrightnessEvent::Change(index, brightness));

                brightness_number_clone.SetText(num_to_hstring(brightness))?;
                notification_icon.modify_tooltip(brightness)?;
            }
            Ok(())
        },
    ))?;

    Panel::from(&slider_container).Children()?.Append(image)?;
    Panel::from(&slider_container).Children()?.Append(slider)?;
    Panel::from(&slider_container)
        .Children()?
        .Append(brightness_number)?;
    Ok(slider_container)
}

/// Handles revealing/hiding the selection of monitors.
fn set_button_click_event(
    window: HWND,
    parent: HWND,
    button: &Button,
    list_box: ListBox,
) -> Result<()> {
    ButtonBase::from(button)
        .Click(RoutedEventHandler::new(move |button, _args| {
            if UIElement::from(&list_box).Visibility()? == Visibility::Collapsed {
                let items = ItemsControl::from(&list_box).Items()?;
                let num_items = items.Size()? as i32;
                // Increate native window height to accomodate the revealed `ListBox`
                unsafe {
                    let height = XamlControls::CONTROLS_HEIGHT
                        + XamlControls::SELECTOR_HEIGHT * (1 + num_items);
                    let (x, y) = window_position(Window::WIDTH, height);
                    SetWindowPos(parent, HWND(0), x, y, Window::WIDTH, height, SWP_SHOWWINDOW);
                    SetWindowPos(window, HWND(0), 0, 0, Window::WIDTH, height, SWP_SHOWWINDOW);
                }
                // Show the monitor selection
                UIElement::from(&list_box).SetVisibility(Visibility::Visible)?;

                // Sets the selector text to "Select Monitor"
                if let Some(button) = button {
                    let button: Button = button.cast()?;
                    let text_block: TextBlock = ContentControl::from(&button).Content()?.cast()?;
                    text_block.SetText(HSTRING::from("Select monitor"))?;
                    text_block.SetFontWeight(FontWeights::Bold()?)?;
                }
            } else {
                hide_selection(window, parent, &list_box);

                // Update the selector text to the currently selected monitor's name
                if let Some(button) = button {
                    let button: Button = button.cast()?;
                    let text_block: TextBlock = ContentControl::from(&button).Content()?.cast()?;
                    let selected_item = Selector::from(&list_box).SelectedItem()?;
                    let monitor_name: TextBlock = selected_item.cast()?;
                    text_block.SetText(monitor_name.Text()?)?;
                    text_block.SetFontWeight(FontWeights::Normal()?)?;
                }
            }
            Ok(())
        }))
        .and(Ok(()))
}

/// Hides the selection of monitors.
pub fn hide_selection(window: HWND, parent: HWND, list_box: &ListBox) {
    // Return the native window to its default size
    unsafe {
        let (x, y) = window_position(Window::WIDTH, Window::HEIGHT);
        SetWindowPos(
            parent,
            HWND(0),
            x,
            y,
            Window::WIDTH,
            Window::HEIGHT,
            SWP_SHOWWINDOW,
        );
        SetWindowPos(
            window,
            HWND(0),
            0,
            0,
            Window::WIDTH,
            Window::HEIGHT,
            SWP_SHOWWINDOW,
        );
    }
    // Re-hide the selection
    let _ = UIElement::from(list_box).SetVisibility(Visibility::Collapsed);
}

/// Helper function for creating a `HSTRING` from an integer.
fn num_to_hstring(num: u32) -> HSTRING {
    let mut buf: [u8; 11] = [0; 11];
    write!(&mut buf[..], "{}", num).unwrap();
    let s = std::str::from_utf8(&buf).unwrap();
    HSTRING::from(s)
}
