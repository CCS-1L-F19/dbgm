use std::fmt::Debug;

use imgui::*;

use super::ModalInterface;
use crate::{
    gui::{GuiState, UiExt as _, utils::AUTO_SIZE},
    sources::{OriginalKey, CompareKey, KeyRelation, OriginalResult, OriginalChange, ChangeKind},
    background::{BackgroundSet, DesktopBackground},
};

pub struct ConfirmChanges { 
    source: usize, 
    changes: Vec<OriginalChange>,
    result_cache: ResultCache,
    show_error_details: bool,
}

impl ModalInterface for ConfirmChanges {
    fn id(&self) -> &str { "confirmchanges" }
    fn title(&self) -> &str { "Confirm changes" }
    fn display(mut self, ui: &Ui, state: &mut GuiState) {
        if let Some(change) = self.changes.pop() {
            let result = match &change.kind {
                ChangeKind::New => self.display_new(ui, state, &change.key),
                ChangeKind::Altered => self.display_altered(ui, state, &change.key),
                ChangeKind::Deleted => self.display_deleted(ui, state, &change.key),
                ChangeKind::Unavailable(cause) => self.display_unavailable(ui, state, &change.key, &cause),
            };

            match result {
                None => state.open_modal(self),
                Some(result) => {
                    self.result_cache.put(&change.kind, result, true); // TODO: Allow remembering answers
                    self.changes.push(change);
                    self.apply_many(state);
                }
            }
        }
    }
}

impl ConfirmChanges {
    pub fn new(source: usize, changes: Vec<OriginalChange>, result_cache: ResultCache) -> ConfirmChanges {
        ConfirmChanges { source, changes, result_cache, show_error_details: false }
    }

    pub fn apply_many(mut self, state: &mut GuiState) {
        let set = state.dbgm.background_set_mut().expect("Cannot incorporate changes when no background set is open!");
        while let Some(change) = self.changes.pop() {
            match self.result_cache.get(&change.kind) {
                None => {
                    self.changes.push(change);
                    state.open_modal(self);
                    return
                },
                Some(result) => self.apply_one(set, change, result),
            }
        }
    }

    fn apply_one(&mut self, set: &mut BackgroundSet, change: OriginalChange, result: ChangeResult) {
        let key = change.key;
        match (change.kind, result) {
            (ChangeKind::New, ChangeResult::Accept) => {
                let original = set.sources[self.source].original(&key);
                let original = if let OriginalResult::Original(o) = original { o } else { panic!("Got an invalid key from reload!"); };
                set.backgrounds.push(DesktopBackground::from_original(self.source, key, original))
            },
            (ChangeKind::Deleted, ChangeResult::Accept) => {
                set.backgrounds_mut().retain(|b| b.original.compare(&key) == KeyRelation::Distinct) 
            },
            (ChangeKind::Altered, ChangeResult::Accept) => {
                for background in set.backgrounds.iter_mut().filter(|b| b.original.compare(&key) != KeyRelation::Distinct) {
                    let original = set.sources[self.source].original(&key);
                    let original = if let OriginalResult::Original(o) = original { o } else { panic!("Got an invalid key from reload!"); };
                    background.update_from(original);
                }
            }
            (ChangeKind::Unavailable(_), _) => { /* TODO */ },
            _ => unimplemented!(),
        }
    }

