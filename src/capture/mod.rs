use crate::error::ScreenShootResult;

use self::{
    dda::{create_device, DDA},
    gdi::GDI,
    utils::{CaptureMethod, Cords, Frame},
};

mod dda;
mod gdi;
pub mod utils;

pub enum Capture {
    GDI { gdi: GDI },
    DDA { dda: DDA },
}

impl Capture {
    pub fn new(method: CaptureMethod, cords: Cords) -> ScreenShootResult<Self> {
        unsafe {
            match method {
                CaptureMethod::GDI => Ok(Capture::GDI {
                    gdi: GDI::new(cords)?,
                }),
                CaptureMethod::DDA => {
                    let (device, context) = create_device()?;
                    Ok(Capture::DDA {
                        dda: DDA::new(device, context, cords)?,
                    })
                }
            }
        }
    }

    pub fn capture_frame(&mut self) -> ScreenShootResult<Frame> {
        unsafe {
            match self {
                Capture::GDI { gdi } => Ok(Frame::OwnedData(gdi.capture_frame()?)),
                Capture::DDA { dda } => Ok(Frame::BorrowedData(dda.get_frame_pixels()?)),
            }
        }
    }
}

impl Drop for Capture {
    fn drop(&mut self) {
        unsafe {
            match self {
                Capture::GDI { gdi } => gdi.release(),
                Capture::DDA { dda } => dda.release(),
            };
        }
    }
}
