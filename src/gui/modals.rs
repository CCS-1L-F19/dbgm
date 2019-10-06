use std::borrow::Cow;
use std::fmt::Debug;
use std::path::PathBuf;

use imgui::*;

use crate::{OptionExt as _};
use super::{AUTO_SIZE, GuiState};
use self::Modal::*;
use enum_dispatch::*;

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
        Modal::from(ChangeSetInfo { image_folder: None, name_buf: ImString::new("") }) // TODO: Add proper resizing support
    }

    pub fn add_folder_source() -> Modal {
        Modal::from(AddFolderSource { })
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
            match nfd::open_pick_folder(None) {
                Ok(nfd::Response::Okay(f)) => {
                    match f.parse() {
                        Ok(path) => self.image_folder = Some(path),
                        Err(e) => return Some(Modal::error("Invalid path to image folder.".to_string(), Some(e))),
                    }
                }
                Err(e) => return Some(Modal::error("Could not open image folder picker.".to_string(), Some(e))),
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

pub struct AddFolderSource { }
impl ModalInterface for AddFolderSource {
    fn id(&self) -> &str { "addfoldersource" }
    fn title(&self) -> &str { "Add source from folder..." }
    fn display(self, ui: &Ui, state: &mut GuiState) -> Option<Modal> {

    }
}