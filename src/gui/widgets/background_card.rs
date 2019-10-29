use imgui::*;
use crate::gui::{utils::{self, *}, GuiResources};
use crate::background::{DesktopBackground, DesktopBackgroundFlags};

const ICON_SIZE: [f32; 2] = [16.0, 16.0];

pub struct CardOriginalInfo { pub texture: Option<Texture>, pub location: String }

pub struct EditableBackgroundCard<'i, 'c> {
    id: Id<'i>,
    resources: &'c GuiResources,
    background: &'c mut DesktopBackground,
    original: Option<CardOriginalInfo>,
}

impl<'i, 'c> EditableBackgroundCard<'i, 'c> {
    pub fn new(
        id: impl Into<Id<'i>>, 
        resources: &'c GuiResources,
        background: &'c mut DesktopBackground, 
        original: Option<CardOriginalInfo>,
    ) -> Self {
        EditableBackgroundCard { id: id.into(), background, resources, original }
    }

    pub fn heights(ui: &Ui) -> (f32, f32) {
        let style = ui.clone_style();
        let non_content = style.window_padding[1] + style.window_border_size;
        let line = ui.current_font_size() + style.item_spacing[1];
        (non_content * 2.0, line * 2.0 + ICON_SIZE[1])
    }

    pub fn draw(self, ui: &Ui) {
        let EditableBackgroundCard { id, resources, background, original } = self;
        let original = original.as_ref();
        let (non_content_height, content_height) = EditableBackgroundCard::heights(ui);
        ChildWindow::new(id)
            .border(true)
            .size([0.0, non_content_height + content_height])
            .build(ui, || {
                ui.set_cursor_pos(ui.window_content_region_min());
                ui.columns(2, im_str!("Columns"), true);

                let max_height = ui.content_region_avail()[1];
                ui.set_current_column_width(max_height + ui.clone_style().window_padding[1] * 2.0); // no idea
                let texture = original.and_then(|o| o.texture).unwrap_or(resources.missing_image);
                let dimensions = utils::fit_size(texture.size, [max_height, max_height]);
                ui.pad_to_center_v(dimensions[1]);
                Image::new(texture.id, dimensions).build(ui);
                // ui.set_cursor_pos([0.0, ui.window_content_region_max()[1]]);

                ui.next_column();

                ui.text(&background.name);
                ui.text_disabled(original.as_ref().map(|o| o.location.as_str()).unwrap_or(""));

                let highlight = ui.style_color(StyleColor::ScrollbarGrab);
                let bcol = ui.push_style_colors(&[
                    (StyleColor::Button, [0.0, 0.0, 0.0, 0.0]),
                    (StyleColor::ButtonActive, highlight),
                    (StyleColor::ButtonHovered, highlight),
                ]);

                if background.flags.contains(DesktopBackgroundFlags::ORIGINAL_UNAVAILABLE) {
                    
                }

                let texture = if background.excluded { resources.hidden } else { resources.not_hidden };
                if ImageButton::new(texture.id, ICON_SIZE).frame_padding(0).build(ui) {
                    background.excluded = !background.excluded;
                }

                bcol.pop(ui);
            });
    }
}