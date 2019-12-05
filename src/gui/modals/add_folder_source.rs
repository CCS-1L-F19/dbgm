use std::path::PathBuf;

use super::ModalInterface;
use crate::gui::prelude::*;
use crate::sources::FolderSource;
use crate::utils::OptionExt;

pub struct AddFolderSource { folder: Option<PathBuf>, name_buf: ImString }
impl ModalInterface for AddFolderSource {
    fn id(&self) -> &str { "addfoldersource" }
    fn title(&self) -> &str { "Add source from folder..." }
    fn display<T: Textures + ?Sized>(mut self, state: &mut GuiState, frame: Frame<T>) {
        let Frame { ui, .. } = frame;
        
        let display_folder = self.folder.deref().map(|f| f.to_string_lossy()).unwrap_or("(none)".into());
        ui.input_text(im_str!("Source folder"), &mut ImString::new(display_folder)).read_only(true).build();
        ui.same_line(0.0);
        if ui.button(im_str!("Choose..."), AUTO_SIZE) {
            match utils::nfd_handler(nfd::open_pick_folder(None), "source folder") {
                Ok(Some(path)) => self.folder = Some(path),
                Err(modal) => { state.open_modal(modal); return }
                _ => {},
            }
        }

        ui.input_text(im_str!("Source name"), &mut self.name_buf).flags(imgui::ImGuiInputTextFlags::CallbackResize).build();

        let is_ok = self.folder.is_some() && self.name_buf.to_str().trim().len() > 0;
        if ui.button_hack(im_str!("OK"), AUTO_SIZE, is_ok) {
            state.add_source(FolderSource::new(self.folder.unwrap(), self.name_buf.to_str()));
            return
        }
        ui.same_line(0.0);
        if ui.button(im_str!("Cancel"), AUTO_SIZE) { return }
        state.open_modal(self)
    }
}

impl AddFolderSource {
    pub fn new() -> AddFolderSource {
        AddFolderSource { folder: None, name_buf: ImString::new("") }
    }
}