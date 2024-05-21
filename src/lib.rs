#![allow(clippy::upper_case_acronyms)]

mod error;
mod screenshot;

pub use error::ScreenShotError;
pub use screenshot::Capture;
pub use screenshot::CaptureMethod;
