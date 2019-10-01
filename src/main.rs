use std::ptr;
use std::ops::Deref;

use winapi::um::{objbase, combaseapi, winnt};

mod renderer;
mod app;
mod background;
mod source;
mod gui;

pub use background::{Original, BackgroundSet, DesktopBackground};
pub use source::*;

use app::DBGM;

fn main() -> Result<(), std::io::Error> {
    unsafe { check_result(combaseapi::CoInitializeEx(ptr::null_mut(), objbase::COINIT_APARTMENTTHREADED))?; }
    let mut dbgm = DBGM::new()?;
    let mut renderer = renderer::init("Desktop Background Manager");
    let mut gui = gui::GuiState::new(&mut dbgm, &mut renderer.render_sys.textures());
    renderer.main_loop(|run, ui, textures| *run = gui.update(ui, textures));
    unsafe { combaseapi::CoUninitialize(); }
    Ok(())
}

#[inline]
fn check_result(result: winnt::HRESULT) -> Result<(), std::io::Error> {
    if result < 0 {  Err(std::io::Error::from_raw_os_error(result)) } else { Ok(()) }
}

// Use this until Option::deref stabilizes.
pub(crate) trait OptionExt<T: Deref> {
    fn deref(&self) -> Option<&T::Target>;
}

impl<T: Deref> OptionExt<T> for Option<T> {
    fn deref(&self) -> Option<&T::Target> {
        self.as_ref().map(|t| t.deref())
    }
}