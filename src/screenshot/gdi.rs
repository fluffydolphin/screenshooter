use winapi::um::{
    errhandlingapi::GetLastError,
    wingdi::{
        BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, DeleteObject, GetDIBits, SelectObject,
        BITMAPINFO, BITMAPINFOHEADER, SRCCOPY,
    },
    winuser::{GetDC, GetDesktopWindow, ReleaseDC},
};

use crate::{
    error::{Error, Result},
    h_gdi,
};

use super::utils::Cords;

pub struct GDI {
    screen_dc: usize,
    bitmap: usize,
    memory_dc: usize,
    cords: Cords,
}

impl GDI {
    pub unsafe fn new(cords: Cords) -> Result<Self> {
        let screen_dc = h_gdi!(GetDC(GetDesktopWindow()) as usize)?;

        let memory_dc = h_gdi!(CreateCompatibleDC(screen_dc as _) as usize)?;

        let bitmap = h_gdi!(CreateCompatibleBitmap(
            screen_dc as _,
            cords.fov_x as i32,
            cords.fov_y as i32,
        ) as usize)?;

        SelectObject(memory_dc as _, bitmap as _);

        Ok(Self {
            screen_dc,
            memory_dc,
            bitmap,
            cords,
        })
    }

    pub unsafe fn capture_frame(&self) -> Result<Vec<u8>> {
        h_gdi!(BitBlt(
            self.memory_dc as _,
            0,
            0,
            self.cords.fov_x as i32,
            self.cords.fov_y as i32,
            self.screen_dc as _,
            self.cords.left as i32,
            self.cords.top as i32,
            SRCCOPY,
        ))?;

        let mut buffer: Vec<u8> = vec![0u8; ((self.cords.fov_x * self.cords.fov_y) * 4) as usize];

        let mut bmi = std::mem::zeroed::<BITMAPINFO>();
        bmi.bmiHeader.biSize = std::mem::size_of::<BITMAPINFOHEADER>() as u32;
        bmi.bmiHeader.biWidth = self.cords.fov_x as i32;
        bmi.bmiHeader.biHeight = -(self.cords.fov_y as i32);
        bmi.bmiHeader.biPlanes = 1;
        bmi.bmiHeader.biBitCount = 32;
        bmi.bmiHeader.biCompression = winapi::um::wingdi::BI_RGB;
        bmi.bmiHeader.biSizeImage = buffer.len() as _;

        h_gdi!(GetDIBits(
            self.screen_dc as _,
            self.bitmap as _,
            0,
            self.cords.fov_y,
            buffer.as_mut_ptr().cast(),
            &mut bmi,
            winapi::um::wingdi::DIB_RGB_COLORS,
        ))?;

        Ok(buffer)
    }

    pub fn release(&self) {
        unsafe {
            let desktop_window = GetDesktopWindow();

            DeleteObject(self.bitmap as _);
            ReleaseDC(desktop_window, self.memory_dc as _);
            ReleaseDC(desktop_window, self.screen_dc as _);
        }
    }
}

impl Drop for GDI {
    fn drop(&mut self) {
        self.release()
    }
}
