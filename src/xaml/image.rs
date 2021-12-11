use windows::core::{Interface, Result};
use windows::Graphics::Imaging::SoftwareBitmap;
use windows::Storage::Streams::Buffer;
use windows::Win32::System::WinRT::IBufferByteAccess;
use windows::UI::Xaml::Controls::Image;
use windows::UI::Xaml::Media::Imaging::SoftwareBitmapSource;

pub fn create_image() -> Result<Image> {
    const IMAGE_WIDTH: i32 = 24;
    const IMAGE_HEIGHT: i32 = 24;

    let buffer = Buffer::Create(ICON.len() as u32)?;
    let writeable: IBufferByteAccess = buffer.0.cast()?;
    unsafe {
        let ptr = writeable.Buffer()?;
        let slice = &mut *(ptr as *mut [u8; ICON.len()]);
        slice.copy_from_slice(&ICON);
        buffer.SetLength(ICON.len() as u32)?;
    }

    let bitmap = SoftwareBitmap::CreateCopyWithAlphaFromBuffer(
        buffer,
        windows::Graphics::Imaging::BitmapPixelFormat::Bgra8,
        IMAGE_WIDTH,
        IMAGE_HEIGHT,
        windows::Graphics::Imaging::BitmapAlphaMode::Premultiplied,
    )?;
    
    let image_source = SoftwareBitmapSource::new()?;
    let _ignored_async = image_source.SetBitmapAsync(bitmap)?;

    let image = Image::new()?;
    image.SetSource(image_source)?;
    image.SetWidth(IMAGE_WIDTH as f64)?;
    image.SetHeight(IMAGE_HEIGHT as f64)?;
    Ok(image)
}

