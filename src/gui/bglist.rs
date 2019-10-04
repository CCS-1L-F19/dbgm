use imgui::*;
use super::{
    GuiState, AUTO_SIZE, utils::Textures, modals::Modal
};

struct BackgroundListEntry {
    name: String,
    source_id: usize,
    original: Option<OriginalEntry>,
}

struct OriginalEntry {
    texture: Option<TextureId>,
    location: String,
    changed: bool,
}

impl<'a> GuiState<'a> {
    fn generate_background_entries<T: Textures + ?Sized>(&mut self, textures: &mut T) -> Vec<Vec<BackgroundListEntry>> {
        use crate::source::OriginalResult;
        match self.dbgm.background_set() {
            Some(set) => {
                let mut entries = (0..set.sources().len()).map(|_| Vec::new()).collect::<Vec<_>>();
                for background in set.backgrounds() {
                    let original = match set.sources()[background.source].original(&background.original) {
                        OriginalResult::Original(original) => Some(OriginalEntry {
                            texture: {
                                if !self.image_cache.contains_image(&background.original) {
                                    if let Ok(image) = original.read_image() {
                                        self.image_cache.insert_image(background.original.clone(), image);
                                    }
                                }
                                match self.image_cache.load_texture(&background.original, textures) {
                                    Some(Ok(texture)) => Some(texture),
                                    _ => None
                                }
                            },
                            location: original.location(),
                            changed: false,
                        }),
                        OriginalResult::ContentMismatch(original) => Some(OriginalEntry {
                            texture: {
                                match original.read_image() {
                                    Ok(image) => self.image_cache.insert_image(background.original.clone(), image),
                                    Err(_) => { self.image_cache.remove_image(&background.original); }
                                }
                                match self.image_cache.load_texture(&background.original, textures) {
                                    Some(Ok(texture)) => Some(texture),
                                    _ => None
                                }
                            },
                            location: original.location(),
                            changed: true,
                        }),
                        _ => None,
                    };
                    entries[background.source].push(BackgroundListEntry { 
                        name: background.name.clone(), 
                        source_id: background.source, 
                        original: original 
                    });
                }
                entries
            }
            None => Vec::new()
        }
    }

    pub(super) fn draw_background_list<T: Textures + ?Sized>(&mut self, ui: &Ui, textures: &mut T) {
        let entries = self.generate_background_entries(textures);
        ChildWindow::new(im_str!("background list")).border(true).build(ui, || {
            if let Some(set) = self.dbgm.background_set() {
                let combo = ComboBox::new(im_str!("###Add source"))
                    .flags(ComboBoxFlags::NO_ARROW_BUTTON)
                    .preview_value(im_str!("Add source"))
                    .build(ui, || {
                        if Selectable::new(im_str!("From folder...")).build(ui) {

                        }
                    });
                ui.same_line(0.0);
                for (i, bgs) in entries.into_iter().enumerate() {
                    if !bgs.is_empty() {
                        let source = &set.sources()[i];
                        if ui.collapsing_header(&im_str!("{}###Source{}", source.name(), i)).build() {
                            for (j, bg) in bgs.into_iter().enumerate() {
                                self.render_background_entry(ui, (i, j), bg);
                            }
                        }
                    }   
                }
            }
        })
    }

    fn render_background_entry(&self, ui: &Ui, id: (usize, usize), background: BackgroundListEntry) {
        let entry_id = ui.push_id(&im_str!("Source{}Background{}", id.0, id.1));
        let child = ChildWindow::new(im_str!("BackgroundFrame")).build(ui, || {
            ui.columns(2, im_str!("Columns"), false);
            let texture = background.original.and_then(|o| o.texture).unwrap_or(self.resources.missing_image);
            Image::new(texture, AUTO_SIZE).build(ui);
        });
        entry_id.pop(ui);
    }
}