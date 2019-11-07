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
        if background.flags.contains(DesktopBackgroundFlags::EXCLUDED) { return self.show_excluded; }
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
        let mut operation = None;
        let entries = self.generate_background_entries(textures);
        self.draw_list_header(ui);
        ChildWindow::new(im_str!("BackgroundList")).build(ui, || {
            if let Some(set) = self.dbgm.background_set() {
                for (i, bgs) in entries.into_iter().enumerate().filter(|(_, bgs)| !bgs.is_empty()) {
                    let header_pos = ui.cursor_pos();
                    if ui.collapsing_header(&im_str!("{}###Source{}", set.sources[i].name(), i)).flags(ImGuiTreeNodeFlags::AllowItemOverlap).build() {
                        for (id, original) in bgs.into_iter() {
                            let imgui_id = &im_str!("##Background{}", id);
                            let cursor_pos = ui.cursor_pos();
                            let alpha = ui.push_style_var(StyleVar::Alpha(0.0));
                            // This is a dummy element for us to check the hover state of.
                            Selectable::new(imgui_id).size(BackgroundCard::size(ui, 0.0)).build(ui);
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

                            if hovered && release { operation = Some(Operation::SelectBackground(id)); }
                            
                            let colors = ui.push_style_colors(&[
                                (StyleColor::Border, ui.style_color(border_color)),
                                (StyleColor::ChildBg, ui.style_color(background_color))
                            ]);
                            let card = BackgroundCard {
                                id: imgui_id,
                                resources: &self.resources,
                                background: &set.backgrounds[id],
                                original,
                                editable: true,
                                width: 0.0,
                            };
                            let new_flags = card.draw(ui);
                            if new_flags != set.backgrounds[id].flags {
                                operation = Some(Operation::ChangeFlags(id, new_flags));
                            }
                            colors.pop(ui);
                        }
                    }
                    let next_item = ui.cursor_pos();
                    
                    let bcol = ui.push_style_colors(&[
                        (StyleColor::Button, [0.0, 0.0, 0.0, 0.0]),
                        (StyleColor::ButtonActive, [0.0, 0.0, 0.0, 0.0]),
                        (StyleColor::ButtonHovered, [0.0, 0.0, 0.0, 0.0]),
                    ]);

                    let num_buttons = 2;
                    let style = ui.clone_style();
                    let base_size = ui.current_font_size();
                    let padding_correction = style.frame_padding[0] - style.item_spacing[0];
                    
                    let mut current_button = 0;
                    let mut toolbar_button = |texture, scale| {
                        let x_center = ui.content_region_max()[0] - padding_correction - (
                            ((num_buttons - current_button) as f32 * (base_size + style.item_spacing[0])) - 0.5 * base_size
                        );
                        let y = header_pos[1] + style.frame_padding[1] - base_size * (scale - 1.0) / 2.0;
                        let icon_size = scale * base_size;
                        ui.set_cursor_pos([x_center - icon_size / 2.0, y]);
                        current_button += 1;
                        ImageButton::new(texture, [icon_size, icon_size]).frame_padding(0).build(ui)
                    };

                    if toolbar_button(self.resources.reload_small.id, 1.15) {
                        operation = Some(Operation::ReloadSource(i));
                    }

                    if toolbar_button(self.resources.blue_x.id, 1.0) {
                        operation = Some(Operation::RemoveSource(i));
                    }

                    bcol.pop(ui);
                    ui.set_cursor_pos(next_item);
                }
            }
        });
        operation.map(|op| op.apply(self));
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
                let buttons_width = 2.0 * (height + style.item_spacing[0]) - style.item_spacing[0];
                ui.set_cursor_pos([ui.content_region_max()[0] - buttons_width, ui.cursor_pos()[1]]);
                let bcol = ui.push_style_colors(&[
                    (StyleColor::Button, [0.0, 0.0, 0.0, 0.0]),
                    (StyleColor::ButtonActive, [0.0, 0.0, 0.0, 0.0]),
                    (StyleColor::ButtonHovered, [0.0, 0.0, 0.0, 0.0]),
                ]);
                ImageDropdown::new(im_str!("FilterBackgrounds"), self.resources.filter.id, [height, height]).frame_padding(0).build(ui, || {
                    ui.checkbox(im_str!("Show edited"), &mut self.filter.show_edited);
                    ui.checkbox(im_str!("Show excluded"), &mut self.filter.show_excluded);
                });
                ui.same_line(0.0);
                ImageDropdown::new(im_str!("AddSource"), self.resources.blue_plus.id, [height, height]).frame_padding(0).build(ui, || {
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
}