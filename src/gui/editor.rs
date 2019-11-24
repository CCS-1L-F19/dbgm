use crate::utils::{OptionExt as _, Flatten as _};
use crate::gui::prelude::*;
use crate::background::*;

use widgets::croppable_image::*;

const INFO_HEIGHT: f32 = 100.0;
const CLIPPING_ADJUSTMENT: f32 = 1.0; // The clipping area of a ChildWindow is asymmetrical for some reason.

struct EditorState {
    selected_background: usize,

}

impl GuiState {
    pub(super) fn draw_editor<T: Textures + ?Sized>(&mut self, frame: Frame<T>, background: usize) {
        // We make this window one pixel bigger because the clipping region is asymmetric for some reason.
        let size = frame.ui.content_region_avail(); 
        ChildWindow::new(im_str!("BackgroundEditor")).size([size[0] + CLIPPING_ADJUSTMENT, size[1]]).build(frame.ui, || {
            let ui = frame.ui;
            self.draw_image(reborrow_frame!(frame), background);
            ui.separator();
            self.draw_info(reborrow_frame!(frame), background);
        });   
    }
    
    fn draw_image<T: Textures + ?Sized>(&mut self, frame: Frame<T>, background: usize) {
        let Frame { ui, textures, resources } = frame;
        let ActiveSet { set, image_cache } = self.set.as_mut().expect("Cannot edit when no background set is open!");
        let mut background = &mut set.backgrounds[background];
        let original = set.sources[background.source].original(&background.original);
        if let Some(original) = original.as_option() {
            if !image_cache.contains_image(&background.original) {
                if let Ok(image) = background.try_read_image_from(original) {
                    image_cache.insert_image(background.original.clone(), image);
                }
            }
        }
        let texture = image_cache.load_texture(&background.original, textures).map(|o| o.ok()).flatten().unwrap_or(resources.missing_image);
        let avail = Vec2::from(ui.content_region_avail()) - [IMAGE_BORDER_WIDTH, INFO_HEIGHT + IMAGE_BORDER_WIDTH];
        let size = utils::fit_size(texture.size, avail);
        let offset = (avail - size) / 2.0;
        ui.move_cursor(offset.into());
        let mut crop_region = CropRegion::new_centered(texture.size, texture.size);
        crop_region.scale *= 0.5;
        CroppableImage::new(texture, size).build(ui, &mut crop_region);
        ui.move_cursor([0.0, offset.y]);
    }

    fn draw_info<T: Textures + ?Sized>(&mut self, frame: Frame<T>, background: usize) {
        let Frame { ui, textures, resources } = frame;
        let set = self.set.as_mut().expect("Cannot edit when no background set is open!");
        let mut background = &mut set.backgrounds[background];
        let mut buf = ImString::new(&background.name);
        let header = if !background.flags.contains(DesktopBackgroundFlags::ORIGINAL_UNAVAILABLE) {
            let size = background.size.value().expect("Size should not be missing!");
            format!("{} - [JPEG Image, {} x {} pixels]", background.name, size.0, size.1)
        } else {
            format!("{} - (original unavailable)", background.name)
        };
        ui.text(header);
        if ui.input_text(im_str!("Name"), &mut buf).flags(ImGuiInputTextFlags::CallbackResize).build() {
            background.name = buf.to_str().to_owned();
        }
        ui.input_text(im_str!("Location"), &mut ImString::new(&background.location)).read_only(true).build();
    }
}