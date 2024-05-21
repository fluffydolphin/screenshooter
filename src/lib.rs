#![allow(clippy::upper_case_acronyms)]

mod capture;
mod error;

pub use capture::utils::CaptureMethod;
pub use capture::utils::Cords;
pub use capture::Capture;
pub use error::ScreenShotError;
