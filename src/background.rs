use std::path::{Path, PathBuf};
use image::{self, ImageResult, DynamicImage};
use crate::{OptionExt as _, sources::*};

#[derive(Clone)]
pub struct DesktopBackground {
    pub name: String,
    pub location: PathBuf,
    pub source: usize,
    pub original: OriginalKey,
    pub size: (usize, usize),
    pub center: (usize, usize),
    pub comments: Vec<String>,
}

pub struct BackgroundSet {
    image_folder: Option<PathBuf>,
    sources: Vec<Box<dyn ErasedDesktopBackgroundSource>>,
    name: Option<String>,
    backgrounds: Vec<DesktopBackground>,
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
        return self.sources.len()
    }

    pub fn backgrounds(&self) -> impl Iterator<Item=&DesktopBackground> {
        self.backgrounds.iter()
    }

    pub fn backgrounds_mut(&mut self) -> impl Iterator<Item=&mut DesktopBackground> {
        self.backgrounds.iter_mut()
    }
}

pub trait Original {
    fn read_image(&self) -> ImageResult<DynamicImage>;
    fn location(&self) -> String;
}


