use crate::gui::prelude::*;
use modals::AddFolderSource;
use widgets::*;
use crate::background::{DesktopBackground, DesktopBackgroundFlags};

pub(super) struct Filter {
    pub show_edited: bool,
    pub show_excluded: bool,
}

impl Filter {
    pub fn should_display(&self, background: &DesktopBackground) -> bool {
        if background.excluded { return self.show_excluded; }
        self.show_edited || background.flags.contains(DesktopBackgroundFlags::UNEDITED)
    }
}

impl Default for Filter {
    fn default() -> Self {
        Filter { 
            show_edited: true,
            show_excluded: false,
        }
    }
}

impl<'a> GuiState<'a> {
    fn generate_background_entries<T: Textures + ?Sized>(&mut self, textures: &mut T) -> Vec<Vec<(usize, Option<CardOriginalInfo>)>> {
        use crate::sources::OriginalResult;
        match self.dbgm.background_set_mut() {
            Some(set) => {
                let mut entries = (0..set.sources.len()).map(|_| Vec::new()).collect::<Vec<_>>();
                let filter = &self.filter;
                for (id, background) in set.backgrounds.iter_mut().enumerate().filter(|(_, b)| filter.should_display(b)) {
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
                            Some(CardOriginalInfo {
                                texture: match self.image_cache.load_texture(&background.original, textures) {
                                    Some(Ok(texture)) => Some(texture),
                                    _ => None
                                },
                                location: original.location(),
                            })
                        },
                        _ => None,
                    };
                    entries[background.source].push((id, original));
                }
                entries
            }
            None => Vec::new()
        }
    }

    pub(super) fn draw_background_list<T: Textures + ?Sized>(&mut self, ui: &Ui, textures: &mut T) {
        let mut new_selected = None;
        let entries = self.generate_background_entries(textures);
        self.draw_list_header(ui);
        ChildWindow::new(im_str!("background list")).build(ui, || {
            if let Some(set) = self.dbgm.background_set_mut() {
                for (i, bgs) in entries.into_iter().enumerate().filter(|(_, bgs)| !bgs.is_empty()) {
                    if ui.collapsing_header(&im_str!("{}###Source{}", set.sources[i].name(), i)).build() {
                        for (id, original) in bgs.into_iter() {
                            let imgui_id = &im_str!("##Background{}", id);
                            let cursor_pos = ui.cursor_pos();
                            let alpha = ui.push_style_var(StyleVar::Alpha(0.0));
                            // This is a dummy element for us to check the hover state of.
                            Selectable::new(imgui_id).size(EditableBackgroundCard::size(ui)).build(ui);
                            alpha.pop(ui);
                            ui.set_cursor_pos(cursor_pos);

                            let selected = self.selected_background == Some(id);
                            let hovered_active = ui.is_item_hovered_with_flags(ItemHoveredFlags::ALLOW_WHEN_BLOCKED_BY_ACTIVE_ITEM);
                            let hovered = ui.is_item_hovered();
                            let down = ui.is_mouse_down(MouseButton::Left);
                            let release = ui.is_mouse_released(MouseButton::Left);

                            let background_color = match selected { 
                                true => StyleColor::FrameBg,
                                false => StyleColor::ChildBg, 
                            };

                            let border_color = match (hovered_active, down, selected) {
                                (true, true, _) => StyleColor::FrameBgActive,
                                (true, false, _) => StyleColor::FrameBgHovered,
                                (false, _, true) => StyleColor::FrameBgHovered,
                                (false, _, false) => StyleColor::Border,
                            };

                            if hovered && release { new_selected = Some(id); }
                            
                            let colors = ui.push_style_colors(&[
                                (StyleColor::Border, ui.style_color(border_color)),
                                (StyleColor::ChildBg, ui.style_color(background_color))
                            ]);
                            let card = EditableBackgroundCard {
                                id: imgui_id,
                                resources: &self.resources,
                                background: &mut set.backgrounds[id],
                                original,
                            };
                            card.draw(ui);
                            colors.pop(ui);
                        }
                    }
                }
            }
        });
        if let Some(id) = new_selected {
            self.select_background(id);
        }
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
                if ImageButton::new(self.resources.blue_plus.id, [height, height]).frame_padding(0).build(ui) {
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

    /*
    fn draw_background_entry<T: Textures + ?Sized>(
        &self, 
        ui: &Ui, 
        textures: &mut T, 
        id: usize,
        background: &mut DesktopBackground, 
        original: Option<OriginalEntry>,
    ) {
        let hsize = ui.content_region_max()[0];
        ChildWindow::new(&im_str!("Background{}", id))
            .border(true)
            .border_box(ui, [0.0, hsize * 0.2])
            .build(ui, || {
                ui.columns(2, im_str!("Columns"), true);
                let max_height = ui.content_region_max()[1];
                ui.set_current_column_width(max_height + ui.clone_style().window_padding[1] * 2.0); // no idea
                let texture = original.as_ref().and_then(|o| o.texture).unwrap_or(self.resources.missing_image);
                let dimensions = utils::fit_size(texture.size, [max_height, max_height]);
                ui.pad_to_center_v(dimensions[1]);
                Image::new(texture, dimensions).build(ui);
                ui.set_cursor_pos([0.0, max_height]);
                ui.next_column();
                ui.text(background.name);
                ui.text_disabled(original.as_ref().map(|o| o.location.as_str()).unwrap_or(""));
                
                let toolbar_size = [16.0, 16.0];
                let frame_padding = ui.clone_style().frame_padding;
                ui.set_cursor_pos([
                    ui.content_region_max()[0] - toolbar_size[0] - frame_padding[0] * 2.0, 
                    ui.content_region_max()[1] - toolbar_size[1] - frame_padding[1] * 2.0,
                ]);

                ImageButton::new(self.resources.white_x, [16.0, 16.0]).build_toggle(ui, &mut background.excluded);
            });
    }
    */
}