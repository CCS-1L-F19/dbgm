use std::path::PathBuf;

use super::ModalInterface;
use crate::gui::prelude::*;
use crate::utils::OptionExt;

pub struct ChangeSetInfo { image_folder: Option<PathBuf>, name_buf: ImString }
impl ModalInterface for ChangeSetInfo {
    fn id(&self) -> &str { "changesetinfo" }
    fn title(&self) -> &str { "Background set information" }
    fn display<T: Textures + ?Sized>(mut self, state: &mut GuiState, frame: Frame<T>) {
        let Frame { ui, .. } = frame;
        let set = state.set.as_mut().expect("Cannot view set information when no background set is open!");
        ui.input_text(im_str!("Name"), &mut self.name_buf).flags(imgui::ImGuiInputTextFlags::CallbackResize).build();
        ui.new_line();
        
        let display_folder = self.image_folder.deref().or(set.image_folder()).map(|f| f.to_string_lossy()).unwrap_or("(none)".into());
        ui.input_text(im_str!("Image folder"), &mut ImString::new(display_folder)).read_only(true).build();
        ui.same_line(0.0);
        if ui.button(im_str!("Choose..."), AUTO_SIZE) {
            match utils::nfd_handler(nfd::open_pick_folder(None), "image folder") {
                Ok(Some(path)) => self.image_folder = Some(path),
                Err(modal) => { state.open_modal(modal); return }
                _ => {},
            }
        }
        ui.new_line();

        if ui.button(im_str!("OK"), AUTO_SIZE) {
            if let Some(folder) = self.image_folder { set.set_image_folder(folder); }
            if self.name_buf.to_str().trim() != "" { set.set_name(self.name_buf.to_str().to_string()); }
            return
        }
        ui.same_line(0.0);
        if ui.button(im_str!("Cancel"), AUTO_SIZE) { return }
        state.open_modal(self)
    }
}

impl ChangeSetInfo {
    pub fn new() -> ChangeSetInfo {
        ChangeSetInfo { image_folder: None, name_buf: ImString::new("") }
    }
}