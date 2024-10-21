#![allow(clippy::upper_case_acronyms)]

mod capture;
mod error;

use capture::utils::Frame;
pub use capture::utils::{CaptureMethod, Cords};
use capture::Capture;
pub use error::{ScreenShootError, ScreenShootResult};

pub struct ScreenShooter {
    capture: Capture,
    frame_count: usize,
}

impl ScreenShooter {
    pub fn new(method: CaptureMethod, cords: Cords) -> ScreenShootResult<Self> {
        Ok(ScreenShooter {
            capture: Capture::new(method, cords)?,
            frame_count: 0,
        })
    }

    pub fn capture_frame(&mut self) -> ScreenShootResult<Frame> {
        unsafe {
            let result = match &mut self.capture {
                Capture::GDI { gdi } => Ok(Frame::OwnedData(gdi.capture_frame()?)),
                Capture::DDA { dda } => Ok(Frame::BorrowedData(dda.get_frame_pixels()?)),
            };

            self.frame_count += 1;

            result
        }
    }

    pub fn frame_count(&self) -> usize {
        self.frame_count
    }
}
