use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::{BufReader, BufWriter};

use serde::*;
use stable_vec::StableVec;

use crate::sources;
use crate::background::*;

impl BackgroundSet {
    pub fn load(path: impl AsRef<Path>) -> Result<(BackgroundSet, Vec<SetLoadWarning>), Error> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let saved_data: SavedBackgroundSet = serde_json::from_reader(reader)?;
        Ok(saved_data.load())
    }

    pub fn save(&self, path: impl AsRef<Path>) -> Result<(), Error> {
        let file = File::create(path)?;
        let writer = BufWriter::new(file);
        let saved_data = SavedBackgroundSet {
            // These are expect calls rather than Err returns because there is no reason to gracefully 
            // handle these errors here when the UI code must check the preconditions itself anyway.
            image_folder: self.image_folder.clone().expect("Cannot save background set without image folder!"),
            name: self.name.clone().expect("Cannot save background set without a name!"),
            resolution: self.resolution,
            sources: self.sources.iter().map(|(id, source)| SavedBackgroundSource {
                ty: source.source_type_id().to_owned(),
                data: serde_json::to_value(source.as_serialize()).expect("Serializing a source should never fail!"),
                backgrounds: self.backgrounds.values().filter(|b| b.source == id).map(|b| SavedDesktopBackground {
                    name: b.name.clone(),
                    location: b.location.clone(),
                    comments: b.comments.clone(),
                    key_data: (&b.original).into(),
                    flags: b.flags.clone(),
                    original_meta: SavedOriginalMeta { last_known_size: b.original_meta.last_known_size() },
                    edit_info: b.edit_info.clone(),
                }).collect()
            }).collect()
        };
        serde_json::to_writer(writer, &saved_data).map_err(Into::into)
    }
}

#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    Json(serde_json::Error),
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Error {
        Error::Io(error)
    }
}

impl From<serde_json::Error> for Error {
    fn from(error: serde_json::Error) -> Error {
        Error::Json(error)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SavedBackgroundSource {
    ty: String,
    data: serde_json::Value,
    backgrounds: Vec<SavedDesktopBackground>,
}

#[derive(Serialize, Deserialize)]
pub struct SavedBackgroundSet {
    image_folder: PathBuf,
    name: String,
    resolution: (usize, usize),
    sources: Vec<SavedBackgroundSource>,
}

#[derive(Debug)]
pub enum SetLoadWarning {
    CorruptSource { source: SavedBackgroundSource, error: sources::SourceLoadError },
    // CorruptOriginalKey { background: SavedDesktopBackground }, 
}

impl SavedBackgroundSet {
    fn load(self) -> (BackgroundSet, Vec<SetLoadWarning>) {
        let mut warnings = Vec::new();
        let mut sources = StableVec::new();
        let mut backgrounds = StableVec::new();
        for saved_source in self.sources {
            match sources::load_source_by_id(&saved_source.ty, saved_source.data.clone()) {
                Ok(source) => {
                    backgrounds.extend(saved_source.backgrounds.into_iter().map(|b| DesktopBackground {
                        original: source.assemble_key(b.key_data),
                        name: b.name,
                        location: b.location,
                        comments: b.comments,
                        source: sources.num_elements(),
                        flags: b.flags,
                        original_meta: OriginalMeta::Stale { last_known_size: b.original_meta.last_known_size },
                        edit_info: b.edit_info,
                    }));
                    sources.push(source);
                }
                Err(error) => warnings.push(SetLoadWarning::CorruptSource {
                    source: saved_source, error
                })
            }
        }
        (BackgroundSet {
            image_folder: Some(self.image_folder),
            name: Some(self.name),
            resolution: self.resolution,
            backgrounds,
            sources,
        }, warnings)
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct SavedOriginalMeta { last_known_size: Option<(u32, u32)> }

#[derive(Debug, Serialize, Deserialize)]
pub struct SavedDesktopBackground {
    name: String,
    location: String,
    comments: String,
    key_data: serde_json::Value,
    flags: DesktopBackgroundFlags,
    original_meta: SavedOriginalMeta,
    edit_info: Option<EditInfo>,
}