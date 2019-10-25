use imgui::*;
use super::{
    GuiState, AUTO_SIZE, utils::{self, Textures, *}, modals::AddFolderSource
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
        match self.dbgm.background_set_mut() {
            Some(set) => {
                let mut entries = (0..set.sources.len()).map(|_| Vec::new()).collect::<Vec<_>>();
                for background in set.backgrounds.iter_mut() {
                    let original = set.sources[background.source].original(&background.original);
                    if let OriginalResult::Original(original) = original {
                        if !self.image_cache.contains_image(&background.original) {
                            if let Ok(image) = background.try_read_image_from(original) {
                                self.image_cache.insert_image(background.original.clone(), image);
                            }
                        }
                    }
                    let original = match original {
                        OriginalResult::Original(original) | OriginalResult::ContentMismatch(original) => {
                            Some(OriginalEntry {
                                texture: match self.image_cache.load_texture(&background.original, textures) {
                                    Some(Ok(texture)) => Some(texture),
                                    _ => None
                                },
                                location: original.location(),
                                changed: false,
                            })
                        },
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
                                self.draw_background_entry(ui, textures, (i, j), bg);
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
                        self.open_modal(AddFolderSource::new());
                    }
                });
                bcol.pop(ui);
            });
        // If the above ChildWindow was offscreen, these haven't been popped yet.
        bgcolor.map(|t| t.pop(ui)); 
        padding.map(|t| t.pop(ui));
    }

    fn draw_background_entry<T: Textures + ?Sized>(&self, ui: &Ui, textures: &mut T, id: (usize, usize), background: BackgroundListEntry) {
        let entry_id = ui.push_id(&im_str!("Source{}Background{}", id.0, id.1));
        let hsize = ui.content_region_max()[0];
        ChildWindow::new(im_str!("BackgroundFrame")).border(true).border_box(ui, [0.0, hsize * 0.2]).build(ui, || {
            ui.columns(2, im_str!("Columns"), true);
            let max_height = ui.content_region_max()[1];
            ui.set_current_column_width(max_height + ui.clone_style().window_padding[1] * 2.0); // no idea
            let texture = background.original.and_then(|o| o.texture).unwrap_or(self.resources.missing_image);
            let dimensions = utils::fit_size(textures.texture_info(texture).unwrap().size, [max_height, max_height]);
            ui.pad_to_center_v(dimensions[1]);
            Image::new(texture, dimensions).build(ui);
            ui.set_cursor_pos([0.0, max_height]);
            ui.next_column();
            ui.text(background.name);
        });
        entry_id.pop(ui);
    }

    
}