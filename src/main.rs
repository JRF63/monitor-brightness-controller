use windows::core::{Result, Interface, IInspectable, HSTRING};
use windows::Win32::Foundation::*;
use windows::Win32::Graphics::Gdi::*;
use windows::Win32::UI::Input::KeyboardAndMouse::SetFocus;
use windows::Win32::UI::WindowsAndMessaging::*;
use windows::UI::Xaml::Hosting::{DesktopWindowXamlSource, WindowsXamlManager};
use windows::Win32::System::WinRT::Xaml::{IDesktopWindowXamlSourceNative, IDesktopWindowXamlSourceNative2};
use windows::Win32::System::LibraryLoader::GetModuleHandleA;
use windows::UI::Xaml::Media::AcrylicBrush;
use windows::UI::Xaml::Media::AcrylicBackgroundSource;
use windows::UI::Xaml::VerticalAlignment;
use windows::UI::Xaml::HorizontalAlignment;
use windows::UI::Xaml::Controls::Orientation;
use windows::UI::Xaml::Controls::{StackPanel, TextBlock, Slider};

use windows::Win32::System::WinRT::{RoInitialize, RO_INIT_SINGLETHREADED};

const MAIN_WINDOW_CLASS: PSTR = PSTR(b"SoftwareBrightness\0".as_ptr() as *mut u8);
const EMPTY_STRING: PSTR = PSTR(b"\0".as_ptr() as *mut u8);

static mut XAML_WINDOW: HWND = HWND(0);

fn create_slider(window: HWND) -> Result<()> {
    let desktop_source = DesktopWindowXamlSource::new()?;
    let interop: IDesktopWindowXamlSourceNative = desktop_source.cast()?;
    unsafe {
        interop.AttachToWindow(window)?;
        XAML_WINDOW = interop.WindowHandle()?;
    };

    let slider = Slider::new()?;
    slider.SetMaximum(100.0)?;
    slider.SetMinimum(0.0)?;
    slider.SetHeight(300.0)?;
    slider.SetWidth(300.0)?;

    let text = windows::UI::Xaml::Controls::TextBox::new()?;
    text.SetHeader(IInspectable::try_from("WHY")?)?;
    
    unsafe { 
        // SetWindowLongA(xaml_window, GWL_STYLE, WS_POPUP.0 as _);
        SetWindowPos(XAML_WINDOW, HWND_TOPMOST, 100, 100, 300, 300, SWP_SHOWWINDOW);
    }
    let desktop_source: DesktopWindowXamlSource = interop.cast()?;
    desktop_source.SetContent(slider)?;
    Ok(())
}

pub unsafe extern "system" fn window_procedure(
    hwnd: HWND,
    umsg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match umsg {
        WM_CREATE => {
            // dbg!(hwnd);
            // create_slider(hwnd).unwrap();
        }
        WM_SIZE => {
            // SetWindowPos(unsafe { XAML_WINDOW }, HWND_TOPMOST, 100, 100, 300, 300, SWP_SHOWWINDOW);
        }
        _ => (),
    }
    DefWindowProcA(hwnd, umsg, wparam, lparam)
}

fn create_window(instance: HINSTANCE) -> Result<HWND> {
    let cursor = unsafe { LoadCursorW(None, IDC_ARROW) };
    let wcex = WNDCLASSEXA {
        cbSize: std::mem::size_of::<WNDCLASSEXA>() as u32,
        style: CS_HREDRAW | CS_VREDRAW,
        lpfnWndProc: Some(window_procedure),
        hInstance: instance,
        // hIcon: meow
        hCursor: cursor,
        lpszMenuName: EMPTY_STRING,
        lpszClassName: MAIN_WINDOW_CLASS,
        // hIconSm: meow
        ..Default::default()
    };
    unsafe {
        if RegisterClassExA(&wcex) == 0 {
            return Err(windows::core::Error::from_win32());
        }
    }

    let window = unsafe {
        CreateWindowExA(
            WS_EX_TOOLWINDOW, // WS_EX_NOREDIRECTIONBITMAP | WS_EX_TOOLWINDOW,
            MAIN_WINDOW_CLASS,
            EMPTY_STRING,
            WS_POPUP | WS_VISIBLE,
            100,
            800,
            800,
            400,
            None,
            None,
            instance,
            std::ptr::null(),
        )
    };
    if window.0 == 0 {
        return Err(windows::core::Error::from_win32());
    }
    // unsafe {
    //     ShowWindow(window, SW_SHOW);
    //     UpdateWindow(window);
    //     SetFocus(window);
    //     COLOR_WINDOW.0 + 1
    //     SetLayeredWindowAttributes(window, 0, 255, LWA_ALPHA);
    //     SetLayeredWindowAttributes(window, 999, 0, LWA_COLORKEY);
    // }
    Ok(window)
}

