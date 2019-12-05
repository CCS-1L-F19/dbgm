use std::borrow::Cow;

use crate::background::BackgroundSet;
use crate::gui::prelude::*;

use modals::{ChangeSetInfo, ErrorModal};

pub struct Frame<'f, T: ?Sized> {
    pub ui: &'f Ui<'f>,
    pub textures: &'f mut T,
    pub resources: &'f GuiResources,
}

pub fn draw_state<T: Textures + ?Sized>(state: &mut GuiState, frame: Frame<T>) -> bool {
    state.draw(frame)
}

impl GuiState {
    fn draw<T: Textures + ?Sized>(&mut self, frame: Frame<T>) -> bool {
        if self.debug { frame.ui.show_metrics_window(&mut self.debug); }
        frame.ui.fullscreen_window(im_str!("Desktop Background Manager"), || {
            self.check_modal(reborrow_frame!(frame));
            let ui = &frame.ui;
            ui.menu_bar(|| self.draw_menu_bar(ui));
            if let Some(set) = &self.set {
                let name = set.name().unwrap_or("(unnamed set)");
                let folder = set.image_folder().map(|f| f.to_string_lossy()).unwrap_or(Cow::from("(no image folder)"));
                let text = im_str!("{} - {}", name, folder);
                ui.center_avail_h(ui.calc_text_size(&text, false, -1.0)[0]);
                ui.text(&text);
                ui.separator();
            }
            let window_width = ui.content_region_max()[0];
            ui.columns(2, im_str!("MainColumns"), true);
            ui.set_column_offset(1, window_width * 2.0 / 3.0);
            if let (Some(set), Some(background)) = (self.set.as_mut(), self.selected_background) {
                self.draw_editor(reborrow_frame!(frame), background);
            } else {
                let text = match self.set.is_some() {
                    true => "Select a background from the list on the right to begin editing.",
                    false => "Open a background set to create and edit desktop backgrounds.",
                };
                let size = ui.calc_text_size(&im_str!("{}", text), false, -1.0);
                ui.center_avail_h(size[0]);
                ui.center_avail_v(size[1]);
                ui.text(text);
            }
            ui.next_column();
            self.draw_background_list(frame);
        });
        true
    }

    fn draw_menu_bar(&mut self, ui: &Ui) {
        ui.menu(im_str!("File"), true, || {
            if MenuItem::new(im_str!("New background set...")).build(ui) {
                self.open_background_set(BackgroundSet::new(crate::utils::primary_monitor_resolution()));
            }
            if MenuItem::new(im_str!("Open background set...")).build(ui) {
                match utils::nfd_handler(nfd::open_file_dialog(Some("bgs"), None), "background set") {
                    Ok(Some(path)) => match BackgroundSet::load(&path) {
                        Ok((set, warnings)) => self.open_background_set(set),
                        Err(e) => {
                            let msg = format!("The background set at {} could not be loaded.", path.to_string_lossy());
                            self.open_modal(ErrorModal::new(msg, Some(e)));
                        }
                    }
                    Err(modal) => self.open_modal(modal),
                    _ => {}
                }
            }
            if MenuItem::new(im_str!("Save background set as...")).enabled(self.set.is_some()).build(ui) {
                // TODO: Use the last path our set was saved as, if available
                match utils::nfd_handler(nfd::open_save_dialog(Some("bgs"), None), "save location") { 
                    Ok(Some(path)) => match self.set.as_ref().unwrap().save(path) {
                        Ok(_) => { /* TODO: Indicate success somehow? */ },
                        Err(e) => self.open_modal(ErrorModal::new("An error occured while saving the background set.", Some(e)))
                    }
                    Err(modal) => self.open_modal(modal),
                    _ => {}
                }
            }
            if MenuItem::new(im_str!("Edit set information...")).enabled(self.set.is_some()).build(ui) {
                self.open_modal(ChangeSetInfo::new())
            }
            if MenuItem::new(im_str!("Show debug window")).build(ui) {
                self.debug = !self.debug;
            }
        });
    }
}