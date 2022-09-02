mod image;

use std::marker::PhantomData;

use windows::{
    core::{IInspectable, Interface, Result},
    Win32::{
        Foundation::{BOOL, HWND},
        Graphics::Gdi::UpdateWindow,
        System::WinRT::Xaml::{IDesktopWindowXamlSourceNative, IDesktopWindowXamlSourceNative2},
        UI::WindowsAndMessaging::{SetWindowPos, MSG, SWP_SHOWWINDOW},
    },
    UI::Xaml::{
        Controls::{
            Orientation, Panel,
            Primitives::{
                RangeBase, RangeBaseValueChangedEventArgs, RangeBaseValueChangedEventHandler,
            },
            Slider, StackPanel, TextBlock,
        },
        FrameworkElement,
        Hosting::{DesktopWindowXamlSource, WindowsXamlManager},
        Media::{AcrylicBackgroundSource, AcrylicBrush},
        TextAlignment, Thickness, UIElement, VerticalAlignment,
    },
};

use crate::Window;

pub struct XamlControls<'a> {
    manager: WindowsXamlManager,
    source: IDesktopWindowXamlSourceNative2,
    controls: StackPanel,
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
    pub fn new(parent: &'a Window, width: i32, height: i32) -> Result<Self> {
        let manager = WindowsXamlManager::InitializeForCurrentThread()?;
        let xaml_source = DesktopWindowXamlSource::new()?;
        let interop: IDesktopWindowXamlSourceNative = xaml_source.cast()?;
        let window = unsafe {
            interop.AttachToWindow(parent.as_handle())?;
            interop.WindowHandle()?
        };

        let controls = XamlControls::create_controls()?;
        xaml_source.SetContent(&controls)?;
        let source: IDesktopWindowXamlSourceNative2 = xaml_source.cast()?;

        // Sets the XAML window's position on its parent
        unsafe {
            SetWindowPos(window, HWND(0), 0, 0, width, height, SWP_SHOWWINDOW);
            UpdateWindow(parent.as_handle());
        }

        Ok(XamlControls {
            manager,
            source,
            controls,
            parent: PhantomData,
        })
    }

    pub fn slider(&self) -> Result<Slider> {
        let main_elements = Panel::from(&self.controls).Children()?;
        for i in 0..main_elements.Size()? {
            if let Ok(slider_container) = main_elements.GetAt(i)?.cast::<StackPanel>() {
                let container_children = Panel::from(&slider_container).Children()?;
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
        let main_elements = Panel::from(&self.controls).Children()?;
        for i in 0..main_elements.Size()? {
            if let Ok(monitor_name) = main_elements.GetAt(i)?.cast::<TextBlock>() {
                return Ok(monitor_name);
            }
        }
        Err(windows::core::Error::from_win32())
    }

    pub fn brightness_number(&self) -> Result<TextBlock> {
        let main_elements = Panel::from(&self.controls).Children()?;
        for i in 0..main_elements.Size()? {
            if let Ok(slider_container) = main_elements.GetAt(i)?.cast::<StackPanel>() {
                let container_children = Panel::from(slider_container).Children()?;
                for j in 0..container_children.Size()? {
                    if let Ok(brightness_number) = container_children.GetAt(j)?.cast::<TextBlock>()
                    {
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
        'b,
        F: FnMut(
                &Option<IInspectable>,
                &Option<RangeBaseValueChangedEventArgs>,
            ) -> ::windows::core::Result<()>
            + 'static
            + Send,
    >(
        callback: F,
    ) -> RangeBaseValueChangedEventHandler {
        RangeBaseValueChangedEventHandler::new(callback)
    }

    fn create_controls() -> Result<StackPanel> {
        let brush = AcrylicBrush::new()?;
        brush.SetBackgroundSource(AcrylicBackgroundSource::HostBackdrop)?;
        brush.SetTintColor(windows::UI::Colors::Black()?)?;

        let xaml_container = StackPanel::new()?;
        Panel::from(&xaml_container).SetBackground(brush.clone())?;

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
            Panel::from(&slider_container).SetBackground(brush)?;
            slider_container.SetOrientation(Orientation::Horizontal)?;
            FrameworkElement::from(&slider_container).SetHeight(76.0)?;

            let slider_width = 232.0;
            let slider_height = 28.0;
            let width = (360.0 - slider_width) / 2.0;

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
            FrameworkElement::from(&brightness_number)
                .SetVerticalAlignment(VerticalAlignment::Center)?;
            brightness_number.SetFontSize(23.5)?;

            let slider = Slider::new()?;
            RangeBase::from(&slider).SetMaximum(100.0)?;
            RangeBase::from(&slider).SetMinimum(0.0)?;
            FrameworkElement::from(&slider).SetWidth(slider_width)?;
            FrameworkElement::from(&slider).SetHeight(slider_height)?;

            Panel::from(&slider_container).Children()?.Append(image)?;
            Panel::from(&slider_container).Children()?.Append(slider)?;
            Panel::from(&slider_container)
                .Children()?
                .Append(brightness_number)?;
            slider_container
        };

        Panel::from(&xaml_container)
            .Children()?
            .Append(monitor_name)?;
        Panel::from(&xaml_container)
            .Children()?
            .Append(slider_container)?;
        UIElement::from(&xaml_container).UpdateLayout()?;

        Ok(xaml_container)
    }
}
