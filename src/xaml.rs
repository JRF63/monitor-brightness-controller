use windows::core::{IInspectable, Interface, Result, HSTRING};
use windows::Foundation::EventRegistrationToken;
use windows::Win32::Foundation::{BOOL, HWND};
use windows::Win32::System::WinRT::Xaml::{
    IDesktopWindowXamlSourceNative, IDesktopWindowXamlSourceNative2,
};
use windows::Win32::UI::WindowsAndMessaging::MSG;
use windows::UI::Xaml::{
    Controls::{
        Orientation,
        Primitives::{RangeBaseValueChangedEventArgs, RangeBaseValueChangedEventHandler},
        Slider, StackPanel, TextBlock,
    },
    HorizontalAlignment,
    Hosting::{DesktopWindowXamlSource, WindowsXamlManager},
    Media::{AcrylicBackgroundSource, AcrylicBrush},
    TextAlignment, Thickness, VerticalAlignment,
};

pub struct XamlControls {
    manager: WindowsXamlManager,
    source: IDesktopWindowXamlSourceNative2,
    parent: HWND,
    window: HWND,
    controls: StackPanel,
}

impl Drop for XamlControls {
    fn drop(&mut self) {
        if let Ok(source) = self.source.cast::<DesktopWindowXamlSource>() {
            source.Close().unwrap();
        }
        self.manager.Close().unwrap();
    }
}

impl XamlControls {
    pub fn new(parent: HWND) -> Result<Self> {
        let manager = WindowsXamlManager::InitializeForCurrentThread()?;
        let xaml_source = DesktopWindowXamlSource::new()?;
        let interop: IDesktopWindowXamlSourceNative = xaml_source.cast()?;
        let window = unsafe {
            interop.AttachToWindow(parent)?;
            interop.WindowHandle()?
        };

        let controls = XamlControls::create_controls()?;
        xaml_source.SetContent(&controls)?;
        let source: IDesktopWindowXamlSourceNative2 = xaml_source.cast()?;

        Ok(XamlControls {
            manager,
            source,
            parent,
            window,
            controls,
        })
    }

    pub fn window(&self) -> HWND {
        self.window
    }

    pub fn slider(&self) -> Result<Slider> {
        if self.controls.FindName(HSTRING::from("monitor_name")).is_ok() {
            println!("Found monitor_name");
        } else {
            println!("Did not find monitor_name");
        }
        let main_elements = self.controls.Children()?;
        for i in 0..main_elements.Size()? {
            if let Ok(slider_container) = main_elements.GetAt(i)?.cast::<StackPanel>() {
                let container_children = slider_container.Children()?;
                for j in 0..container_children.Size()? {
                    if let Ok(slider) = container_children.GetAt(j)?.cast::<Slider>() {
                        return Ok(slider);
                    }
                }
            }

            if let Ok(monitor_name) = main_elements.GetAt(i)?.cast::<TextBlock>() {
                println!("TextBlock name: {}", monitor_name.Name()?);
            }
        }
        Err(windows::core::Error::from_win32())
    }

    pub fn slider_value_changed<
        'a,
        F: FnMut(
                &Option<IInspectable>,
                &Option<RangeBaseValueChangedEventArgs>,
            ) -> ::windows::core::Result<()>
            + 'static,
    >(
        &'a self,
        callback: F,
    ) -> Result<EventRegistrationToken> {
        let callback = RangeBaseValueChangedEventHandler::new(callback);
        let slider = self.slider()?;
        slider.ValueChanged(callback)
    }

    pub fn filter_message(&self, message: *const MSG) -> bool {
        let mut processed = BOOL(0);
        unsafe {
            if let Ok(_) = self.source.PreTranslateMessage(message, &mut processed) {
                return processed.as_bool();
            }
        }
        false
    }

    fn create_controls() -> Result<StackPanel> {
        let brush = AcrylicBrush::new()?;
        brush.SetBackgroundSource(AcrylicBackgroundSource::HostBackdrop)?;
        brush.SetTintColor(windows::UI::Colors::Black()?)?;
        let xaml_container = StackPanel::new()?;
        xaml_container.SetBackground(brush.clone())?;

        // 360x44
        let monitor_name = TextBlock::new()?;
        monitor_name.SetName(HSTRING::from("monitor_name"))?;
        println!("TextBlock name: {}", monitor_name.Name()?);
        monitor_name.SetText(HSTRING::from("Speakers (Logitech USB Headset)"))?;
        monitor_name.SetMargin(Thickness {
            Left: 12.0,
            Top: 12.0,
            Right: 12.0,
            Bottom: 0.0,
        })?;
        let font_size = 15.0;
        monitor_name.SetFontSize(font_size)?;

        let slider_container = {
            let slider_container = StackPanel::new()?;
            slider_container.SetBackground(brush)?;
            slider_container.SetOrientation(Orientation::Horizontal)?;
            slider_container.SetHeight(76.0)?;
            let margin = 4.5;
            let slider_width = 232.0;
            let slider_height = 28.0;
            let width = (360.0 - 232.0 - 2.0 * margin) / 2.0;
            slider_container.SetMargin(Thickness {
                Left: margin,
                Top: 0.0,
                Right: margin,
                Bottom: 0.0,
            })?;

            let font_size = 23.5;

            // 67x55
            let dummy = TextBlock::new()?;
            dummy.SetText(HSTRING::from("60"))?;
            dummy.SetTextAlignment(TextAlignment::Center)?;
            dummy.SetVerticalAlignment(VerticalAlignment::Center)?;
            dummy.SetFontSize(font_size)?;
            dummy.SetWidth(width)?;

            let brightness = TextBlock::new()?;
            brightness.SetText(HSTRING::from("60"))?;
            brightness.SetTextAlignment(TextAlignment::Center)?;
            brightness.SetVerticalAlignment(VerticalAlignment::Center)?;
            brightness.SetFontSize(font_size)?;
            brightness.SetWidth(width)?;

            // 246x47
            let slider = Slider::new()?;
            slider.SetMaximum(100.0)?;
            slider.SetMinimum(0.0)?;
            slider.SetHeight(slider_height)?;
            slider.SetWidth(slider_width)?;

            slider_container.Children()?.Append(dummy)?;
            slider_container.Children()?.Append(slider)?;
            slider_container.Children()?.Append(brightness)?;
            slider_container
        };

        xaml_container.Children()?.Append(monitor_name)?;
        xaml_container.Children()?.Append(slider_container)?;
        xaml_container.UpdateLayout()?;

        Ok(xaml_container)
    }
}