const ICON: [u8; 2304] = [
    0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
    0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
    0x0, 0x0, 0x40, 0x40, 0x40, 0x40, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x40, 0x40,
    0x40, 0x40, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
    0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
    0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
    0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
    0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x40, 0x40, 0x40, 0x40, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, 0xff, 0x40, 0x40, 0x40, 0x40, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
    0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
    0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
    0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x27, 0x27, 0x27, 0x27, 0x18, 0x18, 0x18, 0x18, 0x0, 0x0, 0x0,
    0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x40, 0x40, 0x40, 0x40, 0xff,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x40, 0x40, 0x40, 0x40, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
    0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x18, 0x18, 0x18, 0x18, 0x27, 0x27, 0x27,
    0x27, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
    0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x7a, 0x7a, 0x7a, 0x7a, 0xd7, 0xd7, 0xd7,
    0xd7, 0x50, 0x50, 0x50, 0x50, 0x18, 0x18, 0x18, 0x18, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
    0x0, 0x0, 0x0, 0x0, 0x40, 0x40, 0x40, 0x40, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0x40, 0x40, 0x40, 0x40, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x18, 0x18,
    0x18, 0x18, 0x50, 0x50, 0x50, 0x50, 0xd7, 0xd7, 0xd7, 0xd7, 0x7a, 0x7a, 0x7a, 0x7a, 0x0, 0x0,
    0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x27,
    0x27, 0x27, 0x27, 0xd7, 0xd7, 0xd7, 0xd7, 0xff, 0xff, 0xff, 0xff, 0xd7, 0xd7, 0xd7, 0xd7, 0x50,
    0x50, 0x50, 0x50, 0x18, 0x18, 0x18, 0x18, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
    0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
    0x0, 0x0, 0x0, 0x18, 0x18, 0x18, 0x18, 0x50, 0x50, 0x50, 0x50, 0xd7, 0xd7, 0xd7, 0xd7, 0xff,
    0xff, 0xff, 0xff, 0xd7, 0xd7, 0xd7, 0xd7, 0x27, 0x27, 0x27, 0x27, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
    0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x18, 0x18, 0x18, 0x18, 0x50, 0x50, 0x50,
    0x50, 0xd7, 0xd7, 0xd7, 0xd7, 0xff, 0xff, 0xff, 0xff, 0xd4, 0xd4, 0xd4, 0xd4, 0x16, 0x16, 0x16,
    0x16, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
    0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x16, 0x16, 0x16, 0x16,
    0xd4, 0xd4, 0xd4, 0xd4, 0xff, 0xff, 0xff, 0xff, 0xd7, 0xd7, 0xd7, 0xd7, 0x50, 0x50, 0x50, 0x50,
    0x18, 0x18, 0x18, 0x18, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
    0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x18, 0x18, 0x18, 0x18, 0x50, 0x50, 0x50, 0x50, 0xd4, 0xd4, 0xd4,
    0xd4, 0x47, 0x47, 0x47, 0x47, 0x0, 0x0, 0x0, 0x0, 0x17, 0x17, 0x17, 0x17, 0xb5, 0xb5, 0xb5,
    0xb5, 0xf3, 0xf3, 0xf3, 0xf3, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xf3, 0xf3, 0xf3,
    0xf3, 0xb5, 0xb5, 0xb5, 0xb5, 0x17, 0x17, 0x17, 0x17, 0x0, 0x0, 0x0, 0x0, 0x47, 0x47, 0x47,
    0x47, 0xd4, 0xd4, 0xd4, 0xd4, 0x50, 0x50, 0x50, 0x50, 0x18, 0x18, 0x18, 0x18, 0x0, 0x0, 0x0,
    0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
    0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x18, 0x18, 0x18, 0x18, 0x16, 0x16, 0x16, 0x16, 0x0, 0x0, 0x0,
    0x0, 0x6d, 0x6d, 0x6d, 0x6d, 0xed, 0xed, 0xed, 0xed, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, 0xed, 0xed, 0xed, 0xed, 0x6d, 0x6d, 0x6d, 0x6d, 0x0, 0x0, 0x0, 0x0, 0x16, 0x16, 0x16,
    0x16, 0x18, 0x18, 0x18, 0x18, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
    0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
    0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x17, 0x17, 0x17, 0x17, 0xed, 0xed, 0xed, 0xed, 0xff,
    0xff, 0xff, 0xff, 0xbf, 0xbf, 0xbf, 0xbf, 0x53, 0x53, 0x53, 0x53, 0xd, 0xd, 0xd, 0xd, 0xd, 0xd,
    0xd, 0xd, 0x53, 0x53, 0x53, 0x53, 0xbf, 0xbf, 0xbf, 0xbf, 0xff, 0xff, 0xff, 0xff, 0xed, 0xed,
    0xed, 0xed, 0x17, 0x17, 0x17, 0x17, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
    0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
    0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0xb5,
    0xb5, 0xb5, 0xb5, 0xff, 0xff, 0xff, 0xff, 0xbf, 0xbf, 0xbf, 0xbf, 0x27, 0x27, 0x27, 0x27, 0x0,
    0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x27, 0x27, 0x27,
    0x27, 0xbf, 0xbf, 0xbf, 0xbf, 0xff, 0xff, 0xff, 0xff, 0xb5, 0xb5, 0xb5, 0xb5, 0x0, 0x0, 0x0,
    0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
    0x0, 0x0, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40,
    0x40, 0x40, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0xf3, 0xf3, 0xf3, 0xf3, 0xff, 0xff, 0xff,
    0xff, 0x53, 0x53, 0x53, 0x53, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
    0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x53, 0x53, 0x53, 0x53, 0xff, 0xff,
    0xff, 0xff, 0xf3, 0xf3, 0xf3, 0xf3, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x40, 0x40, 0x40,
    0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0xff, 0xff, 0xff,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x0, 0x0, 0x0,
    0x0, 0x0, 0x0, 0x0, 0x0, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xd, 0xd, 0xd, 0xd,
    0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
    0x0, 0x0, 0x0, 0x0, 0x0, 0xd, 0xd, 0xd, 0xd, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0xff, 0xff,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xd, 0xd, 0xd, 0xd, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
    0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0xd, 0xd, 0xd,
    0xd, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40,
    0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0xf3, 0xf3, 0xf3, 0xf3, 0xff, 0xff, 0xff, 0xff, 0x53,
    0x53, 0x53, 0x53, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
    0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x53, 0x53, 0x53, 0x53, 0xff, 0xff, 0xff, 0xff,
    0xf3, 0xf3, 0xf3, 0xf3, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x40, 0x40, 0x40, 0x40, 0x40,
    0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
    0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0xb5,
    0xb5, 0xb5, 0xb5, 0xff, 0xff, 0xff, 0xff, 0xbf, 0xbf, 0xbf, 0xbf, 0x27, 0x27, 0x27, 0x27, 0x0,
    0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x27, 0x27, 0x27,
    0x27, 0xbf, 0xbf, 0xbf, 0xbf, 0xff, 0xff, 0xff, 0xff, 0xb5, 0xb5, 0xb5, 0xb5, 0x0, 0x0, 0x0,
    0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
    0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
    0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x17, 0x17, 0x17, 0x17, 0xed, 0xed, 0xed, 0xed, 0xff, 0xff,
    0xff, 0xff, 0xbf, 0xbf, 0xbf, 0xbf, 0x53, 0x53, 0x53, 0x53, 0xd, 0xd, 0xd, 0xd, 0xd, 0xd, 0xd,
    0xd, 0x53, 0x53, 0x53, 0x53, 0xbf, 0xbf, 0xbf, 0xbf, 0xff, 0xff, 0xff, 0xff, 0xed, 0xed, 0xed,
    0xed, 0x17, 0x17, 0x17, 0x17, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
    0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
    0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x18, 0x18, 0x18, 0x18, 0x16, 0x16, 0x16, 0x16, 0x0,
    0x0, 0x0, 0x0, 0x6d, 0x6d, 0x6d, 0x6d, 0xed, 0xed, 0xed, 0xed, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, 0xff, 0xff, 0xed, 0xed, 0xed, 0xed, 0x6d, 0x6d, 0x6d, 0x6d, 0x0, 0x0, 0x0, 0x0, 0x16,
    0x16, 0x16, 0x16, 0x18, 0x18, 0x18, 0x18, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
    0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x18,
    0x18, 0x18, 0x18, 0x50, 0x50, 0x50, 0x50, 0xd4, 0xd4, 0xd4, 0xd4, 0x47, 0x47, 0x47, 0x47, 0x0,
    0x0, 0x0, 0x0, 0x17, 0x17, 0x17, 0x17, 0xb5, 0xb5, 0xb5, 0xb5, 0xf3, 0xf3, 0xf3, 0xf3, 0xff,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xf3, 0xf3, 0xf3, 0xf3, 0xb5, 0xb5, 0xb5, 0xb5, 0x17,
    0x17, 0x17, 0x17, 0x0, 0x0, 0x0, 0x0, 0x47, 0x47, 0x47, 0x47, 0xd4, 0xd4, 0xd4, 0xd4, 0x50,
    0x50, 0x50, 0x50, 0x18, 0x18, 0x18, 0x18, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
    0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x18, 0x18, 0x18, 0x18, 0x50, 0x50, 0x50,
    0x50, 0xd7, 0xd7, 0xd7, 0xd7, 0xff, 0xff, 0xff, 0xff, 0xd4, 0xd4, 0xd4, 0xd4, 0x16, 0x16, 0x16,
    0x16, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
    0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x16, 0x16, 0x16, 0x16,
    0xd4, 0xd4, 0xd4, 0xd4, 0xff, 0xff, 0xff, 0xff, 0xd7, 0xd7, 0xd7, 0xd7, 0x50, 0x50, 0x50, 0x50,
    0x18, 0x18, 0x18, 0x18, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
    0x0, 0x0, 0x27, 0x27, 0x27, 0x27, 0xd7, 0xd7, 0xd7, 0xd7, 0xff, 0xff, 0xff, 0xff, 0xd7, 0xd7,
    0xd7, 0xd7, 0x50, 0x50, 0x50, 0x50, 0x18, 0x18, 0x18, 0x18, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
    0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
    0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x18, 0x18, 0x18, 0x18, 0x50, 0x50, 0x50, 0x50, 0xd7, 0xd7, 0xd7,
    0xd7, 0xff, 0xff, 0xff, 0xff, 0xd7, 0xd7, 0xd7, 0xd7, 0x27, 0x27, 0x27, 0x27, 0x0, 0x0, 0x0,
    0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x7a,
    0x7a, 0x7a, 0x7a, 0xd7, 0xd7, 0xd7, 0xd7, 0x50, 0x50, 0x50, 0x50, 0x18, 0x18, 0x18, 0x18, 0x0,
    0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x40, 0x40, 0x40, 0x40, 0xff, 0xff,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x40, 0x40, 0x40, 0x40, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
    0x0, 0x0, 0x0, 0x0, 0x0, 0x18, 0x18, 0x18, 0x18, 0x50, 0x50, 0x50, 0x50, 0xd7, 0xd7, 0xd7,
    0xd7, 0x7a, 0x7a, 0x7a, 0x7a, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
    0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x27, 0x27, 0x27,
    0x27, 0x18, 0x18, 0x18, 0x18, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
    0x0, 0x0, 0x0, 0x40, 0x40, 0x40, 0x40, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x40,
    0x40, 0x40, 0x40, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
    0x0, 0x18, 0x18, 0x18, 0x18, 0x27, 0x27, 0x27, 0x27, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
    0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
    0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
    0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x40, 0x40, 0x40, 0x40, 0xff, 0xff, 0xff,
    0xff, 0xff, 0xff, 0xff, 0xff, 0x40, 0x40, 0x40, 0x40, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
    0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
    0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
    0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
    0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x40, 0x40, 0x40,
    0x40, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x40, 0x40, 0x40, 0x40, 0x0, 0x0, 0x0,
    0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
    0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
];
