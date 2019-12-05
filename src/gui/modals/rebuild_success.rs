use super::ModalInterface;
use crate::gui::prelude::*;
use crate::background::SkipReason;

pub struct RebuildSuccess {
    skipped: Vec<(usize, SkipReason)>,
}

impl ModalInterface for RebuildSuccess {
    fn id(&self) -> &str { "rebuildsuccess" }
    fn title(&self) -> &str { "Image folder successfully rebuilt." }
    fn display<T: Textures + ?Sized>(self, state: &mut GuiState, frame: Frame<T>) {
        let set = state.set.as_ref().expect("Cannot rebuild the image folder when no background set is open!");
        let Frame { ui, .. } = frame;
        let bg_count = set.backgrounds.num_elements() - self.skipped.len();
        ui.text(im_str!("The image folder was successfully rebuilt. It now contains {} backgrounds.", bg_count));

        if self.skipped.len() > 0 {
            ui.separator();
            ui.text(im_str!("The following {} backgrounds were skipped.", self.skipped.len()));
            let mut skipped_info = ImString::new(self.skipped.iter().map(|(id, reason)| {
                format!("{}: {}", set.backgrounds[*id].name, match reason {
                    SkipReason::Excluded => "The background is excluded from the set.",
                    SkipReason::CorruptImage(_) => "The image file is corrupt or inaccessible.",
                    SkipReason::OriginalUnavailable => "The background's original is unavailable."
                })
            }).collect::<Vec<_>>().join("\n"));
            ui.input_text_multiline(im_str!("###SkippedBgs"), &mut skipped_info, AUTO_SIZE).read_only(true).build();
        }
        
        let ok_label = im_str!("OK");
        if ui.button(ok_label, AUTO_SIZE) { return }
        state.open_modal(self)
    }
}

impl RebuildSuccess {
    pub fn new(skipped: Vec<(usize, SkipReason)>) -> RebuildSuccess {
        RebuildSuccess { skipped }
    }
}