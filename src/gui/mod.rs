use std::borrow::Cow;
use imgui::*;
use crate::{app::DBGM, background::*, source::OriginalKey};

mod bglist;
mod resources;
mod modals;
mod utils;

use self::utils::*;
use self::modals::Modal;
use self::resources::GuiResources;

pub use self::utils::{Textures, ImageCache};

pub const AUTO_SIZE: [f32; 2] = [0.0, 0.0];

pub struct GuiState<'a> {
    modal: Option<Modal>,
    pub(crate) dbgm: &'a mut DBGM,
    image_cache: ImageCache<OriginalKey>,
    resources: GuiResources,
}

impl<'a> GuiState<'a> {
    pub fn new<T: Textures + ?Sized>(dbgm: &'a mut DBGM, textures: &mut T) -> Self {
        GuiState {
            modal: None, 
            dbgm: dbgm, 
            image_cache: ImageCache::new(), 
            resources: GuiResources::load(textures),
        }
    }

    fn open_modal(&mut self, modal: Modal) {
        self.modal = Some(modal);
    }

    fn check_modal(&mut self, ui: &Ui) {
        if let Some(modal) = self.modal.take() {
            let id = im_str!("###{}", modal.get_id()).to_owned();
            if !ui.is_popup_open(&id) { ui.open_popup(&id); }
            let mut new = None;
            let id_with_title = im_str!("{}###{}", modal.get_title(), id.to_str());
            modal.open_with(PopupModal::new(ui, &id_with_title)).build(|| new = modal.display(ui, self));
            match &new {
                Some(m) if im_str!("###{}", m.get_id()) != id => {
                    ui.close_current_popup();
                    ui.open_popup(&im_str!("###{}", m.get_id()));
                }
                None => ui.close_current_popup(),
                _ => {},
            }
            self.modal = new;
        }
    }

    pub fn update<T: Textures + ?Sized>(&mut self, ui: &Ui, textures: &mut T) -> bool {
        Self::in_window(ui, || {
            self.check_modal(ui);
            ui.menu_bar(|| self.draw_menu_bar(ui));
            if let Some(set) = self.dbgm.background_set() {
                let name = set.name().unwrap_or("(unnamed set)");
                let folder = set.image_folder().map(|f| f.to_string_lossy()).unwrap_or(Cow::from("(no image folder)"));
                let text = im_str!("{} - {}", name, folder);
                ui.pad_to_center(ui.calc_text_size(&text, false, -1.0)[0]);
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
                self.open_modal(Modal::change_set_info())
            }
        });
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
}


