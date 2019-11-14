use super::ModalInterface;
use crate::gui::prelude::*;

use widgets::{BackgroundGrid, CardOriginalInfo};

pub struct RemoveSource(pub usize);
impl ModalInterface for RemoveSource {
    fn id(&self) -> &str { "removesource" }
    fn title(&self) -> &str { "Confirm source removal" }
    fn display<T: Textures + ?Sized>(self, state: &mut GuiState, frame: Frame<T>) { 
        let Frame { ui, .. } = frame;
        let set = state.set.as_mut().expect("Cannot remove a source when no background set is open!");
        let source = &set.sources[self.0];
        ui.text(format!("Are you sure you want to remove the source '{}'?", source.name()));

        let textures = &mut *frame.textures;
        let affected_backgrounds = set.backgrounds.indices().collect::<Vec<_>>().into_iter().filter_map(|id| {
            if set.backgrounds[id].source != self.0 { return None };
            Some((id, CardOriginalInfo::try_load(set, id, textures)))
        }).collect::<Vec<_>>();
       
        if !affected_backgrounds.is_empty() {
            ui.text("If you click Remove, the following backgrounds will be permanently removed from the set:");
            ui.spacing();
            let card_width = ui.current_font_size() * 25.0; // TODO: Is there a less arbitrary choice here
            let grid = BackgroundGrid {
                id: &im_str!("AffectedBackgrounds"),
                entries: affected_backgrounds,
                card_width: card_width,
                max_size: AUTO_SIZE,
            };
            ui.center_h(grid.size(ui)[0]);
            grid.draw(set, reborrow_frame!(frame));
        }

        ui.spacing();
        
        let mut decision = None;
        if ui.button(im_str!("Remove"), AUTO_SIZE) { decision = Some(true); }
        ui.same_line(0.0);
        if ui.button(im_str!("Cancel"), AUTO_SIZE) { decision = Some(false); }

        match decision {
            Some(true) => {
                if let Some(b) = state.selected_background {
                    if set.backgrounds[b].source == self.0 {
                        state.selected_background = None;
                    }
                }
                set.remove_source(self.0);
            }
            Some(false) => return,
            None => state.open_modal(self),
        }
    }
}