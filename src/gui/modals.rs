use std::borrow::Cow;
use std::fmt::Debug;
use std::path::PathBuf;

use enum_dispatch::*;

use imgui::*;

use crate::{OptionExt as _, };
use crate::gui::{AUTO_SIZE, GuiState};
use crate::gui::utils::{self, UiExt as _};
use crate::sources;

#[enum_dispatch]
pub trait ModalInterface {
    fn id(&self) -> &str;
    fn title(&self) -> &str;
    fn display(self, ui: &Ui, state: &mut GuiState) -> Option<Modal>;
    fn open_with<'ui, 'p>(&self, modal: PopupModal<'ui, 'p>) -> PopupModal<'ui, 'p> {
        modal.always_auto_resize(true)
    }
}

#[enum_dispatch(ModalInterface)]
pub enum Modal {
    ErrorModal,
    ChangeSetInfo,
    AddFolderSource,
}

impl Modal {
    pub fn error(message: impl AsRef<str>, error: Option<impl Debug + 'static>) -> Modal {
        Modal::from(ErrorModal { 
            message: message.as_ref().to_string(), 
            info: error.map(|d| Box::new(d) as Box<dyn Debug>) 
        })
    }

    pub fn change_set_info() -> Modal {
        Modal::from(ChangeSetInfo { image_folder: None, name_buf: ImString::new("") })
    }

    pub fn add_folder_source() -> Modal {
        Modal::from(AddFolderSource { folder: None, name_buf: ImString::new("") })
    }
}

pub struct ErrorModal { message: String, info: Option<Box<dyn Debug>>}
impl ModalInterface for ErrorModal {
    fn id(&self) -> &str { "error" }
    fn title(&self) -> &str { "Error" }
    fn display(mut self, ui: &Ui, state: &mut GuiState) -> Option<Modal> {
        ui.text(im_str!("{} {}", self.message, self.info.as_ref().map(|e| format!("Details: {:?}", e)).unwrap_or("".to_string())));
        let ok_label = im_str!("OK");
        match ui.button(ok_label, AUTO_SIZE) {
            true => None,
            false => Some(Modal::from(self))
        }
    }
}

pub struct ChangeSetInfo { image_folder: Option<PathBuf>, name_buf: ImString }
impl ModalInterface for ChangeSetInfo {
    fn id(&self) -> &str { "changesetinfo" }
    fn title(&self) -> &str { "Background set information" }
    fn display(mut self, ui: &Ui, state: &mut GuiState) -> Option<Modal> {
        let set = state.dbgm.background_set_mut().expect("Cannot view set information when no background set is open!");
        ui.input_text(im_str!("Name"), &mut self.name_buf).flags(imgui::ImGuiInputTextFlags::CallbackResize).build();
        ui.new_line();
        
        let display_folder = self.image_folder.deref().or(set.image_folder()).map(|f| f.to_string_lossy()).unwrap_or(Cow::from("(none)"));
        ui.input_text(im_str!("Image folder"), &mut ImString::new(display_folder)).read_only(true).build();
        ui.same_line(0.0);
        if ui.button(im_str!("Choose..."), AUTO_SIZE) {
            match utils::choose_folder("image folder") {
                Ok(Some(path)) => self.image_folder = Some(path),
                Err(modal) => return Some(modal),
                _ => {},
            }
        }
        ui.new_line();

        if ui.button(im_str!("OK"), AUTO_SIZE) {
            if let Some(folder) = self.image_folder { set.set_image_folder(folder); }
            if self.name_buf.to_str().trim() != "" { set.set_name(self.name_buf.to_str().to_string()); }
            return None
        }
        ui.same_line(0.0);
        if ui.button(im_str!("Cancel"), AUTO_SIZE) { return None }
        Some(Modal::from(self))
    }
}

pub struct AddFolderSource { folder: Option<PathBuf>, name_buf: ImString }
impl ModalInterface for AddFolderSource {
    fn id(&self) -> &str { "addfoldersource" }
    fn title(&self) -> &str { "Add source from folder..." }
    fn display(mut self, ui: &Ui, state: &mut GuiState) -> Option<Modal> {
        let set = state.dbgm.background_set_mut().expect("Cannot add a source when no background set is open!");

        let display_folder = self.folder.deref().or(set.image_folder()).map(|f| f.to_string_lossy()).unwrap_or(Cow::from("(none)"));
        ui.input_text(im_str!("Source folder"), &mut ImString::new(display_folder)).read_only(true).build();
        ui.same_line(0.0);
        if ui.button(im_str!("Choose..."), AUTO_SIZE) {
            match utils::choose_folder("source folder") {
                Ok(Some(path)) => self.folder = Some(path),
                Err(modal) => return Some(modal),
                _ => {},
            }
        }

        ui.input_text(im_str!("Source name"), &mut self.name_buf).flags(imgui::ImGuiInputTextFlags::CallbackResize).build();

        let is_ok = self.folder.is_some() && self.name_buf.to_str().trim().len() > 0;
        if ui.button_hack(im_str!("OK"), AUTO_SIZE, is_ok) {
            let new_source = sources::FolderSource::new(self.folder.unwrap(), self.name_buf.to_str());
            let source_id = set.add_source(new_source);
            let changes = set.sources_mut()[source_id].reload();
            return None
        }
        ui.same_line(0.0);
        if ui.button(im_str!("Cancel"), AUTO_SIZE) { return None }
        Some(Modal::from(self))
    }
}