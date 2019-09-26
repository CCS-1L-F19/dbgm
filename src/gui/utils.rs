use std::hash::Hash;
use std::collections::HashMap;
use imgui::*;

pub trait Textures {
    type CreationError: std::fmt::Debug;
    fn create_texture(&mut self, image: &image::DynamicImage) -> Result<ImTexture, Self::CreationError>;
}

pub struct ImageCache<K: Hash + Eq> {
    images: HashMap<K, (image::DynamicImage, Option<ImTexture>)>,
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

    pub fn remove_image(&mut self, key: &K) -> Option<(image::DynamicImage, Option<ImTexture>)> {
        self.images.remove(key)
    }

    pub fn get_image(&self, key: &K) -> Option<&image::DynamicImage> {
        self.images.get(key).map(|(i, _)| i)
    }

    pub fn load_texture<T: Textures + ?Sized>(&mut self, key: &K, textures: &mut T) -> Option<Result<ImTexture, T::CreationError>> {
        match self.images.get_mut(key) {
            Some((image, Some(texture))) => Some(Ok(*texture)),
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
    // fn list_box_header(&self, label: &ImStr, count: i32, height_in_items: i32) -> bool;
    // fn list_box_footer(&self);
}

impl<'ui> UiExt for Ui<'ui> {
    fn pad_to_center(&self, width: f32) {
        let cpos = self.get_cursor_pos();
        self.set_cursor_pos((cpos.0 + (self.get_content_region_max().0 - width) / 2.0, cpos.1));
    }

    fn is_popup_open(&self, popup: &ImStr) -> bool {
        unsafe { imgui::sys::igIsPopupOpen(popup.as_ptr()) }
    }
}

pub struct ListClipper(sys::ImGuiListClipper);

impl ListClipper {
    pub fn new(count: u32, item_height: Option<f32>) -> ListClipper {
        let mut inner = sys::ImGuiListClipper { // Dummy values, will be overwritten in next call.
            start_pos_y: 0.0,
            items_height: 0.0,
            items_count: 0,
            step_no: 0,
            display_start: 0,
            display_end: 0,
        };
        unsafe { 
            sys::ImGuiListClipper_Begin(
                &mut inner as *mut _, 
                if count > std::i32::MAX as u32 { std::i32::MAX } else { count as i32 },
                item_height.unwrap_or(-1.0),
            ) 
        }
        ListClipper(inner)
    }

    pub fn step(&mut self) -> bool {
        unsafe { sys::ImGuiListClipper_Step(&mut self.0 as *mut _) }
    }

    pub fn display_items(&self) -> impl Iterator<Item=usize> {
        self.0.display_start as usize .. self.0.display_end as usize
    }
}

macro_rules! load_internal_texture {
    {$textures:ident, $path:tt} => { {
        let image = image::load_from_memory(include_bytes!($path)).expect("Failed to load internal texture!");
        $textures.create_texture(&image).expect("Failed to create internal texture!")
    } }
}