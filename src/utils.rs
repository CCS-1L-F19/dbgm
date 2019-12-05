use std::ops::Deref;
use winapi::um::{
    winnt::HRESULT,
    winuser::{GetSystemMetrics, SM_CXSCREEN, SM_CYSCREEN}
};

#[inline]
pub fn check_result(result: HRESULT) -> Result<(), std::io::Error> {
    if result < 0 {  Err(std::io::Error::from_raw_os_error(result)) } else { Ok(()) }
}

pub fn primary_monitor_resolution() -> (usize, usize) {
    unsafe { (GetSystemMetrics(SM_CXSCREEN) as usize, GetSystemMetrics(SM_CYSCREEN) as usize) }
}

// Use this until Option::deref stabilizes.
pub trait OptionExt<T: Deref> {
    fn deref(&self) -> Option<&T::Target>;
}

impl<T: Deref> OptionExt<T> for Option<T> {
    fn deref(&self) -> Option<&T::Target> {
        self.as_ref().map(|t| t.deref())
    }
}

pub trait Flatten<T> {
    fn flatten(self) -> Option<T>;
}

impl<T> Flatten<T> for Option<Option<T>> {
    fn flatten(self) -> Option<T> {
        match self {
            None => None,
            Some(v) => v,
        }
    }
}

macro_rules! register_source_type {
    {$source:ty} => {
        ::inventory::submit! {
            $crate::sources::SourceLoader(<$source>::TYPE_IDENT, Box::new(|v| {
                serde_json::from_value::<$source>(v).map(|s| Box::new(s) as Box<_>)
            }))
        }
    }
}

pub mod as_pairs {
    use std::collections::HashMap;
    use std::hash::Hash;
    use serde::{Serialize, Serializer, Deserialize, Deserializer};

    pub fn serialize<K, V, S>(map: &HashMap<K, V>, serializer: S) -> Result<S::Ok, S::Error> 
        where K: Serialize, V: Serialize, S: Serializer
    {
        let pairs: Vec<(&K, &V)> = map.iter().collect();
        Serialize::serialize(&pairs, serializer)
    }

    pub fn deserialize<'de, K, V, D>(deserializer: D) -> Result<HashMap<K, V>, D::Error> 
        where K: Deserialize<'de> + Hash + Eq, V: Deserialize<'de>, D: Deserializer<'de>
    {
        let pairs: Vec<(K, V)> = Deserialize::deserialize(deserializer)?;
        Ok(pairs.into_iter().collect())
    }
}

