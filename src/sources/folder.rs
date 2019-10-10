use std::path::PathBuf;
use std::io::ErrorKind;
use std::fs::{self, File};
use image::{ImageResult, DynamicImage};

use super::*;

pub struct FolderSource {
    folder: PathBuf,
    name: String,
    originals: Vec<FolderSourceOriginal>,
}

impl FolderSource {
    pub fn new(folder: PathBuf, name: &str) -> Self {
        FolderSource {
            folder: folder,
            name: name.to_owned(),
            originals: Vec::new(),
        }
    }
}

impl<'a> DesktopBackgroundSource<'a> for FolderSource {
    type Key = FolderKey;
    type Error = std::io::Error;
    type Original = FolderSourceOriginal;
    // type KeyIter = !;
    fn name(&self) -> &str { &self.name }
    // fn keys(&'a self) -> Self::KeyIter { unimplemented!() }
    fn original(&self, key: &Self::Key) -> OriginalResult<&Self::Original> {
        match self.originals.get(key.original_id) {
            Some(original) => match !original.mismatch && original.hash == key.hash {
                true => OriginalResult::Original(original),
                false => OriginalResult::ContentMismatch(original),
            },
            None => OriginalResult::NotFound // TODO: Distinguish between this and WrongSource?
        }
    }

    fn reload(&mut self) -> Vec<OriginalChange<FolderKey, std::io::Error>> {
        Vec::new()
    }
}

#[derive(Hash, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct FolderKey {
    original_id: usize,
    hash: [u8; 32],
}

pub struct FolderSourceOriginal {
    mismatch: bool,
    path: PathBuf,
    hash: [u8; 32],
}

impl Original for FolderSourceOriginal {
    fn read_image(&self) -> ImageResult<DynamicImage> {
        image::open(&self.path)
    }

    fn location(&self) -> String {
        self.path.to_string_lossy().to_owned().to_string()
    }
}