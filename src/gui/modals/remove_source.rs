use super::ModalInterface;
use crate::gui::prelude::*;

pub struct RemoveSource(pub usize);
impl ModalInterface for RemoveSource {
    fn id(&self) -> &str { "removesource" }
    fn title(&self) -> &str { "Confirm source removal" }
    fn display(mut self, ui: &Ui, state: &mut GuiState) { 
        let set = state.dbgm.background_set_mut().expect("Cannot remove a source when no background set is open!");
        let source = &set.sources[self.0];
        ui.text(format!("Are you sure you want to remove the source '{}'?", source.name()));
        ui.text("If you click Remove, the following backgrounds will be permanently removed from the set:");
        // TODO: Display backgrounds

        let mut decision = None;
        if ui.button(im_str!("Remove"), AUTO_SIZE) { decision = Some(true); }
        ui.same_line(0.0);
        if ui.button(im_str!("Cancel"), AUTO_SIZE) { decision = Some(false); }

        match decision {
            Some(true) => {
                unimplemented!()
            }
            Some(false) => return,
            None => state.open_modal(self),
        }
    }
}