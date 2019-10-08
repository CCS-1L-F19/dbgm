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
        use crate::sources::OriginalResult;
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
        self.draw_list_header(ui);
        ChildWindow::new(im_str!("background list")).build(ui, || {
            if let Some(set) = self.dbgm.background_set() {
                for (i, bgs) in entries.into_iter().enumerate() {
                    if !bgs.is_empty() {
                        let source = &set.sources()[i];
                        if ui.collapsing_header(&im_str!("{}###Source{}", source.name(), i)).build() {
                            for (j, bg) in bgs.into_iter().enumerate() {
                                self.draw_background_entry(ui, (i, j), bg);
                            }
                        }
                    }   
                }
            }
        })
    }

    fn draw_list_header(&mut self, ui: &Ui) {
        let style = ui.clone_style();
        let header_text = im_str!("Background sources");
        let text_height = ui.calc_text_size(header_text, false, -1.0)[1];
        let mut bgcolor = Some(ui.push_style_color(StyleColor::ChildBg, ui.style_color(StyleColor::Header)));
        let mut padding = Some(ui.push_style_var(StyleVar::WindowPadding(style.frame_padding)));
        ChildWindow::new(im_str!("ListHeader"))
            .size([0.0, text_height + 2.0 * style.frame_padding[1]])
            .flags(WindowFlags::ALWAYS_USE_WINDOW_PADDING)
            .build(ui, || { 
                bgcolor.take().map(|t| t.pop(ui));
                padding.take().map(|t| t.pop(ui));
                ui.text(header_text);
                ui.same_line(0.0);
                let height = ui.content_region_max()[1] - style.frame_padding[1];
                ui.set_cursor_pos([ui.content_region_max()[0] - height, ui.cursor_pos()[1]]);
                let bottom_right = [
                    ui.window_pos()[0] + ui.cursor_pos()[0] + height,
                    ui.window_pos()[1] + ui.cursor_pos()[1] + height
                ];
                let bcol = ui.push_style_colors(&[
                    (StyleColor::Button, [0.0, 0.0, 0.0, 0.0]),
                    (StyleColor::ButtonActive, [0.0, 0.0, 0.0, 0.0]),
                    (StyleColor::ButtonHovered, [0.0, 0.0, 0.0, 0.0]),
                ]);
                if ImageButton::new(self.resources.blue_plus, [height, height]).frame_padding(0).build(ui) {
                    ui.open_popup(im_str!("AddSource"));
                }
                unsafe {
                    imgui::sys::igSetNextWindowPos(bottom_right.into(), Condition::Always as i32, [1.0, 0.0].into());
                }
                ui.popup(im_str!("AddSource"), || {
                    if Selectable::new(im_str!("From folder...")).build(ui) {
                        ui.close_current_popup();
                        self.open_modal(Modal::add_folder_source());
                    }
                });
                bcol.pop(ui);
            });
        // If the above ChildWindow was offscreen, these haven't been popped yet.
        bgcolor.map(|t| t.pop(ui)); 
        padding.map(|t| t.pop(ui));
    }

    fn draw_background_entry(&self, ui: &Ui, id: (usize, usize), background: BackgroundListEntry) {
        let entry_id = ui.push_id(&im_str!("Source{}Background{}", id.0, id.1));
        ChildWindow::new(im_str!("BackgroundFrame")).build(ui, || {
            ui.columns(2, im_str!("Columns"), false);
            let texture = background.original.and_then(|o| o.texture).unwrap_or(self.resources.missing_image);
            Image::new(texture, AUTO_SIZE).build(ui);
        });
        entry_id.pop(ui);
    }
}