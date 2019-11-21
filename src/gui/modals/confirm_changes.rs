#![allow(unused_variables)]

use std::fmt::Debug;

use super::ModalInterface;
use crate::{
    gui::prelude::*,
    sources::{OriginalKey, CompareKey, KeyRelation, OriginalResult, OriginalChange, ChangeKind},
    background::{BackgroundSet, DesktopBackground, DesktopBackgroundFlags},
};

use widgets::{BackgroundCard, BackgroundGrid, CardOriginalInfo};

const ALTERED_DESCRIPTION: &'static str = "\
    If you choose Accept Change, these backgrounds will be marked as unedited and use the new original.\
    If you choose Reject Change, these backgrounds will remain as they are, but will be marked as \
    missing an original, and become unable to be edited.";

const DELETED_DESCRIPTION: &'static str = "\
    If you choose Remove, these backgrounds will be deleted from the library. \
    If you choose Don't Remove, these backgrounds will remain in the library, but will be marked as \
    missing an original, and will not be able to be edited.";

const UNAVAILABLE_DESCRIPTION: &'static str = "\
    The original associated with the following backgrounds cannot be accessed. This condition may be temporary\
    or permanent. You will not be able to edit any of the backgrounds until the original becomes available again.";

pub struct ConfirmChanges { 
    source: usize, 
    changes: Vec<OriginalChange>,
    result_cache: ResultCache,
    show_error_details: bool,
}

impl ModalInterface for ConfirmChanges {
    fn id(&self) -> &str { "confirmchanges" }
    fn title(&self) -> &str { "Confirm changes" }

    fn open_with<'ui, 'p>(&self, ui: &'ui Ui, modal: PopupModal<'ui, 'p>) -> PopupModal<'ui, 'p> {
        let modal = modal.always_auto_resize(true);
        let style = ui.clone_style();
        let max_content_width = (ui.io().display_size[0] / 2.0) - 2.0 * (style.window_border_size + style.window_padding[0]);
        match self.changes.last().map(|c| &c.kind) {
            Some(ChangeKind::Altered) => 
                modal.size([ui.calc_text_size(&im_str!("{}", ALTERED_DESCRIPTION), false, max_content_width)[0], 0.0]),
            Some(ChangeKind::Deleted) => 
                modal.size([ui.calc_text_size(&im_str!("{}", DELETED_DESCRIPTION), false, max_content_width)[0], 0.0]),
            _ => modal,
        }
    }
    
