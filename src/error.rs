use std::fmt;

#[derive(Debug)]
pub enum Error {
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
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Error::DDA {
                hresult,
                file,
                line,
            } => {
                format!("HResult error: 0x{hresult:x} \nFile: {file}, Line: {line}")
            }
            Error::GDI {
                last_error,
                file,
                line,
            } => {
                format!("GDI error: 0x{last_error:x} \nFile: {file}, Line: {line}")
            }
        };

        write!(f, "{message}")
    }
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;

#[macro_export]
macro_rules! h_dda {
    ($hresult:expr) => {
        match $hresult {
            hresult if hresult == 0 => Ok(()),
            hresult => Err(Error::DDA {
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
            nonzero if nonzero == 0 => Err(Error::GDI {
                last_error: GetLastError(),
                file: file!(),
                line: line!(),
            }),
            nonzero => Ok(nonzero),
        }
    };
}
