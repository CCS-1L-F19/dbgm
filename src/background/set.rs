use std::path::{Path, PathBuf};

use stable_vec::StableVec;

use crate::sources::{DesktopBackgroundSource, ErasedDesktopBackgroundSource};
use crate::background::DesktopBackground;
use crate::utils::OptionExt as _;

pub struct BackgroundSet {
    image_folder: Option<PathBuf>,
    name: Option<String>,
    pub(crate) backgrounds: StableVec<DesktopBackground>,
    pub(crate) sources: StableVec<Box<dyn ErasedDesktopBackgroundSource>>,
}

impl BackgroundSet {
    pub fn new() -> BackgroundSet {
        BackgroundSet {
            image_folder: None,
            name: None,
            backgrounds: StableVec::new(),
            sources: StableVec::new(),
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

    pub fn add_source<S: for<'a> DesktopBackgroundSource<'a> + 'static>(&mut self, source: S) -> usize {
        self.sources.push(Box::new(source))
    }

    pub fn remove_source(&mut self, source: usize) {
        self.backgrounds.retain(|b| b.source != source);
        self.sources.remove(source);
    }
}