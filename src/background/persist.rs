use std::path::PathBuf;
use std::collections::HashMap;

use serde::*;
use stable_vec::StableVec;

use crate::sources::{self, ErasedDesktopBackgroundSource};
use crate::background::*;

#[derive(Serialize, Deserialize)]
pub struct SavedBackgroundSet {
    image_folder: PathBuf,
    name: String,
    resolution: (usize, usize),
    sources: HashMap<usize, (String, serde_json::Value)>,
    backgrounds: Vec<SavedDesktopBackground>,
}

#[derive(Debug)]
enum SetLoadWarning {
    CorruptSource { source: (String, serde_json::Value), error: sources::SourceLoadError },
    SourceDoesNotExist { background: SavedDesktopBackground },
    // CorruptOriginalKey { background: SavedDesktopBackground }, 
}


impl SavedBackgroundSet {
    fn load(self) -> (BackgroundSet, Vec<SetLoadWarning>) {
        let mut warnings = Vec::new();
        let sources = self.sources.into_iter().filter_map(|(k, (id, data))| {
            match sources::load_source_by_id(&id, data.clone()) {
                Ok(source) => Some((k, source)),
                Err(e) => {
                    warnings.push(SetLoadWarning::CorruptSource { source: (id, data), error: e}); 
                    None 
                }
            }
        }).collect::<HashMap<usize, Box<dyn ErasedDesktopBackgroundSource>>>();
        let backgrounds = self.backgrounds.into_iter().filter_map(|b| {
            Some(DesktopBackground {
                original: match sources.get(&b.source) {
                    Some(source) => source.assemble_key(b.key_data.clone()),
                    None => {
                        warnings.push(SetLoadWarning::SourceDoesNotExist { background: b });
                        return None
                    }
                },
                name: b.name,
                location: b.location,
                comments: b.comments,
                source: b.source,
                flags: b.flags,
                original_meta: OriginalMeta::Stale { last_known_size: b.original_meta.last_known_size },
                edit_info: b.edit_info,
            })
        }).collect::<StableVec<DesktopBackground>>();
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
    source: usize,
    key_data: serde_json::Value,
    flags: DesktopBackgroundFlags,
    original_meta: SavedOriginalMeta,
    edit_info: Option<EditInfo>,
}