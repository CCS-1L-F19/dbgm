use super::ModalInterface;
use crate::gui::prelude::*;

use widgets::BackgroundGrid;

pub struct RemoveSource(pub usize);
impl ModalInterface for RemoveSource {
    fn id(&self) -> &str { "removesource" }
    fn title(&self) -> &str { "Confirm source removal" }
    fn display(self, ui: &Ui, state: &mut GuiState) { 
        let set = state.dbgm.background_set_mut().expect("Cannot remove a source when no background set is open!");
        let source = &set.sources[self.0];
        ui.text(format!("Are you sure you want to remove the source '{}'?", source.name()));
        ui.text("If you click Remove, the following backgrounds will be permanently removed from the set:");

        let backgrounds = set.backgrounds.iter_mut().filter(|b| b.source == self.0).map(|b| {
            state.load_background
        })

        let grid = BackgroundGrid {
            id: &im_str!("AffectedBackgrounds"),
            entries: unimplemented!(),
            card_width: unimplemented!(),
            max_dimensions: [5, 7],
        };

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