    fn display_altered(&mut self, ui: &Ui, state: &mut GuiState, key: &OriginalKey) -> Option<ChangeResult> {
        ui.text("The content of the original associated with the following backgrounds has changed:");
        //TODO: Show backgrounds here
        ui.text("\
            If you choose Accept Change, these backgrounds will be marked as unedited and use the new original.\
            If you choose Reject Change, these backgrounds will remain as they are, but will be marked as \
            missing an original, and become unable to be edited.
        ");

        if ui.button(im_str!("Accept Change"), AUTO_SIZE) { return Some(ChangeResult::Accept); }
        ui.same_line(0.0);
        if ui.button(im_str!("Reject Change"), AUTO_SIZE) { return Some(ChangeResult::Reject); }
        None
    }

    fn display_deleted(&mut self, ui: &Ui, state: &mut GuiState, key: &OriginalKey) -> Option<ChangeResult> {
        ui.text("The original associated with the following backgrounds no longer exists:");
        // TODO: Show backgrounds here
        ui.text("\
            If you choose Remove, these backgrounds will be deleted from the library. \
            If you choose Don't Remove, these backgrounds will remain in the library, but will be marked as \
            missing an original, and will not be able to be edited.
        ");

        if ui.button(im_str!("Remove"), AUTO_SIZE) { return Some(ChangeResult::Accept); }
        ui.same_line(0.0);
        if ui.button(im_str!("Don't Remove"), AUTO_SIZE) { return Some(ChangeResult::Reject); }
        None
    }

    fn display_unavailable(&mut self, ui: &Ui, state: &mut GuiState, key: &OriginalKey, cause: &dyn Debug) -> Option<ChangeResult> {
        ui.text("\
            The original associated with the following backgrounds cannot be accessed. This condition may be temporary\
            or permanent. You will not be able to edit any of the backgrounds until the original becomes available again.
        ");
        // TODO: Show backgrounds here
        if ui.button(im_str!("OK"), AUTO_SIZE) { return Some(ChangeResult::Accept); }
        ui.same_line(0.0);
        ui.toggle_button(&im_str!("ShowDetails"), "Hide Error Details", "Show Error Details", &mut self.show_error_details);
        if self.show_error_details {
            let mut details = im_str!("{:#?}", cause);
            InputTextMultiline::new(ui, &im_str!("###Error Details"), &mut details, AUTO_SIZE).read_only(true).build();
        }
        None
    }

    fn display_new(&mut self, ui: &Ui, state: &mut GuiState, key: &OriginalKey) -> Option<ChangeResult> {
        ui.text("A new background is available. Would you like to add it to the library?");
        //TODO: Show background here

        if ui.button(im_str!("Add"), AUTO_SIZE) { return Some(ChangeResult::Accept); }
        ui.same_line(0.0);
        if ui.button(im_str!("Don't Add"), AUTO_SIZE) { return Some(ChangeResult::Reject); }
        None
    }
}

#[derive(Copy, Clone)]
pub enum ChangeResult { Accept, Reject } 

pub struct ResultCache {
    new: Option<(ChangeResult, bool)>,
    altered: Option<(ChangeResult, bool)>,
    deleted: Option<(ChangeResult, bool)>,
    unavailable: Option<(ChangeResult, bool)>,
}

impl ResultCache {
    pub fn new() -> ResultCache {
        ResultCache { new: None, altered: None, deleted: None, unavailable: None }
    }

    fn select<E: std::fmt::Debug>(&mut self, kind: &ChangeKind<E>) -> &mut Option<(ChangeResult, bool)> {
        match kind {
            ChangeKind::New => &mut self.new,
            ChangeKind::Altered => &mut self.altered,
            ChangeKind::Deleted => &mut self.deleted,
            ChangeKind::Unavailable(_) => &mut self.unavailable,
        }
    }

    pub fn get<E: std::fmt::Debug>(&mut self, kind: &ChangeKind<E>) -> Option<ChangeResult> {
        let loc = self.select(kind);
        match loc.take() {
            Some((result, true)) => { *loc = Some((result, true)); Some(result) }
            Some((result, false)) => Some(result),
            None => None,
        }
    }

    pub fn put<E: std::fmt::Debug>(&mut self, kind: &ChangeKind<E>, result: ChangeResult, once: bool) {
        let loc = self.select(kind);
        if let None | Some((_, false)) = loc {
            *loc = Some((result, once));
        }
    }
}

