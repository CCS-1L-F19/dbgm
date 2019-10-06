use std::hash::{Hash, Hasher};
use crate::Original;

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
    type KeyIter: Iterator<Item=Self::Key> + 'a;
    fn name(&self) -> String;
    fn keys(&'a self) -> Self::KeyIter;
    fn original(&self, key: &Self::Key) -> OriginalResult<&Self::Original>;
    fn reload(&mut self);
}

pub trait ErasedDesktopBackgroundSource {
    fn name(&self) -> String;
    fn keys<'a>(&'a self) -> Box<dyn Iterator<Item=OriginalKey> + 'a>;
    fn original(&self, id: &OriginalKey) -> OriginalResult<&dyn Original>;
    fn reload(&mut self);
}

#[derive(Clone)]
pub struct OriginalKey {
    value: serde_json::Value,
    comparer: Box<fn(&OriginalKey, &OriginalKey) -> bool>,
    hasher: Box<fn(&OriginalKey, &mut dyn Hasher)>,
}

impl PartialEq for OriginalKey { fn eq(&self, other: &Self) -> bool { (self.comparer)(self, other) } }
impl Hash for OriginalKey { fn hash<H: Hasher>(&self, hasher: &mut H) { (self.hasher)(self, hasher) } }
impl Eq for OriginalKey { }

mod erase {
    use super::*;
    use std::hash::{Hash, Hasher};

    impl OriginalKey {
        pub(in super) fn new<'a, S: DesktopBackgroundSource<'a>>(key: S::Key) -> OriginalKey {
            OriginalKey {
                value: serde_json::to_value(key).expect("Could not serialize original key to JSON!"),
                comparer: Box::new(erase::key_comparer::<S>),
                hasher: Box::new(erase::key_hasher::<S>)
            }
        }

        pub(in super) fn try_deserialize<K: serde::de::DeserializeOwned>(&self) -> Option<K> {
            serde_json::from_value(self.value.clone()).ok()
        }
    }

    fn key_comparer<'a, S: DesktopBackgroundSource<'a>>(k1: &OriginalKey, k2: &OriginalKey) -> bool {
        match (serde_json::from_value::<S::Key>(k1.value.clone()), serde_json::from_value::<S::Key>(k2.value.clone())) {
            (Ok(k1), Ok(k2)) => k1 == k2,
            _ => false,
        }
    }

    struct HashWrapper<'a>(&'a mut dyn Hasher);
    impl<'a> Hasher for HashWrapper<'a> {
        fn write(&mut self, bytes: &[u8]) { self.0.write(bytes); }
        fn finish(&self) -> u64 { self.0.finish() }
    }

    fn key_hasher<'a, S: DesktopBackgroundSource<'a>>(key: &OriginalKey, hasher: &mut dyn Hasher) {
        let key = serde_json::from_value::<S::Key>(key.value.clone()).expect("Corrupt OriginalKey detected!");
        key.hash(&mut HashWrapper(hasher));
    }
}

impl<S: for<'a> DesktopBackgroundSource<'a>> ErasedDesktopBackgroundSource for S {
    fn name(&self) -> String { self.name() }

    fn keys<'a>(&'a self) -> Box<dyn Iterator<Item=OriginalKey> + 'a> {
        Box::new(self.keys().map(OriginalKey::new::<S>))
    }

    fn original(&self, key: &OriginalKey) -> OriginalResult<&dyn Original> {
        key.try_deserialize()
            .map(|k| self.original(&k).map(|o| o as &dyn Original))
            .unwrap_or(OriginalResult::WrongSource)
    }

    fn reload(&mut self) { self.reload() }
}