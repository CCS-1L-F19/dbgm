use std::{
    collections::HashMap,
    path::PathBuf,
    ffi::OsString,
    io::{self, Read, ErrorKind},
    fs::{self, File},
};

use image::{ImageResult, DynamicImage};
use serde::{Serialize, Deserialize};

use super::*;

pub const HASH_SIZE: usize = 32;

#[derive(Serialize, Deserialize)]
pub struct FolderSource {
    folder: PathBuf,
    name: String,
    #[serde(with = "crate::utils::as_pairs")] // OsStrings can't be used as JSON keys, so we save a list of pairs
    originals: HashMap<OsString, OriginalFile>,
}

impl FolderSource {
    pub fn new(folder: PathBuf, name: &str) -> Self {
        FolderSource {
            folder: folder,
            name: name.to_owned(),
            originals: HashMap::new(),
        }
    }

    fn hash_file(mut file: File) -> io::Result<[u8; HASH_SIZE]> {
        use blake2::{*, digest::*};
        let mut hasher = VarBlake2b::new(HASH_SIZE).unwrap();
        let mut buf = Vec::new();
        file.read_to_end(&mut buf)?;
        hasher.input(&buf);
        let mut hash = [0; HASH_SIZE];
        hasher.variable_result(|h| hash.copy_from_slice(h));
        Ok(hash)
    }
}

impl<'a> DesktopBackgroundSource<'a> for FolderSource {
    type Key = FileKey;
    type Error = std::io::Error;
    type Original = OriginalFile;

    const TYPE_IDENT: &'static str = "folder";

    fn name(&self) -> &str { &self.name }

    fn original(&self, key: &Self::Key) -> OriginalResult<&Self::Original> {
        match self.originals.get(&key.filename) {
            Some(original) => match !original.mismatch && original.hash == key.hash {
                true => OriginalResult::Original(original),
                false => OriginalResult::ContentMismatch(original),
            },
            None => OriginalResult::NotFound // TODO: Distinguish between this and WrongSource?
        }
    }

    fn reload(&mut self) -> Vec<OriginalChange<FileKey, std::io::Error>> {
        let mut contents: HashMap<_, _> = fs::read_dir(&self.folder).map(|dir| {
            dir.filter_map(|r| r.ok())
                .filter(|e| e.metadata().ok().map(|m| m.is_file()).unwrap_or(false))
                .map(|e| (e.file_name(), e))
                .collect()
        }).unwrap_or_else(|_| HashMap::new());
        
        let mut to_remove = Vec::new();

        // Check all the originals we already have for changes.
        let mut changes = self.originals.iter_mut().filter_map(|(filename, original)| {
            contents.remove(filename);
            let key = FileKey { filename: filename.clone(), hash: original.hash };
            // TODO: maybe avoid computing the hash if there's a timestamp mismatch?
            match File::open(&original.path).and_then(FolderSource::hash_file) {
                Ok(hash) if hash != original.hash => {
                    original.hash = hash; // TODO: Figure out how to deal with the two hashes
                    Some(OriginalChange { key: key, kind: ChangeKind::Altered })
                },
                Err(ref e) if e.kind() == ErrorKind::NotFound => {
                    to_remove.push(filename.clone());
                    Some(OriginalChange { key: key, kind: ChangeKind::Deleted })
                },
                Err(e) => {
                    Some(OriginalChange { key: key, kind: ChangeKind::Unavailable(e) })
                },
                _ => None,
            }
        }).collect::<Vec<_>>();

        self.originals.retain(|k, _| !to_remove.contains(k));
        
        // We've removed all existing originals, anything left in contents is new.
        for (_, entry) in contents {
            // TODO: We could go purely by extension here, and say that other files are corrupted
            // instead of silently ignoring them. Alternatively, logging for people who care.
            if image::open(entry.path()).is_ok() {
                if let Ok(hash) = File::open(entry.path()).and_then(FolderSource::hash_file) {
                    let filename = entry.file_name();
                    self.originals.insert(filename.clone(), OriginalFile {
                        mismatch: false,
                        path: entry.path(),
                        hash: hash,
                    });
                    changes.push(OriginalChange { 
                        key: FileKey { filename, hash }, kind: ChangeKind::New
                    });
                }
            }
        }

        changes
    }
}

register_source_type!(FolderSource);

#[derive(Hash, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct FileKey {
    filename: OsString,
    hash: [u8; HASH_SIZE],
}

impl CompareKey for FileKey {
    fn compare(&self, other: &Self) -> KeyRelation {
        match (self.filename == other.filename, self.hash == other.hash) {
            (false, _) => KeyRelation::Distinct,
            (true, false) => KeyRelation::ContentMismatch,
            (true, true) => KeyRelation::SameOriginal,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct OriginalFile {
    mismatch: bool, // TODO: Remove this?
    path: PathBuf,
    hash: [u8; HASH_SIZE],
}

impl Original for OriginalFile {
    fn read_image(&self) -> ImageResult<DynamicImage> {
        image::open(&self.path)
    }

    fn name(&self) -> String {
        self.path.file_name().unwrap().to_string_lossy().to_string()
    }

    fn location(&self) -> String {
        self.path.to_string_lossy().to_owned().to_string()
    }
}