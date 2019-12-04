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

