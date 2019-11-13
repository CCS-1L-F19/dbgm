use std::collections::HashMap;
use std::hash::Hash;
use std::path::PathBuf;

use imgui::*;

use crate::renderer::{Textures, Texture};
use super::modals::ErrorModal;

pub const AUTO_SIZE: [f32; 2] = [0.0, 0.0];

pub struct ImageCache<K: Hash + Eq> {
    images: HashMap<K, (image::DynamicImage, Option<Texture>)>,
}

impl<K: Hash + Eq> ImageCache<K> {
    pub fn new() -> Self { 
        ImageCache { images: HashMap::new() }
    }

    pub fn contains_image(&self, key: &K) -> bool {
        self.images.contains_key(key)
    }

    pub fn insert_image(&mut self, key: K, image: image::DynamicImage) {
        self.images.insert(key, (image, None));
    }

    pub fn remove_image(&mut self, key: &K) -> Option<(image::DynamicImage, Option<Texture>)> {
        self.images.remove(key)
    }

    pub fn get_image(&self, key: &K) -> Option<&image::DynamicImage> {
        self.images.get(key).map(|(i, _)| i)
    }

    pub fn load_texture<T: Textures + ?Sized>(&mut self, key: &K, textures: &mut T) -> Option<Result<Texture, T::CreationError>> {
        match self.images.get_mut(key) {
            Some((_, Some(texture))) => Some(Ok(*texture)),
            Some((image, texture_slot)) => {
                let texture = match textures.create_texture(image) {
                    Ok(texture) => texture,
                    Err(e) => return Some(Err(e))
                };
                *texture_slot = Some(texture);
                self.load_texture(key, textures)
            },
            None => None
        }
    }
}

pub trait UiExt {
    fn center_h(&self, width: f32);
    fn center_v(&self, height: f32);
    fn center_avail_h(&self, width: f32);
    fn center_avail_v(&self, height: f32);
    fn is_popup_open(&self, popup: &ImStr) -> bool;
    fn button_hack(&self, label: &ImStr, size: [f32; 2], enabled: bool) -> bool;
    fn toggle_button_labeled(&self, id: &ImStr, text_on: &str, text_off: &str, pressed: &mut bool);
    fn small_toggle_button(&self, label: &ImStr, pressed: &mut bool) -> bool;
    fn move_cursor(&self, amount: [f32; 2]);
    fn fullscreen_window(&self, title: &ImStr, contents: impl FnOnce());
}

impl<'ui> UiExt for Ui<'ui> {
    fn center_h(&self, width: f32) {
        self.set_cursor_pos([(self.content_region_max()[0] - self.window_content_region_min()[0] - width) / 2.0, self.cursor_pos()[1]]);
    }

    fn center_v(&self, height: f32) {
        self.set_cursor_pos([self.cursor_pos()[0], (self.content_region_max()[1] - self.window_content_region_min()[1] - height) / 2.0]);
    }

    fn center_avail_h(&self, width: f32) {
        self.move_cursor([(self.content_region_avail()[0] - width) / 2.0, 0.0]);
    }

    fn center_avail_v(&self, height: f32) {
        self.move_cursor([0.0, (self.content_region_avail()[1] - height) / 2.0])
    }

    fn is_popup_open(&self, popup: &ImStr) -> bool {
        unsafe { imgui::sys::igIsPopupOpen(popup.as_ptr()) }
    }

    // TODO: Replace this when ImGui supports proper disabled widgets.
    fn button_hack(&self, label: &ImStr, size: [f32; 2], enabled: bool) -> bool {
        match enabled {
            true => self.button(label, size),
            false => {
                let style = self.push_style_var(StyleVar::Alpha(self.clone_style().alpha * 0.5));
                let colors = self.push_style_colors(&[
                    (StyleColor::ButtonActive, self.style_color(StyleColor::Button)),
                    (StyleColor::ButtonHovered, self.style_color(StyleColor::Button)),
                ]);
                self.button(label, size);
                style.pop(self);
                colors.pop(self);
                false
            }
        }
    }

    fn toggle_button_labeled(&self, id: &ImStr, text_on: &str, text_off: &str, pressed: &mut bool) {
        let label = if *pressed { text_on } else { text_off };
        if self.button(&im_str!("{}###{}", label, id), AUTO_SIZE) {
            *pressed = !*pressed;
        }
    }

    fn small_toggle_button(&self, label: &ImStr, pressed: &mut bool) -> bool {
        let style_color = if *pressed { StyleColor::ButtonActive } else { StyleColor::Button };
        let color = self.push_style_color(StyleColor::Button, self.style_color(style_color));
        let toggle = self.small_button(label);
        if toggle { *pressed = !*pressed; }
        color.pop(self);
        toggle
    }

    fn move_cursor(&self, amount: [f32; 2]) {
        let cursor_pos = self.cursor_pos();
        self.set_cursor_pos([cursor_pos[0] + amount[0], cursor_pos[1] + amount[1]]);
    }

    fn fullscreen_window(&self, id: &ImStr, contents: impl FnOnce()) {
        let wr = self.push_style_var(StyleVar::WindowRounding(0.0));
        Window::new(id)
            .position([0.0, 0.0], Condition::FirstUseEver)
            .size(self.io().display_size, Condition::Always)
            .flags(WindowFlags::NO_TITLE_BAR | WindowFlags::NO_DECORATION | WindowFlags::NO_MOVE | WindowFlags::MENU_BAR)
            .build(self, || {
                let wr = self.push_style_var(StyleVar::WindowRounding(1.0));
                contents();
                wr.pop(self);
            });
        wr.pop(self);
    }

}

pub trait ChildWindowExt: Sized {
    fn border_box(self, ui: &Ui, size: [f32; 2]) -> Self;
}

impl<'a> ChildWindowExt for ChildWindow<'a> {
    fn border_box(self, ui: &Ui, size: [f32; 2]) -> Self {
        let border_size = ui.clone_style().child_border_size;
        self.size([f32::max(0.0, size[0] - border_size), f32::max(0.0, size[1] - border_size)])
    }
}

pub fn choose_folder(desc: &str) -> Result<Option<PathBuf>, ErrorModal> {
    match nfd::open_pick_folder(None) {
        Ok(nfd::Response::Okay(f)) => match f.parse() {
            Ok(path) => Ok(Some(path)),
            Err(e) => return Err(ErrorModal::new(format!("Invalid path to {}.", desc), Some(e))),
        }
        Err(e) => return Err(ErrorModal::new(format!("Could not open {} picker.", desc), Some(e))),
        _ => Ok(None),
    }
}

pub fn fit_size(original: [f32; 2], bounds: [f32; 2]) -> [f32; 2] {
    let scale_factor = f32::min(bounds[0] / original[0], bounds[1] / original[1]);
    [original[0] * scale_factor, original[1] * scale_factor]
}

#[macro_export]
macro_rules! reborrow_frame {
    {$frame:ident} => {
        Frame { ui: $frame.ui, resources: $frame.resources, textures: &mut *$frame.textures }
    }
}
