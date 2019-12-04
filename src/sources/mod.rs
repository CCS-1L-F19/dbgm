use std::hash::Hash;
use std::fmt::Debug;

use crate::background::Original;

mod folder;
mod erased;

pub use erased::{OriginalKey, ErasedDesktopBackgroundSource, load_source_by_id, SourceLoadError, SourceLoader};
pub use folder::FolderSource;

pub trait DesktopBackgroundSource<'a>: erased_serde::Serialize {
    type Key: Hash + Clone + serde::Serialize + serde::de::DeserializeOwned + CompareKey;
    type Original: Original;
    type Error: Debug + 'a;

    const TYPE_IDENT: &'static str;

    fn name(&self) -> &str;
    fn original(&self, key: &Self::Key) -> OriginalResult<&Self::Original>;
    fn reload(&mut self) -> Vec<OriginalChange<Self::Key, Self::Error>>;
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum KeyRelation { SameOriginal, ContentMismatch, Distinct }

pub trait CompareKey {
    fn compare(&self, other: &Self) -> KeyRelation;
}

pub struct OriginalChange<K = OriginalKey, E = Box<dyn Debug>> {
    pub key: K,
    pub kind: ChangeKind<E>,
}

pub enum OriginalResult<O> {
    Original(O),
    ContentMismatch(O),
    WrongSource,
    NotFound,
}

impl<O> OriginalResult<O> {
    fn map<T>(self, f: impl FnOnce(O) -> T) -> OriginalResult<T> {
        use OriginalResult::*;
        match self {
            OriginalResult::Original(o) => Original(f(o)),
            OriginalResult::ContentMismatch(o) => ContentMismatch(f(o)),
            OriginalResult::WrongSource => WrongSource,
            OriginalResult::NotFound => NotFound,
        }
    }

    pub fn as_option(self) -> Option<O> {
        match self {
            OriginalResult::Original(o) => Some(o),
            OriginalResult::ContentMismatch(o) => Some(o),
            OriginalResult::WrongSource => None,
            OriginalResult::NotFound => None,
        }
    }
}

#[derive(Debug)]
pub enum ChangeKind<E> {
    /// A new original has been discovered.
    New,
    /// An existing original has been deleted.
    Deleted,
    /// An existing original has been altered.
    Altered,
    // An existing original has become unavailable (perhaps temporarily).
    Unavailable(E)
}