    fn display<T: Textures + ?Sized>(mut self, state: &mut GuiState, frame: Frame<T>) {
        let ui = frame.ui;
        if let Some(change) = self.changes.pop() {
            let frame2 = reborrow_frame!(frame);
            let set = state.set.as_mut().expect("Cannot confirm changes when no set is open!");
            let result = match &change.kind {
                ChangeKind::New => self.display_new(set, frame2, &change.key),
                ChangeKind::Altered => self.display_altered(set, frame2, &change.key),
                ChangeKind::Deleted => self.display_deleted(set, frame2, &change.key),
                ChangeKind::Unavailable(cause) => self.display_unavailable(set, frame2, &change.key, &cause),
            };

            match result {
                None => {
                    self.changes.push(change);
                    state.open_modal(self);
                },
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
        let set = state.set.as_mut().expect("Cannot incorporate changes when no background set is open!");
        while let Some(change) = self.changes.pop() {
            println!("Change: {:?}", &change.kind);
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
            (ChangeKind::New, result) => {
                let original = set.sources[self.source].original(&key);
                let original = if let OriginalResult::Original(o) = original { o } else { panic!("Got an invalid key from reload!"); };
                let new_id = set.backgrounds.push(DesktopBackground::from_original(self.source, key, original));
                if result == ChangeResult::Reject { 
                    set.backgrounds[new_id].flags.set(DesktopBackgroundFlags::EXCLUDED, true);
                }
            },
            (ChangeKind::Altered, ChangeResult::Accept) => {
                for background in set.backgrounds.values_mut().filter(|b| b.original.compare(&key) != KeyRelation::Distinct) {
                    let original = set.sources[self.source].original(&key);
                    let original = if let OriginalResult::Original(o) = original { o } else { panic!("Got an invalid key from reload!"); };
                    background.update_from(key.clone(), original);
                }
            },
            (ChangeKind::Deleted, ChangeResult::Accept) => {
                set.backgrounds.retain(|b| b.original.compare(&key) == KeyRelation::Distinct) 
            },
            (ChangeKind::Deleted, ChangeResult::Reject) | (ChangeKind::Altered, ChangeResult::Reject) => {
                for background in set.backgrounds.values_mut().filter(|b| b.original.compare(&key) != KeyRelation::Distinct) {
                    background.flags.insert(DesktopBackgroundFlags::ORIGINAL_MISSING);
                }
            },
            (ChangeKind::Unavailable(_), _) => { 
                for background in set.backgrounds.values_mut().filter(|b| b.original.compare(&key) == KeyRelation::SameOriginal) {
                    background.flags.insert(DesktopBackgroundFlags::ORIGINAL_UNAVAILABLE);
                }
            },
        }
    }

    fn display_affected<T: Textures + ?Sized>(set: &mut ActiveSet, frame: Frame<T>, key: &OriginalKey) {
        let ui = frame.ui;
        let affected_backgrounds = set.backgrounds.indices().collect::<Vec<_>>().into_iter().filter_map(|id| {
            if set.backgrounds[id].original.compare(key) == KeyRelation::Distinct { return None };
            Some((id, CardOriginalInfo::try_load_from_set(set, id, &mut *frame.textures)))
        }).collect::<Vec<_>>();

        let grid = BackgroundGrid {
            id: &im_str!("AffectedBackgrounds"),
            entries: affected_backgrounds,
            card_width: ui.current_font_size() * 25.0, // TODO: Is there a less arbitrary choice here
            max_size: [0.0, (ui.io().display_size[1] * 2.0 / 3.0) - (ui.window_content_region_min()[1] - ui.cursor_pos()[1])].into(),
        };
        ui.center_avail_h(grid.size(ui).x);
        grid.draw(set, reborrow_frame!(frame));
    }

    fn display_altered<T: Textures + ?Sized>(&mut self, set: &mut ActiveSet, frame: Frame<T>, key: &OriginalKey) -> Option<ChangeResult> {
        let ui = frame.ui;

        ui.text("The content of the original associated with the following backgrounds has changed:");
        ui.spacing();
        ConfirmChanges::display_affected(set, reborrow_frame!(frame), key);
        ui.spacing();
        ui.text_wrapped(&im_str!("{}", ALTERED_DESCRIPTION));

        if ui.button(im_str!("Accept Change"), AUTO_SIZE) { return Some(ChangeResult::Accept); }
        ui.same_line(0.0);
        if ui.button(im_str!("Reject Change"), AUTO_SIZE) { return Some(ChangeResult::Reject); }
        None
    }

    fn display_deleted<T: Textures + ?Sized>(&mut self, set: &mut ActiveSet, frame: Frame<T>, key: &OriginalKey) -> Option<ChangeResult> {
        let ui = frame.ui;
        
        ui.text("The original associated with the following backgrounds no longer exists:");
        ui.spacing();
        ConfirmChanges::display_affected(set, reborrow_frame!(frame), key);
        ui.spacing();
        ui.text_wrapped(&im_str!("{}", DELETED_DESCRIPTION));

        if ui.button(im_str!("Remove"), AUTO_SIZE) { return Some(ChangeResult::Accept); }
        ui.same_line(0.0);
        if ui.button(im_str!("Don't Remove"), AUTO_SIZE) { return Some(ChangeResult::Reject); }
        None
    }

    fn display_unavailable<T: Textures + ?Sized>(&mut self, set: &mut ActiveSet, frame: Frame<T>, key: &OriginalKey, cause: &dyn Debug) -> Option<ChangeResult> {
        let ui = frame.ui;
        
        ui.text_wrapped(&im_str!("{}", UNAVAILABLE_DESCRIPTION));
        ui.spacing();
        ConfirmChanges::display_affected(set, reborrow_frame!(frame), key);
        ui.spacing();

        if ui.button(im_str!("OK"), AUTO_SIZE) { return Some(ChangeResult::Accept); }
        ui.same_line(0.0);
        ui.toggle_button_labeled(&im_str!("ShowDetails"), "Hide Error Details", "Show Error Details", &mut self.show_error_details);
        if self.show_error_details {
            let mut details = im_str!("{:#?}", cause);
            InputTextMultiline::new(ui, &im_str!("###Error Details"), &mut details, AUTO_SIZE).read_only(true).build();
        }
        None
    }

    fn display_new<T: Textures + ?Sized>(&mut self, set: &mut ActiveSet, frame: Frame<T>, key: &OriginalKey) -> Option<ChangeResult> {
        let Frame { ui, resources, textures } = frame;
        
        let original = set.set.sources[self.source].original(&key);
        let original = if let OriginalResult::Original(o) = original { o } else { panic!("Got an invalid key from reload!"); };
        let mut background = DesktopBackground::from_original(self.source, key.clone(), original);

        ui.text("A new background is available. Would you like to add it to the library?");
        ui.spacing();
        let card_width = ui.current_font_size() * 25.0;
        let card = BackgroundCard {
            id: im_str!("NewBackground"),
            resources: resources,
            original: Some(CardOriginalInfo::load(&mut background, original, &mut set.image_cache, textures)),
            background: &background,
            editable: false,
            width: card_width,
        };
        ui.center_avail_h(card_width);
        card.draw(ui);
        ui.spacing();

        if ui.button(im_str!("Add"), AUTO_SIZE) { return Some(ChangeResult::Accept); }
        ui.same_line(0.0);
        if ui.button(im_str!("Don't Add"), AUTO_SIZE) { return Some(ChangeResult::Reject); }
        None
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
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
            Some((result, false)) => { *loc = Some((result, false)); Some(result) }
            Some((result, true)) => Some(result),
            None => None,
        }
    }

    pub fn put<E: std::fmt::Debug>(&mut self, kind: &ChangeKind<E>, result: ChangeResult, once: bool) {
        let loc = self.select(kind);
        if let None | Some((_, true)) = loc {
            *loc = Some((result, once));
        }
    }
}

