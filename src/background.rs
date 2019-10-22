use std::path::{Path, PathBuf};
use image::{self, ImageResult, DynamicImage};
use crate::{OptionExt as _, sources::*};
use bitflags::bitflags;

#[derive(Clone)]
pub struct DesktopBackground {
    pub name: String,
    pub location: String,
    pub source: usize,
    pub original: OriginalKey,
    pub size: Option<(u32, u32)>,
    pub center: Option<(u32, u32)>,
    pub comments: Vec<String>,
    pub flags: DesktopBackgroundFlags,
}

impl DesktopBackground {
    pub fn from_original(source: usize, key: OriginalKey, original: &dyn Original) -> DesktopBackground {
        DesktopBackground {
            name: original.location(),
            location: original.location(),
            source: source,
            original: key,
            size: original.size(),
            center: original.size().map(|(x, y)| (x / 2, y / 2)),
            comments: Vec::new(),
            flags: DesktopBackgroundFlags::UNEDITED,
        }
    }

    pub fn update_from(&mut self, _original: &dyn Original) -> DesktopBackground {
        unimplemented!()
    }
}

bitflags! {
    pub struct DesktopBackgroundFlags: u32 {
        const UNEDITED = 0x1;
        const MISSING_ORIGINAL = 0x2;
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
    fn location(&self) -> String;
    fn size(&self) -> Option<(u32, u32)>;
}


