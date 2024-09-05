use std::ops::Deref;

use winapi::um::d3d11::D3D11_BOX;

#[derive(Debug)]
pub enum CaptureMethod {
    GDI,
    DDA,
}

#[derive(Clone, Copy)]
pub struct Cords {
    pub fov_x: u32,
    pub fov_y: u32,
    pub left: u32,
    pub top: u32,
    pub right: u32,
    pub bottom: u32,
}

impl Cords {
    pub fn new(fov_x: u32, fov_y: u32, width: u32, height: u32) -> Self {
        let (left, right) = if fov_x == width {
            (0, fov_x)
        } else {
            let half_width = width / 2;
            let half_fov_x = fov_x / 2;

            (half_width - half_fov_x, half_width + half_fov_x)
        };

        let (top, bottom) = if fov_y == height {
            (0, fov_y)
        } else {
            let half_height = height / 2;
            let half_fov_y = fov_y / 2;

            (half_height - half_fov_y, half_height + half_fov_y)
        };

        Self {
            fov_x,
            fov_y,
            left,
            top,
            right,
            bottom,
        }
    }
}

impl From<Cords> for D3D11_BOX {
    fn from(value: Cords) -> Self {
        D3D11_BOX {
            left: value.left,
            top: value.top,
            front: 0,
            right: value.right,
            bottom: value.bottom,
            back: 1,
        }
    }
}

#[derive(Debug)]
pub enum Frame<'a> {
    OwnedData(Vec<u8>),
    BorrowedData(&'a [u8]),
}

#[cfg(feature = "save")]
impl<'a> Frame<'a> {
    pub fn save(&self, path: &Path, width: u32, height: u32) -> Result<()> {
        use crate::error::Result;
        use image::RgbaImage;
        use std::path::Path;

        let frame = match self {
            Frame::OwnedData(frame) => frame.as_slice(),
            Frame::BorrowedData(frame) => frame,
        };

        let rgba = bgra_to_rgba(frame);
        let image = RgbaImage::from_raw(width, height, rgba).unwrap();
        image.save(path).unwrap();

        Ok(())
    }
}

#[cfg(feature = "save")]
fn bgra_to_rgba(bgra: &[u8]) -> Vec<u8> {
    bgra.chunks(4)
        .flat_map(|chunk| vec![chunk[2], chunk[1], chunk[0], chunk[3]])
        .collect()
}

impl Deref for Frame<'_> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        match self {
            Frame::OwnedData(v) => v,
            Frame::BorrowedData(b) => b,
        }
    }
}
