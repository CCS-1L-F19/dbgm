use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::BufReader;

use stable_vec::StableVec;

use crate::sources::{DesktopBackgroundSource, ErasedDesktopBackgroundSource};
use crate::background::DesktopBackground;
use crate::utils::OptionExt as _;

#[derive(Debug)]
pub enum SetLoadError {
    Io(std::io::Error),
}

impl From<std::io::Error> for SetLoadError {
    fn from(error: std::io::Error) -> SetLoadError {
        SetLoadError::Io(error)
    }
}

pub struct BackgroundSet {
    image_folder: Option<PathBuf>,
    name: Option<String>,
    pub(crate) resolution: (usize, usize),
    pub(crate) backgrounds: StableVec<DesktopBackground>,
    pub(crate) sources: StableVec<Box<dyn ErasedDesktopBackgroundSource>>,
}

impl BackgroundSet {
    pub fn new(resolution: (usize, usize)) -> BackgroundSet {
        BackgroundSet {
            image_folder: None,
            name: None,
            resolution: resolution,
            backgrounds: StableVec::new(),
            sources: StableVec::new(),
        }
    }

    pub fn load(path: impl AsRef<Path>) -> Result<BackgroundSet, SetLoadError> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        unimplemented!()
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