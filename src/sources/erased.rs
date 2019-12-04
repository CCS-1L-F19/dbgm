use super::*;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};
use serde::Serialize;

pub trait ErasedDesktopBackgroundSource: erased_serde::Serialize {
    fn name(&self) -> &str;
    fn original(&self, id: &OriginalKey) -> OriginalResult<&dyn Original>;
    fn reload(&mut self) -> Vec<OriginalChange<OriginalKey, Box<dyn Debug>>>;
    fn assemble_key(&self, value: serde_json::Value) -> OriginalKey;
    fn source_type_id(&self) -> &'static str;
    fn as_serialize(&self) -> &dyn erased_serde::Serialize;
}

#[derive(Clone)]
struct KeyVtable {
    comparer: Box<fn(&OriginalKey, &OriginalKey) -> KeyRelation>,
    hasher: Box<fn(&OriginalKey, &mut dyn Hasher)>,
}

impl KeyVtable {
    fn of<'a, S: DesktopBackgroundSource<'a>>() -> KeyVtable {
        KeyVtable {
            comparer: Box::new(KeyVtable::key_comparer::<S>),
            hasher: Box::new(KeyVtable::key_hasher::<S>),
        }
    }

    fn key_comparer<'a, S: DesktopBackgroundSource<'a>>(k1: &OriginalKey, k2: &OriginalKey) -> KeyRelation {
        match (serde_json::from_value::<S::Key>(k1.value.clone()), serde_json::from_value::<S::Key>(k2.value.clone())) {
            (Ok(k1), Ok(k2)) => k1.compare(&k2),
            _ => KeyRelation::Distinct,
        }
    }

    fn key_hasher<'a, S: DesktopBackgroundSource<'a>>(key: &OriginalKey, hasher: &mut dyn Hasher) {
        struct HashWrapper<'a>(&'a mut dyn Hasher);
        impl<'a> Hasher for HashWrapper<'a> {
            fn write(&mut self, bytes: &[u8]) { self.0.write(bytes); }
            fn finish(&self) -> u64 { self.0.finish() }
        }
        
        let key = serde_json::from_value::<S::Key>(key.value.clone()).expect("Corrupt OriginalKey detected!");
        key.hash(&mut HashWrapper(hasher));
    }
}

#[derive(Clone)]
pub struct OriginalKey {
    value: serde_json::Value,
    vtable: KeyVtable,
}

impl<'a> Into<serde_json::Value> for &'a OriginalKey {
    fn into(self) -> serde_json::Value { self.value.clone() }
}

impl CompareKey for OriginalKey { 
    fn compare(&self, other: &Self) -> KeyRelation { (self.vtable.comparer)(self, other) } 
}

impl Hash for OriginalKey { 
    fn hash<H: Hasher>(&self, hasher: &mut H) { (self.vtable.hasher)(self, hasher) } 
}

impl PartialEq for OriginalKey {
    fn eq(&self, other: &Self) -> bool { self.compare(other) == KeyRelation::SameOriginal }
}

impl Eq for OriginalKey { }

impl OriginalKey {
    fn new<'a, S: DesktopBackgroundSource<'a>>(key: S::Key) -> OriginalKey {
        OriginalKey {
            value: serde_json::to_value(key).expect("Could not serialize original key to JSON!"),
            vtable: KeyVtable::of::<S>(),
        }
    }

    fn try_deserialize<K: serde::de::DeserializeOwned>(&self) -> Option<K> {
        serde_json::from_value(self.value.clone()).ok()
    }
}

impl<S: for<'a> DesktopBackgroundSource<'a>> ErasedDesktopBackgroundSource for S {
    fn name(&self) -> &str { self.name() }

    fn original(&self, key: &OriginalKey) -> OriginalResult<&dyn Original> {
        key.try_deserialize()
            .map(|k| self.original(&k).map(|o| o as &dyn Original))
            .unwrap_or(OriginalResult::WrongSource)
    }

    fn reload(&mut self) -> Vec<OriginalChange<OriginalKey, Box<dyn Debug>>> {
        self.reload().into_iter().map(|c| OriginalChange {
            key: OriginalKey::new::<S>(c.key),
            kind: match c.kind {
                ChangeKind::New => ChangeKind::New,
                ChangeKind::Deleted => ChangeKind::Deleted,
                ChangeKind::Altered => ChangeKind::Altered,
                ChangeKind::Unavailable(e) => ChangeKind::Unavailable(Box::new(e) as Box<dyn Debug>),
            }
        }).collect()
    }

    fn assemble_key(&self, value: serde_json::Value) -> OriginalKey {
        OriginalKey { value: value, vtable: KeyVtable::of::<Self>() }
    }

    fn source_type_id(&self) -> &'static str {
        Self::TYPE_IDENT
    }

    fn as_serialize(&self) -> &dyn erased_serde::Serialize { self }
}

#[doc(hidden)]
pub struct SourceLoader(
    pub &'static str,
    pub Box<fn(v: serde_json::Value) -> Result<Box<dyn ErasedDesktopBackgroundSource>, serde_json::Error>>
);

#[derive(Debug)]
pub enum SourceLoadError {
    Deserialize(serde_json::Error),
    IdNotFound,
}

impl From<serde_json::Error> for SourceLoadError {
    fn from(err: serde_json::Error) -> SourceLoadError {
        SourceLoadError::Deserialize(err)
    }
}

pub fn load_source_by_id(id: &str, data: serde_json::Value) -> Result<Box<dyn ErasedDesktopBackgroundSource>, SourceLoadError> {
    for loader in inventory::iter::<SourceLoader> {
        if loader.0 == id {
            let source = (loader.1)(data)?;
            return Ok(source)
        }
    }
    Err(SourceLoadError::IdNotFound)
}

inventory::collect!(SourceLoader);