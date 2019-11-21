use std::ops::Deref;
use winapi::um::winnt;

#[inline]
pub fn check_result(result: winnt::HRESULT) -> Result<(), std::io::Error> {
    if result < 0 {  Err(std::io::Error::from_raw_os_error(result)) } else { Ok(()) }
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

#[derive(Clone)]
pub enum MaybeStale<T> {
    Fresh(T),
    Stale(T),
    Unknown,
}

impl<T> MaybeStale<T> {
    pub fn update(&mut self, value: Option<T>) {
        match value {
            Some(value) => *self = MaybeStale::Fresh(value),
            None => if let MaybeStale::Fresh(val) | MaybeStale::Stale(val) = std::mem::replace(self, MaybeStale::Unknown) {
                *self = MaybeStale::Stale(val);
            }
        }
    }

    pub fn value(&self) -> Option<&T> {
        match self {
            MaybeStale::Fresh(value) | MaybeStale::Stale(value) => Some(value),
            MaybeStale::Unknown => None,
        }
    }
}

impl<T> From<Option<T>> for MaybeStale<T> {
    fn from(opt: Option<T>) -> MaybeStale<T> {
        let mut value = MaybeStale::Unknown;
        value.update(opt);
        value
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

