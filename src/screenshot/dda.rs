use std::{mem, ptr};

use winapi::{
    shared::{
        dxgi::{IDXGIAdapter, IDXGIDevice},
        dxgi1_2::{IDXGIOutput1, IDXGIOutputDuplication, DXGI_OUTDUPL_FRAME_INFO},
        dxgiformat::DXGI_FORMAT_B8G8R8A8_UNORM,
        dxgitype::DXGI_SAMPLE_DESC,
        winerror::{
            DXGI_ERROR_ACCESS_LOST, DXGI_ERROR_INVALID_CALL, DXGI_ERROR_WAIT_TIMEOUT,
            DXGI_ERROR_WAS_STILL_DRAWING,
        },
    },
    um::{
        d3d11::{
            D3D11CreateDevice, ID3D11Device, ID3D11DeviceContext, ID3D11Resource, ID3D11Texture2D,
            D3D11_CPU_ACCESS_READ, D3D11_CREATE_DEVICE_DEBUG, D3D11_MAPPED_SUBRESOURCE,
            D3D11_MAP_FLAG_DO_NOT_WAIT, D3D11_MAP_READ, D3D11_SDK_VERSION, D3D11_TEXTURE2D_DESC,
            D3D11_USAGE_STAGING,
        },
        d3dcommon::D3D_DRIVER_TYPE_HARDWARE,
    },
    Interface,
};

use crate::{
    error::{Result, ScreenShotError},
    h_dda,
};

use super::utils::Cords;

pub unsafe fn create_device() -> Result<(*mut ID3D11Device, *mut ID3D11DeviceContext)> {
    let mut device = ptr::null_mut();
    let mut context = ptr::null_mut();

    h_dda!(D3D11CreateDevice(
        ptr::null_mut(),
        D3D_DRIVER_TYPE_HARDWARE,
        ptr::null_mut(),
        D3D11_CREATE_DEVICE_DEBUG,
        ptr::null(),
        0,
        D3D11_SDK_VERSION,
        &mut device,
        ptr::null_mut(),
        &mut context,
    ))?;

    Ok((device, context))
}

pub struct DDA {
    device: usize,
    context: usize,
    out_dup: usize,
    frame_lock: bool,
    cpu_texture: usize,
    cords: Cords,
}

impl DDA {
    pub unsafe fn new(
        device: *mut ID3D11Device,
        context: *mut ID3D11DeviceContext,
        cords: Cords,
    ) -> Result<Self> {
        let mut device2 = ptr::null_mut();
        h_dda!((*device).QueryInterface(&IDXGIDevice::uuidof(), &mut device2))?;

        let mut adapter = ptr::null_mut();
        h_dda!((*(device2 as *mut IDXGIDevice)).GetParent(&IDXGIAdapter::uuidof(), &mut adapter))?;

        let mut output = ptr::null_mut();
        h_dda!((*(adapter as *mut IDXGIAdapter)).EnumOutputs(0, &mut output))?;

        let mut output1 = ptr::null_mut();
        h_dda!((*output).QueryInterface(&IDXGIOutput1::uuidof(), &mut output1))?;

        let mut out_dup = ptr::null_mut();
        h_dda!((*(output1 as *mut IDXGIOutput1)).DuplicateOutput(device as _, &mut out_dup))?;

        (*(output1 as *mut IDXGIOutput1)).Release();
        (*output).Release();
        (*(device2 as *mut IDXGIDevice)).Release();

        let cpu_desc = D3D11_TEXTURE2D_DESC {
            Width: cords.fov_x,
            Height: cords.fov_y,
            Format: DXGI_FORMAT_B8G8R8A8_UNORM,
            ArraySize: 1,
            BindFlags: 0,
            MiscFlags: 0,
            SampleDesc: DXGI_SAMPLE_DESC {
                Count: 1,
                Quality: 0,
            },
            MipLevels: 1,
            CPUAccessFlags: D3D11_CPU_ACCESS_READ,
            Usage: D3D11_USAGE_STAGING,
        };

        let mut cpu_texture = ptr::null_mut();
        h_dda!((*device).CreateTexture2D(&cpu_desc, ptr::null_mut(), &mut cpu_texture))?;

        Ok(Self {
            device: device as usize,
            context: context as usize,
            out_dup: out_dup as usize,
            frame_lock: false,
            cpu_texture: cpu_texture as usize,
            cords,
        })
    }

