use crate::gui::prelude::*;
use crate::background::{DesktopBackground, DesktopBackgroundFlags, Original};
use crate::sources::{OriginalResult, OriginalKey};

const ICON_SIZE: [f32; 2] = [16.0, 16.0];

pub struct CardOriginalInfo { pub texture: Option<Texture>, pub location: String }

// TODO: See about deduplicating some of the code in these two functions. There's only a slight semantic difference.
impl CardOriginalInfo {
    pub fn try_load_from_set<T: Textures + ?Sized>(set: &mut ActiveSet, id: usize, textures: &mut T) -> Option<CardOriginalInfo> {
        let ActiveSet { set, image_cache } = set;
        let background = &mut set.backgrounds[id];
        let original = set.sources[background.source].original(&background.original);
        if let OriginalResult::Original(original) = original {
            if !image_cache.contains_image(&background.original) {
                if let Ok(image) = background.try_read_image_from(original) {
                    image_cache.insert_image(background.original.clone(), image);
                }
            }
        }
        match original {
            OriginalResult::Original(original) | OriginalResult::ContentMismatch(original) => {
                Some(CardOriginalInfo {
                    texture: match image_cache.load_texture(&background.original, textures) {
                        Some(Ok(texture)) => Some(texture),
                        _ => None
                    },
                    location: original.location(),
                })
            },
            _ => None,
        }
    }

    pub fn load<T: Textures + ?Sized>(
        background: &mut DesktopBackground,
        original: &dyn Original, 
        image_cache: &mut ImageCache<OriginalKey>, 
        textures: &mut T
    ) -> CardOriginalInfo {
        if !image_cache.contains_image(&background.original) {
            if let Ok(image) = background.try_read_image_from(original) {
                image_cache.insert_image(background.original.clone(), image);
            }
        }
        CardOriginalInfo {
            texture: match image_cache.load_texture(&background.original, textures) {
                Some(Ok(texture)) => Some(texture),
                _ => None
            },
            location: original.location(),
        }
    }
}

#[must_use]
pub struct BackgroundCard<'i, 'c> {
    pub id: &'i ImStr,
    pub resources: &'c GuiResources,
    pub background: &'c DesktopBackground,
    pub original: Option<CardOriginalInfo>,
    pub editable: bool,
    pub width: f32,
}

impl<'i, 'c> BackgroundCard<'i, 'c> {
    pub fn size(ui: &Ui, custom_width: f32) -> Vec2 {
        let style = ui.clone_style();
        let non_content = style.window_padding[1] + style.window_border_size;
        let line = ui.current_font_size() + style.item_spacing[1];
        [custom_width, non_content * 2.0 + line * 2.0 + ICON_SIZE[1]].into()
    }

    pub fn draw(self, ui: &Ui) -> DesktopBackgroundFlags {
        let BackgroundCard { id, resources, background, original, editable, width } = self;
        let mut flags = background.flags.clone();
        let original = original.as_ref();
        ChildWindow::new(id)
            .border(true)
            .size(BackgroundCard::size(ui, width).into())
            .build(ui, || {
                ui.set_cursor_pos(ui.window_content_region_min());
                ui.columns(2, im_str!("Columns"), true);

                let max_height = ui.content_region_avail()[1];
                ui.set_current_column_width(max_height + ui.clone_style().window_padding[1] * 2.0); // no idea
                let texture = original.and_then(|o| o.texture).unwrap_or(resources.missing_image);
                let dimensions = utils::fit_size(texture.size, [max_height, max_height]);
                ui.center_avail_v(dimensions.y);
                Image::new(texture.id, dimensions.into()).build(ui);
                // ui.set_cursor_pos([0.0, ui.window_content_region_max()[1]]);

                ui.next_column();

                ui.text(&background.name);
                ui.text_disabled(original.as_ref().map(|o| o.location.as_str()).unwrap_or(""));

                let highlight = match editable {
                    true => ui.style_color(StyleColor::ScrollbarGrab),
                    false => [0.0, 0.0, 0.0, 0.0]
                };

                let bcol = ui.push_style_colors(&[
                    (StyleColor::Button, [0.0, 0.0, 0.0, 0.0]),
                    (StyleColor::ButtonActive, highlight),
                    (StyleColor::ButtonHovered, highlight),
                ]);
                let frame_padding = ui.push_style_var(StyleVar::FramePadding([0.0, 0.0]));

                if flags.contains(DesktopBackgroundFlags::ORIGINAL_UNAVAILABLE) {
                    Image::new(resources.unavailable.id, ICON_SIZE).build(ui);
                    ui.same_line(0.0);
                }

                let icon = if flags.contains(DesktopBackgroundFlags::UNEDITED) { resources.unedited } else { resources.edited };
                if ImageButton::new(icon.id, ICON_SIZE).build(ui) && editable {
                    flags.toggle(DesktopBackgroundFlags::UNEDITED);
                }

                ui.same_line(0.0);

                let icon = if flags.contains(DesktopBackgroundFlags::EXCLUDED) { resources.hidden } else { resources.not_hidden };
                if ImageButton::new(icon.id, ICON_SIZE).build(ui) && editable {
                    flags.toggle(DesktopBackgroundFlags::EXCLUDED);
                }

                frame_padding.pop(ui);
                bcol.pop(ui);
            });
        flags
    }
}