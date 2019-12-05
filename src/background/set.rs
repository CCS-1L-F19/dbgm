use std::path::{Path, PathBuf};

use stable_vec::StableVec;

use crate::sources::{DesktopBackgroundSource, ErasedDesktopBackgroundSource};
use crate::background::{DesktopBackground, DesktopBackgroundFlags};
use crate::utils::OptionExt as _;

pub struct BackgroundSet {
    pub(super) image_folder: Option<PathBuf>,
    pub(super) name: Option<String>,
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

    /// Rebuilds the image folder from scratch. Returns a list of background IDs that were *not* included.
    pub fn rebuild_image_folder(&mut self) -> Result<Vec<(usize, SkipReason)>, std::io::Error> {
        use std::fs;
        use blake2::{Blake2b, digest::Digest};
        use image::ImageFormat;
        
        let image_folder = self.image_folder.as_ref().expect("Cannot update image folder when none is set!");
        
        // Ensure the image folder exists.
        fs::create_dir_all(image_folder)?;

        // Clear contents of the image folder. If this fails, we abort.
        for entry in image_folder.read_dir()? {
            let entry = entry?;
            if entry.metadata()?.is_file() {
                fs::remove_file(entry.path())?; 
            }
        }

        // Save a file in the folder for each background whose original is accessible.
        let mut skipped = Vec::new();
        for (id, background) in self.backgrounds.iter_mut().filter(|(_, b)| !b.flags.contains(DesktopBackgroundFlags::EXCLUDED)) { 
            if background.flags.contains(DesktopBackgroundFlags::EXCLUDED) {
                skipped.push((id, SkipReason::Excluded));
                continue
            }
            
            let original = match self.sources[background.source].original(&background.original).as_option() {
                Some(original) => original,
                None => { skipped.push((id, SkipReason::OriginalUnavailable)); continue }
            };

            let mut image = match background.try_read_image_from(original) {
                Ok(image) => image,
                Err(e) => { skipped.push((id, SkipReason::CorruptImage(e))); continue }
            };

            let resolution = vec2![self.resolution.0 as f32, self.resolution.1 as f32];
            let crop_region = match background.crop_region(resolution) {
                Ok(crop_region) => crop_region,
                Err(_) => { skipped.push((id, SkipReason::OriginalUnavailable)); continue }
            };

            let cropped = crop_region.crop(&mut image).to_image();
            let mut hasher = Blake2b::new();
            hasher.input(&*cropped);
            let path = image_folder.join(base64::encode_config(&hasher.result(), base64::URL_SAFE) + ".png");
            cropped.save_with_format(path, ImageFormat::PNG)?;
        }
        Ok(skipped)
    }
}

#[derive(Debug)]
pub enum SkipReason {
    OriginalUnavailable,
    CorruptImage(image::ImageError),
    Excluded,
}