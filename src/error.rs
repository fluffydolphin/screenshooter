use std::fmt;

#[cfg(feature = "save")]
use image::ImageError;

#[derive(Debug)]
pub enum ScreenShotError {
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
    Save(ImageError),
}

impl fmt::Display for ScreenShotError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            ScreenShotError::DDA {
                hresult,
                file,
                line,
            } => {
                format!("HResult error: 0x{hresult:x} \nFile: {file}, Line: {line}")
            }
            ScreenShotError::GDI {
                last_error,
                file,
                line,
            } => {
                format!("GDI error: 0x{last_error:x} \nFile: {file}, Line: {line}")
            }

            #[cfg(feature = "save")]
            ScreenShotError::Save(error) => error.to_string(),
        };

        write!(f, "{message}")
    }
}

impl std::error::Error for ScreenShotError {}

pub type Result<T> = std::result::Result<T, ScreenShotError>;

#[macro_export]
macro_rules! h_dda {
    ($hresult:expr) => {
        match $hresult {
            hresult if hresult == 0 => Ok(()),
            hresult => Err(ScreenShotError::DDA {
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
            nonzero if nonzero == 0 => Err(ScreenShotError::GDI {
                last_error: GetLastError(),
                file: file!(),
                line: line!(),
            }),
            nonzero => Ok(nonzero),
        }
    };
}
