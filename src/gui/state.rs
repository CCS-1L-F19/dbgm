use std::ops::{Deref, DerefMut};

use crate::{
    background::*,
    sources::*, 
    gui::prelude::*
};

use modals::{Modal, RemoveSource, confirm_changes::*};
use super::bglist::Filter;

pub enum Operation {
    ReloadSource(usize),
    RemoveSource(usize),
    SelectBackground(usize),
    ChangeFlags(usize, DesktopBackgroundFlags),
}

pub struct ActiveSet {
    pub set: BackgroundSet,
    pub image_cache: ImageCache<OriginalKey>,
}

impl Deref for ActiveSet {
    type Target = BackgroundSet;
    fn deref(&self) -> &BackgroundSet {
        &self.set
    }
}

impl DerefMut for ActiveSet {
    fn deref_mut(&mut self) -> &mut BackgroundSet {
        &mut self.set
    }
}

pub struct GuiState {
    pub(in super) modal: Option<Modal>,
    pub(in super) set: Option<ActiveSet>,
    pub(in super) filter: Filter,
    pub(in super) selected_background: Option<usize>,
    pub(in super) debug: bool,
}

impl Default for GuiState {
    fn default() -> GuiState {
        GuiState {
            modal: None, 
            set: None,
            filter: Default::default(),
            selected_background: None,
            debug: false,
        }
    }
}

// Commands
impl GuiState {
    pub(in super) fn apply(&mut self, operation: Operation) {
        match operation {
            Operation::ReloadSource(source) => self.reload_source(source),
            Operation::RemoveSource(source) => self.open_modal(RemoveSource(source)),
            Operation::SelectBackground(background) => self.select_background(background),
            Operation::ChangeFlags(background, flags) => {
                if let Some(set) = &mut self.set {
                    set.backgrounds[background].flags = flags;
                }
            }
        }
    }

    pub(in super) fn add_source<S: for<'s> DesktopBackgroundSource<'s> + 'static>(&mut self, source: S) {
        let set = self.set.as_mut().expect("Cannot add source when no background set is open!");
        let id = set.add_source(source);
        let mut result_cache = ResultCache::new();
        result_cache.put::<()>(&ChangeKind::New, ChangeResult::Accept, false);
        ConfirmChanges::new(id, set.sources[id].reload(), result_cache).apply_many(self);
    }

    pub(in super) fn reload_source(&mut self, id: usize) {
        let set = self.set.as_mut().expect("Cannot reload source when no background set is open!");
        ConfirmChanges::new(id, set.sources[id].reload(), ResultCache::new()).apply_many(self);
    }

    // TODO: Support multiple selection?
    pub(in super) fn select_background(&mut self, background: usize) {
        assert!(self.set.as_ref().map(|b| b.backgrounds.has_element_at(background)).unwrap_or(false));
        self.selected_background = if self.selected_background != Some(background) { Some(background) } else { None }
    }

    // TODO: Prompt, save current set.
    pub(in super) fn open_background_set(&mut self, set: BackgroundSet) {
        self.set = Some(ActiveSet { set, image_cache: ImageCache::new() });
    }
}