fn create_controls() -> Result<StackPanel> {
    let brush = AcrylicBrush::new()?;
    brush.SetBackgroundSource(AcrylicBackgroundSource::HostBackdrop)?;
    brush.SetTintColor(windows::UI::Colors::Black()?)?;
    // let y: IInspectable = IInspectable::try_from(200.0f64)?;
    // let x: windows::Foundation::IReference<f64> = unsafe { std::mem::transmute(y) };
    // brush.SetTintLuminosityOpacity(x)?;

    let xaml_container = StackPanel::new()?;
    xaml_container.SetBackground(brush.clone())?;
    
    let text_block = TextBlock::new()?;
    text_block.SetText(HSTRING::from("Generic PnP Monitor"))?;
    text_block.SetVerticalAlignment(VerticalAlignment::Center)?;
    text_block.SetHorizontalAlignment(HorizontalAlignment::Left)?;
    // text_block.SetFontSize(48.0)?;

    let slider_container = {
        let slider_container = StackPanel::new()?;
        slider_container.SetBackground(brush)?;
        slider_container.SetOrientation(Orientation::Horizontal)?;
        // slider_container.SetVerticalAlignment(VerticalAlignment::Center)?;
        slider_container.SetHorizontalAlignment(HorizontalAlignment::Center)?;

        let label1 = TextBlock::new()?;
        label1.SetText(HSTRING::from("MM"))?;
        label1.SetVerticalAlignment(VerticalAlignment::Center)?;
        label1.SetHorizontalAlignment(HorizontalAlignment::Center)?;
        label1.SetFontSize(48.0)?;
        slider_container.Children()?.Append(label1)?;

        let slider = Slider::new()?;
        slider.SetMaximum(100.0)?;
        slider.SetMinimum(0.0)?;
        slider.SetHeight(300.0)?;
        slider.SetWidth(300.0)?;
        slider_container.Children()?.Append(slider)?;

        let label2 = TextBlock::new()?;
        label2.SetText(HSTRING::from("100"))?;
        label2.SetVerticalAlignment(VerticalAlignment::Center)?;
        label2.SetHorizontalAlignment(HorizontalAlignment::Center)?;
        label2.SetFontSize(48.0)?;
        slider_container.Children()?.Append(label2)?;
        slider_container
    };

    xaml_container.Children()?.Append(text_block)?;
    xaml_container.Children()?.Append(slider_container)?;
    xaml_container.UpdateLayout()?;

    Ok(xaml_container)
}

fn main() -> Result<()> {
    
    let instance = unsafe { GetModuleHandleA(PSTR::default()) };
    let window = create_window(instance)?;

    unsafe {
        RoInitialize(RO_INIT_SINGLETHREADED)?;
    }
    let _manager = WindowsXamlManager::InitializeForCurrentThread()?;

    let desktop_source = DesktopWindowXamlSource::new()?;
    let interop: IDesktopWindowXamlSourceNative = desktop_source.clone().cast()?;
    let xaml_window = unsafe {
        interop.AttachToWindow(window)?;
        interop.WindowHandle()?
    };
    unsafe {
        // SetWindowLongA(xaml_window, GWL_EXSTYLE, WS_EX_LAYERED.0 as i32);
        // SetLayeredWindowAttributes(xaml_window, 0, 0, LWA_ALPHA);
        SetWindowPos(xaml_window, HWND(0), 0, 0, 800, 200, SWP_SHOWWINDOW);
    }

    let xaml_container = create_controls()?;
    
    desktop_source.SetContent(xaml_container)?;

    // unsafe {
    //     ShowWindow(window, SW_HIDE);
    //     UpdateWindow(window);
    // }

    // std::thread::sleep(std::time::Duration::from_millis(1000));

    unsafe {
        ShowWindow(window, SW_SHOW);
        UpdateWindow(window);
    }
    
    // // let brush = windows::UI::Xaml::Media::AcrylicBrush::new()?;
    // let extra_styles = WS_TABSTOP;
    // let style = unsafe { GetWindowLongA(xaml_window, GWL_STYLE) as u32 | extra_styles.0 };
    // unsafe { SetWindowLongA(xaml_window, GWL_STYLE, style as i32); }

    // let slider = Slider::new()?;
    // slider.SetMaximum(100.0)?;
    // slider.SetMinimum(0.0)?;
    // slider.SetHeight(300.0)?;
    // slider.SetWidth(300.0)?;
    
    // desktop_source.SetContent(slider)?;
    // unsafe { ShowWindow(xaml_window, SW_SHOW); }

    // let interop2: IDesktopWindowXamlSourceNative2 = interop.clone().cast()?;

    // let mut msg = MSG::default();
    // unsafe {
    //     while GetMessageA(&mut msg, HWND::default(), 0, 0).as_bool() {
    //         let mut result = BOOL(0);
    //         interop2.PreTranslateMessage(&msg, &mut result)?;
    //         if !result.as_bool() {
    //             TranslateMessage(&msg);
    //             DispatchMessageA(&msg);
    //         }
    //     }
    // }

    let mut msg = MSG::default();
    unsafe {
        while GetMessageA(&mut msg, HWND::default(), 0, 0).as_bool() {
            TranslateMessage(&msg);
            DispatchMessageA(&msg);
        }
    }
    
    Ok(())
}
