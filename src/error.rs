use std::fmt;

#[cfg(feature = "save")]
use image::ImageError;

#[derive(Debug)]
pub enum ScreenShootError {
    DDA {
        hresult: i32,
        file: &'static str,
        line: u32,
    },
    GDI {
        last_error: u32,
        file: &'static str,
        line: u32,
    },

    #[cfg(feature = "save")]
    Save(String),
}

#[cfg(feature = "save")]
impl From<ImageError> for ScreenShootError {
    fn from(value: ImageError) -> Self {
        ScreenShootError::Save(value.to_string())
    }
}

impl fmt::Display for ScreenShootError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            ScreenShootError::DDA {
                hresult,
                file,
                line,
            } => {
                format!("HResult error: 0x{hresult:x} \nFile: {file}, Line: {line}")
            }
            ScreenShootError::GDI {
                last_error,
                file,
                line,
            } => {
                format!("GDI error: 0x{last_error:x} \nFile: {file}, Line: {line}")
            }

            #[cfg(feature = "save")]
            ScreenShootError::Save(error) => error,
        };

        write!(f, "{message}")
    }
}

impl std::error::Error for ScreenShootError {}

pub type ScreenShootResult<T> = std::result::Result<T, ScreenShootError>;

#[macro_export]
macro_rules! h_dda {
    ($hresult:expr) => {
        match $hresult {
            hresult if hresult == 0 => Ok(()),
            hresult => Err(ScreenShootError::DDA {
                hresult,
                file: file!(),
                line: line!(),
            }),
        }
    };
}

#[macro_export]
macro_rules! h_gdi {
    ($nonzero:expr) => {
        match $nonzero {
            nonzero if nonzero == 0 => Err(ScreenShootError::GDI {
                last_error: GetLastError(),
                file: file!(),
                line: line!(),
            }),
            nonzero => Ok(nonzero),
        }
    };
}
