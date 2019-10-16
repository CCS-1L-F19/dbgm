use std::hash::Hash;
use std::fmt::Debug;

use crate::background::Original;

mod folder;
mod erased;

pub use erased::{OriginalKey, ErasedDesktopBackgroundSource};
pub use folder::FolderSource;

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
            Original(o) => Original(f(o)),
            ContentMismatch(o) => ContentMismatch(f(o)),
            WrongSource => WrongSource,
            NotFound => NotFound,
        }
    }
}

pub trait DesktopBackgroundSource<'a> {
    type Key: serde::Serialize + serde::de::DeserializeOwned + Eq + Hash;
    type Original: Original;
    type Error: Debug + 'a;
    // type KeyIter: Iterator<Item=Self::Key> + 'a;
    fn name(&self) -> &str;
    // fn keys(&'a self) -> Self::KeyIter;
    fn original(&self, key: &Self::Key) -> OriginalResult<&Self::Original>;
    fn reload(&mut self) -> Vec<OriginalChange<Self::Key, Self::Error>>;
}

pub struct OriginalChange<K = OriginalKey, E = Box<dyn Debug>> {
    pub key: K,
    pub kind: ChangeKind<E>,
}

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