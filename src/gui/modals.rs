use std::borrow::Cow;
use std::fmt::Debug;
use std::path::PathBuf;

use imgui::*;

use crate::{OptionExt as _};
use super::{AUTO_SIZE, GuiState};
use self::Modal::*;

pub enum Modal {
    Error(String, Option<Box<dyn Debug>>),
    ChangeSetInfo {
        image_folder: Option<PathBuf>,
        name_buf: ImString,
    },
    AddFileSource {

    },
}

impl Modal {
    pub fn get_id(&self) -> &str {
        match self {
            Error(..) => "error",
            ChangeSetInfo { .. } => "changesetinfo",
            AddFileSource { .. } => "addfilesource",
        }
    }

    pub fn get_title(&self) -> &str {
        match self {
            Error(..) => "Error",
            ChangeSetInfo { .. } => "Background set information",
        }
    }

    pub fn open_with<'ui, 'p>(&self, modal: PopupModal<'ui, 'p>) -> PopupModal<'ui, 'p> {
        modal.always_auto_resize(true)
    }

    pub fn display(mut self, ui: &Ui, state: &mut GuiState) -> Option<Modal> {
        match &mut self {
            Error(message, info) => {
                ui.text(im_str!("{} {}", message, info.as_ref().map(|e| format!("Details: {:?}", e)).unwrap_or("".to_string())));
                let ok_label = im_str!("OK");
                if ui.button(ok_label, AUTO_SIZE) {
                    return None
                }
            }
            ChangeSetInfo { image_folder, name_buf } => {
                let set = state.dbgm.background_set_mut().expect("Cannot view set information when no background set is open!");
                ui.input_text(im_str!("Name"), name_buf).flags(imgui::ImGuiInputTextFlags::CallbackResize).build();
                ui.new_line();
                
                let display_folder = image_folder.deref().or(set.image_folder()).map(|f| f.to_string_lossy()).unwrap_or(Cow::from("(none)"));
                ui.input_text(im_str!("Image folder"), &mut ImString::new(display_folder)).read_only(true).build();
                ui.same_line(0.0);
                if ui.button(im_str!("Choose..."), AUTO_SIZE) {
                    match nfd::open_pick_folder(None) {
                        Ok(nfd::Response::Okay(f)) => {
                            match f.parse() {
                                Ok(path) => *image_folder = Some(path),
                                Err(e) => return Some(Modal::error("Invalid path to image folder.".to_string(), Some(e))),
                            }
                        }
                        Err(e) => return Some(Modal::error("Could not open image folder picker.".to_string(), Some(e))),
                        _ => {},
                    }
                }
                ui.new_line();

                if ui.button(im_str!("OK"), AUTO_SIZE) {
                    if let Some(folder) = image_folder { set.set_image_folder(folder); }
                    if name_buf.to_str().trim() != "" { set.set_name(name_buf.to_str().to_string()); }
                    return None
                }
                ui.same_line(0.0);
                if ui.button(im_str!("Cancel"), AUTO_SIZE) { return None }
            }
        }
        return Some(self)
    }
}

impl Modal {
    pub fn error(message: impl AsRef<str>, error: Option<impl Debug + 'static>) -> Modal {
        Error(message.as_ref().to_string(), error.map(|d| Box::new(d) as Box<dyn Debug>))
    }

    pub fn change_set_info() -> Modal {
        ChangeSetInfo { image_folder: None, name_buf: ImString::new("") } // TODO: Add proper resizing support
    }

    pub fn add_file_source() -> Modal {
        AddFileSource { }
    }
}