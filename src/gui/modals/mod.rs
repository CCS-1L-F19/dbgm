use enum_dispatch::*;

use crate::gui::prelude::*;

pub mod error;
pub mod change_set_info;
pub mod add_folder_source;
pub mod confirm_changes;
pub mod remove_source;
pub mod rebuild_success;

pub use error::ErrorModal;
pub use change_set_info::ChangeSetInfo;
pub use add_folder_source::AddFolderSource;
pub use confirm_changes::ConfirmChanges;
pub use remove_source::RemoveSource;
pub use rebuild_success::RebuildSuccess;

#[enum_dispatch]
pub trait ModalInterface {
    fn id(&self) -> &str;
    fn title(&self) -> &str;
    fn display<T: Textures + ?Sized>(self, state: &mut GuiState, frame: Frame<T>);
    fn open_with<'ui, 'p>(&self, _ui: &'ui Ui, modal: PopupModal<'ui, 'p>) -> PopupModal<'ui, 'p> {
        modal.always_auto_resize(true)
    }
}

#[enum_dispatch(ModalInterface)]
pub enum Modal {
    ErrorModal,
    ChangeSetInfo,
    AddFolderSource,
    ConfirmChanges,
    RemoveSource,
    RebuildSuccess,
}

impl GuiState {
    pub(super) fn open_modal(&mut self, modal: impl Into<Modal>) {
        self.modal = Some(modal.into());
    }

    pub(super) fn check_modal<T: Textures + ?Sized>(&mut self, frame: Frame<T>) {
        let ui = frame.ui;
        if let Some(modal) = self.modal.take() {
            let id = im_str!("###{}", modal.id()).to_owned();
            if !ui.is_popup_open(&id) { ui.open_popup(&id); }
            let id_with_title = im_str!("{}###{}", modal.title(), id.to_str());
            modal.open_with(ui, PopupModal::new(ui, &id_with_title)).build(ui, || modal.display(self, frame));
            match &self.modal {
                Some(m) if im_str!("###{}", m.id()) != id => {
                    ui.close_current_popup();
                    ui.open_popup(&im_str!("###{}", m.id()));
                }
                None => ui.close_current_popup(),
                _ => {},
            }
        }
    }
}