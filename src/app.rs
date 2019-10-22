use winapi::Interface;
use winapi::um::shobjidl_core::{IDesktopWallpaper, CLSID_DesktopWallpaper};
use winapi::um::combaseapi;

use crate::{check_result, background::*};

pub struct DBGM {
    interface: *mut IDesktopWallpaper,
    current_set: Option<BackgroundSet>,
}

impl DBGM {
    pub fn new() -> Result<DBGM, std::io::Error> {
        let interface = unsafe {
            let mut interface: *mut IDesktopWallpaper = std::mem::uninitialized();
            check_result(combaseapi::CoCreateInstance(
                &CLSID_DesktopWallpaper,
                std::ptr::null_mut(),
                combaseapi::CLSCTX_ALL,
                &IDesktopWallpaper::uuidof(),
                &mut interface as *mut *mut IDesktopWallpaper as *mut _,
            ))?;
            interface
        };
        Ok(DBGM {
            interface: interface,
            current_set: None,
        })
    }

    pub fn background_set(&self) -> Option<&BackgroundSet> {
        self.current_set.as_ref()
    }

    pub fn background_set_mut(&mut self) -> Option<&mut BackgroundSet> {
        self.current_set.as_mut()
    }

    pub fn open_background_set(&mut self, set: BackgroundSet) {
        self.current_set = Some(set);
    }
}

impl Drop for DBGM {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            (*self.interface).Release();
        }
    }
}

