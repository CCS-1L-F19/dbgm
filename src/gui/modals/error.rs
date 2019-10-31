use std::fmt::Debug;

use super::ModalInterface;
use crate::gui::prelude::*;

pub struct ErrorModal { message: String, info: Option<Box<dyn Debug>>}
impl ModalInterface for ErrorModal {
    fn id(&self) -> &str { "error" }
    fn title(&self) -> &str { "Error" }
    fn display(self, ui: &Ui, state: &mut GuiState) {
        ui.text(im_str!("{} {}", self.message, self.info.as_ref().map(|e| format!("Details: {:?}", e)).unwrap_or("".to_string())));
        let ok_label = im_str!("OK");
        if ui.button(ok_label, AUTO_SIZE) { return }
        state.open_modal(self)
    }
}

impl ErrorModal {
    pub fn new(message: impl AsRef<str>, error: Option<impl Debug + 'static>) -> ErrorModal {
        ErrorModal { 
            message: message.as_ref().to_string(), 
            info: error.map(|d| Box::new(d) as Box<dyn Debug>) 
        }
    }
}