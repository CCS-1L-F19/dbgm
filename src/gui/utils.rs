use std::collections::HashMap;
use std::hash::Hash;
use std::path::PathBuf;

use imgui::*;

use super::modals::Modal;

pub const AUTO_SIZE: [f32; 2] = [0.0, 0.0];

pub trait Textures {
    type CreationError: std::fmt::Debug;
    fn create_texture(&mut self, image: &image::DynamicImage) -> Result<TextureId, Self::CreationError>;
}

pub struct ImageCache<K: Hash + Eq> {
    images: HashMap<K, (image::DynamicImage, Option<TextureId>)>,
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

    pub fn remove_image(&mut self, key: &K) -> Option<(image::DynamicImage, Option<TextureId>)> {
        self.images.remove(key)
    }

    pub fn get_image(&self, key: &K) -> Option<&image::DynamicImage> {
        self.images.get(key).map(|(i, _)| i)
    }

    pub fn load_texture<T: Textures + ?Sized>(&mut self, key: &K, textures: &mut T) -> Option<Result<TextureId, T::CreationError>> {
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
    fn pad_to_center(&self, width: f32);
    fn is_popup_open(&self, popup: &ImStr) -> bool;
    fn button_hack(&self, label: &ImStr, size: [f32; 2], enabled: bool) -> bool;
}

impl<'ui> UiExt for Ui<'ui> {
    fn pad_to_center(&self, width: f32) {
        let cpos = self.cursor_pos();
        self.set_cursor_pos([cpos[0] + (self.content_region_max()[0] - width) / 2.0, cpos[1]]);
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
}

pub fn choose_folder(desc: &str) -> Result<Option<PathBuf>, Modal> {
    match nfd::open_pick_folder(None) {
        Ok(nfd::Response::Okay(f)) => match f.parse() {
            Ok(path) => Ok(Some(path)),
            Err(e) => return Err(Modal::error(format!("Invalid path to {}.", desc), Some(e))),
        }
        Err(e) => return Err(Modal::error(format!("Could not open {} picker.", desc), Some(e))),
        _ => Ok(None),
    }
}