use std::path::{Path, PathBuf};
use image::{self, ImageResult, DynamicImage, GenericImageView};
use crate::{sources::*, utils::{OptionExt as _, MaybeStale}};
use bitflags::bitflags;

#[derive(Clone)]
pub struct DesktopBackground {
    pub name: String,
    pub location: String,
    pub source: usize,
    pub original: OriginalKey,
    pub size: MaybeStale<(u32, u32)>,
    pub center: MaybeStale<(u32, u32)>,
    pub comments: Vec<String>,
    pub flags: DesktopBackgroundFlags,
    pub excluded: bool,
}

impl DesktopBackground {
    /// Create a new DesktopBackground from an Original.
    pub fn from_original(source: usize, key: OriginalKey, original: &dyn Original, excluded: bool) -> DesktopBackground {
        let mut flags = DesktopBackgroundFlags::UNEDITED;
        let size = original.read_image().map(|i| i.dimensions()).ok();
        flags.set(DesktopBackgroundFlags::ORIGINAL_UNAVAILABLE, size.is_none());
        DesktopBackground {
            name: original.name(),
            location: original.location(), // TODO: Figure out how this should work
            source: source,
            original: key,
            size: size.into(),
            center: size.map(|(x, y)| (x / 2, y / 2)).into(),
            comments: Vec::new(),
            flags: DesktopBackgroundFlags::UNEDITED,
            excluded: excluded,
        }
    }

    /// Update this background when changes have been made to its original. 
    pub fn update_from(&mut self, key: OriginalKey, original: &dyn Original) {
        assert!(key.compare(&self.original) != KeyRelation::Distinct);
        self.name = original.name();
        self.location = original.location();
        self.original = key;
        let image = self.try_read_image_from(original);
        let size = image.map(|i| i.dimensions()).ok();
        if size.as_ref() != self.size.value() { self.center.update(size.map(|(x, y)| (x / 2, y / 2))); }
        self.size.update(size);
        self.flags.insert(DesktopBackgroundFlags::UNEDITED);
    }

    /// Helper function to try reading this background's original. It is a logic error to call this with
    /// a different original than the one actually associated with the background.
    pub fn try_read_image_from(&mut self, original: &dyn Original) -> ImageResult<DynamicImage> {
        let image = original.read_image();
        self.flags.set(DesktopBackgroundFlags::ORIGINAL_UNAVAILABLE, image.is_err());
        image
    }
}

bitflags! {
    pub struct DesktopBackgroundFlags: u32 {
        /// This background has not been edited since its original last changed.
        const UNEDITED = 0x1;
        /// This background's original has been deleted or altered.
        const ORIGINAL_MISSING = 0x2;
        /// This background's original is temporarily or permanently unavailable.
        const ORIGINAL_UNAVAILABLE = 0x4;
    }
}

pub struct BackgroundSet {
    image_folder: Option<PathBuf>,
    pub(crate) sources: Vec<Box<dyn ErasedDesktopBackgroundSource>>,
    name: Option<String>,
    pub(crate) backgrounds: Vec<DesktopBackground>,
}

impl BackgroundSet {
    pub fn new() -> BackgroundSet {
        BackgroundSet {
            image_folder: None,
            sources: Vec::new(),
            name: None,
            backgrounds: Vec::new(),
        }
    }

    pub fn image_folder(&self) -> Option<&Path> {
        self.image_folder.deref()
    }

    pub fn set_image_folder(&mut self, path: impl AsRef<Path>) {
        self.image_folder = Some(path.as_ref().to_owned());
    }

    pub fn name(&self) -> Option<&str> {
        self.name.deref()
    }

    pub fn set_name(&mut self, name: impl AsRef<str>) {
        self.name = Some(name.as_ref().to_owned());
    }

    pub fn sources(&self) -> &Vec<Box<dyn ErasedDesktopBackgroundSource>> {
        &self.sources
    }

    pub fn sources_mut(&mut self) -> &mut Vec<Box<dyn ErasedDesktopBackgroundSource>> {
        &mut self.sources
    }

    pub fn add_source<S: for<'a> DesktopBackgroundSource<'a> + 'static>(&mut self, source: S) -> usize {
        self.sources.push(Box::new(source));
        self.sources.len() - 1
    }

    pub fn backgrounds(&self) -> &Vec<DesktopBackground> {
        &self.backgrounds
    }

    pub fn backgrounds_mut(&mut self) -> &mut Vec<DesktopBackground> {
        &mut self.backgrounds
    }
}

pub trait Original {
    fn read_image(&self) -> ImageResult<DynamicImage>;
    fn name(&self) -> String;
    fn location(&self) -> String;
}


