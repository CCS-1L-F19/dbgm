use std::borrow::Cow;

use crate::app::DBGM;
use crate::background::*;
use crate::sources::{DesktopBackgroundSource, OriginalKey, ChangeKind};

mod bglist;
mod resources;
mod modals;
mod widgets;
mod utils;

mod prelude {
    pub(in super) use imgui::*;
    pub(in super) use super::{resources::GuiResources, GuiState, modals, widgets, utils};
    pub(in super) use super::utils::{UiExt, /* ChildWindowExt, */ AUTO_SIZE};
    pub(in super) use crate::renderer::{Texture, Textures};
}

use self::prelude::*;
use utils::ImageCache;
use modals::{Modal, ChangeSetInfo};

pub struct GuiState<'a> {
    modal: Option<Modal>,
    pub(crate) dbgm: &'a mut DBGM,
    image_cache: ImageCache<OriginalKey>,
    resources: GuiResources,
    filter: bglist::Filter,
    selected_background: Option<usize>,
    debug: bool,
}

impl<'a> GuiState<'a> {
    pub fn new<T: Textures + ?Sized>(dbgm: &'a mut DBGM, textures: &mut T) -> Self {
        GuiState {
            modal: None, 
            dbgm: dbgm, 
            image_cache: ImageCache::new(), 
            resources: GuiResources::load(textures),
            filter: Default::default(),
            selected_background: None,
            debug: false,
        }
    }

    fn in_window(ui: &Ui, contents: impl FnOnce()) {
        let wr = ui.push_style_var(StyleVar::WindowRounding(0.0));
        Window::new(im_str!("Desktop Background Manager"))
            .position([0.0, 0.0], Condition::FirstUseEver)
            .size(ui.io().display_size, Condition::Always)
            .flags(WindowFlags::NO_TITLE_BAR | WindowFlags::NO_DECORATION | WindowFlags::NO_MOVE | WindowFlags::MENU_BAR)
            .build(ui, || {
                let wr = ui.push_style_var(StyleVar::WindowRounding(1.0));
                contents();
                wr.pop(ui);
            });
        wr.pop(ui);
    }

    pub fn update<T: Textures + ?Sized>(&mut self, ui: &Ui, textures: &mut T) -> bool {
        if self.debug { ui.show_metrics_window(&mut self.debug); }
        Self::in_window(ui, || {
            self.check_modal(ui);
            ui.menu_bar(|| self.draw_menu_bar(ui));
            if let Some(set) = self.dbgm.background_set() {
                let name = set.name().unwrap_or("(unnamed set)");
                let folder = set.image_folder().map(|f| f.to_string_lossy()).unwrap_or(Cow::from("(no image folder)"));
                let text = im_str!("{} - {}", name, folder);
                ui.pad_to_center_h(ui.calc_text_size(&text, false, -1.0)[0]);
                ui.text(&text);
                ui.separator();
            }
            let window_width = ui.content_region_max()[0];
            ui.columns(2, im_str!("MainColumns"), true);
            ui.set_column_offset(1, 2.0 * window_width / 3.0);
            ui.next_column();
            self.draw_background_list(ui, textures);
        });
        true
    }

    fn draw_menu_bar(&mut self, ui: &Ui) {
        ui.menu(im_str!("File"), true, || {
            if MenuItem::new(im_str!("New background set...")).build(ui) {
                self.dbgm.open_background_set(BackgroundSet::new());
            }
            if MenuItem::new(im_str!("Open background set...")).build(ui) {
                
            }
            if MenuItem::new(im_str!("Edit set information...")).enabled(self.dbgm.background_set().is_some()).build(ui) {
                self.open_modal(ChangeSetInfo::new())
            }
            if MenuItem::new(im_str!("Show debug window")).build(ui) {
                self.debug = !self.debug;
            }
        });
    }

    fn add_source<S: for<'s> DesktopBackgroundSource<'s> + 'static>(&mut self, source: S) {
        use modals::confirm_changes::*;
        let set = self.dbgm.background_set_mut().expect("Cannot add source when no background set is open!");
        let id = set.add_source(source);
        let mut result_cache = ResultCache::new();
        result_cache.put::<()>(&ChangeKind::New, ChangeResult::Accept, false);
        ConfirmChanges::new(id, set.sources_mut()[id].reload(), result_cache).apply_many(self);
    }

    fn select_background(&mut self, background: usize) {
        
    }
}


