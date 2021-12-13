use windows::core::{IInspectable, Interface, Result};
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
    Hosting::{DesktopWindowXamlSource, WindowsXamlManager},
    Media::{AcrylicBackgroundSource, AcrylicBrush},
    TextAlignment, Thickness, VerticalAlignment,
};

mod image;

pub struct XamlControls {
    manager: WindowsXamlManager,
    source: IDesktopWindowXamlSourceNative2,
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
            window,
            controls,
        })
    }

    pub fn window(&self) -> HWND {
        self.window
    }

    pub fn slider(&self) -> Result<Slider> {
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
        }
        Err(windows::core::Error::from_win32())
    }

    pub fn monitor_name(&self) -> Result<TextBlock> {
        let main_elements = self.controls.Children()?;
        for i in 0..main_elements.Size()? {
            if let Ok(monitor_name) = main_elements.GetAt(i)?.cast::<TextBlock>() {
                return Ok(monitor_name);
            }
        }
        Err(windows::core::Error::from_win32())
    }

    pub fn brightness_number(&self) -> Result<TextBlock> {
        let main_elements = self.controls.Children()?;
        for i in 0..main_elements.Size()? {
            if let Ok(slider_container) = main_elements.GetAt(i)?.cast::<StackPanel>() {
                let container_children = slider_container.Children()?;
                for j in 0..container_children.Size()? {
                    if let Ok(brightness_number) = container_children.GetAt(j)?.cast::<TextBlock>() {
                        return Ok(brightness_number);
                    }
                }
            }
        }
        Err(windows::core::Error::from_win32())
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

    pub fn create_slider_callback<
        'a,
        F: FnMut(
                &Option<IInspectable>,
                &Option<RangeBaseValueChangedEventArgs>,
            ) -> ::windows::core::Result<()>
            + 'static,
    >(callback: F) -> RangeBaseValueChangedEventHandler {
        RangeBaseValueChangedEventHandler::new(callback)
    }

    fn create_controls() -> Result<StackPanel> {
        let brush = AcrylicBrush::new()?;
        brush.SetBackgroundSource(AcrylicBackgroundSource::HostBackdrop)?;
        brush.SetTintColor(windows::UI::Colors::Black()?)?;

        let xaml_container = StackPanel::new()?;
        xaml_container.SetBackground(brush.clone())?;

        let monitor_name = TextBlock::new()?;
        monitor_name.SetPadding(Thickness {
            Left: 12.0,
            Top: 12.0,
            Right: 12.0,
            Bottom: 0.0,
        })?;
        monitor_name.SetFontSize(15.0)?;

        let slider_container = {
            let slider_container = StackPanel::new()?;
            slider_container.SetBackground(brush)?;
            slider_container.SetOrientation(Orientation::Horizontal)?;
            slider_container.SetHeight(76.0)?;

            let slider_width = 232.0;
            let slider_height = 28.0;
            let width = (360.0 - slider_width) / 2.0;

            let image = image::create_image()?;
            image.SetWidth(width)?;
            image.SetMargin(Thickness {
                Left: 0.0,
                Top: 4.0,
                Right: 0.0,
                Bottom: 0.0,
            })?;

            let brightness_number = TextBlock::new()?;
            brightness_number.SetWidth(width)?;
            brightness_number.SetTextAlignment(TextAlignment::Center)?;
            brightness_number.SetVerticalAlignment(VerticalAlignment::Center)?;
            brightness_number.SetFontSize(23.5)?;
            
            let slider = Slider::new()?;
            slider.SetMaximum(100.0)?;
            slider.SetMinimum(0.0)?;
            slider.SetWidth(slider_width)?;
            slider.SetHeight(slider_height)?;

            slider_container.Children()?.Append(image)?;
            slider_container.Children()?.Append(slider)?;
            slider_container.Children()?.Append(brightness_number)?;
            slider_container
        };

        xaml_container.Children()?.Append(monitor_name)?;
        xaml_container.Children()?.Append(slider_container)?;
        xaml_container.UpdateLayout()?;

        Ok(xaml_container)
    }
}