    pub unsafe fn get_frame_pixels(&mut self) -> Result<&[u8]> {
        loop {
            match self.get_frame_texture(self.cpu_texture as _) {
                Ok(()) => break,
                Err(ScreenShotError::DDA {
                    hresult: DXGI_ERROR_ACCESS_LOST,
                    file: _,
                    line: _,
                })
                | Err(ScreenShotError::DDA {
                    hresult: DXGI_ERROR_INVALID_CALL,
                    file: _,
                    line: _,
                }) => {
                    self.release();
                    let (d11_device, d11_context) = create_device()?;
                    *self = DDA::new(d11_device, d11_context, self.cords)?;
                    continue;
                }
                Err(e) => return Err(e),
            };
        }

        let mut mapped_res = mem::zeroed::<D3D11_MAPPED_SUBRESOURCE>();
        loop {
            match h_dda!((*(self.context as *mut ID3D11DeviceContext)).Map(
                self.cpu_texture as _,
                0,
                D3D11_MAP_READ,
                D3D11_MAP_FLAG_DO_NOT_WAIT,
                &mut mapped_res,
            )) {
                Ok(()) => break,
                Err(ScreenShotError::DDA {
                    hresult: DXGI_ERROR_WAS_STILL_DRAWING,
                    file: _,
                    line: _,
                }) => {
                    continue;
                }
                Err(ScreenShotError::DDA {
                    hresult: DXGI_ERROR_ACCESS_LOST,
                    file: _,
                    line: _,
                })
                | Err(ScreenShotError::DDA {
                    hresult: DXGI_ERROR_INVALID_CALL,
                    file: _,
                    line: _,
                }) => {
                    self.release();
                    *self = DDA::new(self.device as _, self.context as _, self.cords)?;
                    continue;
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }

        (*(self.context as *mut ID3D11DeviceContext)).Unmap(self.cpu_texture as _, 0);

        Ok(std::slice::from_raw_parts(
            mapped_res.pData as *const u8,
            ((self.cords.fov_x * self.cords.fov_y) * 4) as usize,
        ))
    }

    unsafe fn get_frame_texture(&mut self, texture: *mut ID3D11Texture2D) -> Result<()> {
        if self.frame_lock {
            (*(self.out_dup as *mut IDXGIOutputDuplication)).ReleaseFrame();
            self.frame_lock = false
        }

        let mut frame_info = mem::zeroed::<DXGI_OUTDUPL_FRAME_INFO>();
        let mut desktop_res = ptr::null_mut();

        loop {
            match h_dda!(
                (*(self.out_dup as *mut IDXGIOutputDuplication)).AcquireNextFrame(
                    99999,
                    &mut frame_info,
                    &mut desktop_res
                )
            ) {
                Ok(()) => break,
                Err(ScreenShotError::DDA {
                    hresult: DXGI_ERROR_WAIT_TIMEOUT,
                    file: _,
                    line: _,
                }) => continue,
                Err(e) => return Err(e),
            }
        }
        self.frame_lock = true;

        let mut desktop_image = ptr::null_mut();
        h_dda!((*desktop_res).QueryInterface(&ID3D11Resource::uuidof(), &mut desktop_image))?;

        (*(self.context as *mut ID3D11DeviceContext)).CopySubresourceRegion(
            texture as _,
            0,
            0,
            0,
            0,
            desktop_image as _,
            0,
            &self.cords.into(),
        );
        (*(desktop_image as *mut ID3D11Resource)).Release();
        (*desktop_res).Release();

        Ok(())
    }

    pub unsafe fn release(&mut self) {
        (*(self.cpu_texture as *mut ID3D11Texture2D)).Release();
        (*(self.out_dup as *mut IDXGIOutputDuplication)).Release();
        (*(self.context as *mut ID3D11DeviceContext)).Release();
        (*(self.device as *mut ID3D11Device)).Release();
    }
}
