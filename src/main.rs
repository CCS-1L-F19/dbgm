use std::ptr;

use winapi::um::{objbase, combaseapi};

mod renderer;
mod background;
mod sources;
mod gui;
mod utils;
mod math;

use utils::check_result;

fn main() -> Result<(), std::io::Error> {
    unsafe { check_result(combaseapi::CoInitializeEx(ptr::null_mut(), objbase::COINIT_APARTMENTTHREADED))?; }
    let mut renderer = renderer::init("Desktop Background Manager");
    let mut state = gui::GuiState::default();
    let resources = gui::GuiResources::load(&mut renderer.render_sys.textures());
    renderer.main_loop(|run, ui, textures| {
        let frame = gui::draw::Frame { ui, textures, resources: &resources };
        *run = gui::draw::draw_state(&mut state, frame);
    });
    unsafe { combaseapi::CoUninitialize(); }
    Ok(())
}