use std::ptr;

use winapi::um::{objbase, combaseapi};

mod renderer;
mod app;
mod background;
mod sources;
mod gui;
mod utils;

use app::DBGM;
use utils::check_result;

fn main() -> Result<(), std::io::Error> {
    unsafe { check_result(combaseapi::CoInitializeEx(ptr::null_mut(), objbase::COINIT_APARTMENTTHREADED))?; }
    let mut dbgm = DBGM::new()?;
    let mut renderer = renderer::init("Desktop Background Manager");
    let mut gui = gui::GuiState::new(&mut dbgm, &mut renderer.render_sys.textures());
    renderer.main_loop(|run, ui, textures| *run = gui.update(ui, textures));
    unsafe { combaseapi::CoUninitialize(); }
    Ok(())
}