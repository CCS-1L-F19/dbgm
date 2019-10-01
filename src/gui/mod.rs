use std::borrow::Cow;
use imgui::*;
use crate::{app::DBGM, background::*, source::OriginalKey};

#[macro_use]
mod utils;

use self::utils::*;
use self::popups::Modal;

pub use self::utils::{Textures, ImageCache};

pub const AUTO_SIZE: [f32; 2] = [0.0, 0.0];

pub(crate) mod popups {
    use std::borrow::Cow;
    use std::fmt::Debug;
    use std::path::PathBuf;

    use imgui::*;

    use crate::{OptionExt as _};
    use super::{AUTO_SIZE, GuiState};
    use self::Modal::*;

    pub enum Modal {
        Error(String, Option<Box<Debug>>),
        ChangeSetInfo {
            image_folder: Option<PathBuf>,
            name_buf: ImString,
        }
    }

    impl Modal {
        pub fn get_id(&self) -> &str {
            match self {
                Error(..) => "error",
                ChangeSetInfo { .. } => "changesetinfo",
            }
        }

        pub fn get_title(&self) -> &str {
            match self {
                Error(..) => "Error",
                ChangeSetInfo { .. } => "Background set information",
            }
        }

        pub fn open_with<'ui, 'p>(&self, modal: PopupModal<'ui, 'p>) -> PopupModal<'ui, 'p> {
            modal.always_auto_resize(true)
        }

        pub fn display(mut self, ui: &Ui, state: &mut GuiState) -> Option<Modal> {
            match &mut self {
                Error(message, info) => {
                    ui.text(im_str!("{} {}", message, info.as_ref().map(|e| format!("Details: {:?}", e)).unwrap_or("".to_string())));
                    let ok_label = im_str!("OK");
                    if ui.button(ok_label, AUTO_SIZE) {
                        return None
                    }
                }
                ChangeSetInfo { image_folder, name_buf } => {
                    let set = state.dbgm.background_set_mut().expect("Cannot view set information when no background set is open!");
                    ui.input_text(im_str!("Name"), name_buf).flags(imgui::ImGuiInputTextFlags::CallbackResize).build();
                    ui.new_line();
                    
                    let display_folder = image_folder.deref().or(set.image_folder()).map(|f| f.to_string_lossy()).unwrap_or(Cow::from("(none)"));
                    ui.input_text(im_str!("Image folder"), &mut ImString::new(display_folder)).read_only(true).build();
                    ui.same_line(0.0);
                    if ui.button(im_str!("Choose..."), AUTO_SIZE) {
                        match nfd::open_pick_folder(None) {
                            Ok(nfd::Response::Okay(f)) => {
                                match f.parse() {
                                    Ok(path) => *image_folder = Some(path),
                                    Err(e) => return Some(Modal::error("Invalid path to image folder.".to_string(), Some(e))),
                                }
                            }
                            Err(e) => return Some(Modal::error("Could not open image folder picker.".to_string(), Some(e))),
                            _ => {},
                        }
                    }
                    ui.new_line();

                    if ui.button(im_str!("OK"), AUTO_SIZE) {
                        if let Some(folder) = image_folder { set.set_image_folder(folder); }
                        if name_buf.to_str().trim() != "" { set.set_name(name_buf.to_str().to_string()); }
                        return None
                    }
                    ui.same_line(0.0);
                    if ui.button(im_str!("Cancel"), AUTO_SIZE) { return None }
                }
            }
            return Some(self)
        }
    }

    impl Modal {
        pub fn error(message: impl AsRef<str>, error: Option<impl Debug + 'static>) -> Modal {
            Error(message.as_ref().to_string(), error.map(|d| Box::new(d) as Box<dyn Debug>))
        }

        pub fn change_set_info() -> Modal {
            ChangeSetInfo { image_folder: None, name_buf: ImString::new("") } // TODO: Add proper resizing support
        }
    }
}

struct BackgroundListEntry {
    name: String,
    source_id: usize,
    original: Option<OriginalEntry>,
}

struct OriginalEntry {
    texture: Option<TextureId>,
    location: String,
    changed: bool,
}

struct GuiResources {
    missing_image: TextureId,
}

impl GuiResources {
    fn load<T: Textures + ?Sized>(textures: &mut T) -> GuiResources {
        GuiResources {
            missing_image: load_internal_texture!(textures, "../resources/missing.png"),
        }
    }
}

pub struct GuiState<'a> {
    modal: Option<popups::Modal>,
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
            ui.menu_bar(|| self.render_menu_bar(ui));
            if let Some(set) = self.dbgm.background_set() {
                let name = set.name().unwrap_or("(unnamed set)");
                let folder = set.image_folder().map(|f| f.to_string_lossy()).unwrap_or(Cow::from("(no image folder)"));
                let text = im_str!("{} - {}", name, folder);
                ui.pad_to_center(ui.calc_text_size(&text, false, -1.0)[0]);
                ui.text(&text);
                ui.separator();
            }
            let window_width = ui.content_region_max()[0];
            ui.text(im_str!("Test!"));
            ui.columns(2, im_str!("MainColumns"), true);
            ui.set_column_offset(1, 2.0 * window_width / 3.0);
            ui.next_column();
            self.render_background_list(ui, textures);
        });
        true
    }

    fn generate_background_entries<T: Textures + ?Sized>(&mut self, textures: &mut T) -> Vec<Vec<BackgroundListEntry>> {
        use crate::source::OriginalResult;
        match self.dbgm.background_set() {
            Some(set) => {
                let mut entries = (0..set.sources().len()).map(|_| Vec::new()).collect::<Vec<_>>();
                for background in set.backgrounds() {
                    let original = match set.sources()[background.source].original(&background.original) {
                        OriginalResult::Original(original) => Some(OriginalEntry {
                            texture: {
                                if !self.image_cache.contains_image(&background.original) {
                                    if let Ok(image) = original.read_image() {
                                        self.image_cache.insert_image(background.original.clone(), image);
                                    }
                                }
                                match self.image_cache.load_texture(&background.original, textures) {
                                    Some(Ok(texture)) => Some(texture),
                                    _ => None
                                }
                            },
                            location: original.location(),
                            changed: false,
                        }),
                        OriginalResult::ContentMismatch(original) => Some(OriginalEntry {
                            texture: {
                                match original.read_image() {
                                    Ok(image) => self.image_cache.insert_image(background.original.clone(), image),
                                    Err(_) => { self.image_cache.remove_image(&background.original); }
                                }
                                match self.image_cache.load_texture(&background.original, textures) {
                                    Some(Ok(texture)) => Some(texture),
                                    _ => None
                                }
                            },
                            location: original.location(),
                            changed: true,
                        }),
                        _ => None,
                    };
                    entries[background.source].push(BackgroundListEntry { 
                        name: background.name.clone(), 
                        source_id: background.source, 
                        original: original 
                    });
                }
                entries
            }
            None => Vec::new()
        }
    }

    fn render_background_list<T: Textures + ?Sized>(&mut self, ui: &Ui, textures: &mut T) {
        let entries = self.generate_background_entries(textures);
        ChildWindow::new(im_str!("background list")).build(ui, || {
            if let Some(set) = self.dbgm.background_set() {
                for (i, bgs) in entries.into_iter().enumerate() {
                    if !bgs.is_empty() {
                        let source = &set.sources()[i];
                        if ui.collapsing_header(&im_str!("{}###Source{}", source.name(), i)).build() {
                            for (j, bg) in bgs.into_iter().enumerate() {
                                self.render_background_entry(ui, (i, j), bg);
                            }
                        }
                    }   
                }
            }
        })
    }

    fn render_background_entry(&self, ui: &Ui, id: (usize, usize), background: BackgroundListEntry) {
        let entry_id = ui.push_id(&im_str!("Source{}Background{}", id.0, id.1));
        let child = ChildWindow::new(im_str!("BackgroundFrame")).build(ui, || {
            ui.columns(2, im_str!("Columns"), false);
            let texture = background.original.and_then(|o| o.texture).unwrap_or(self.resources.missing_image);
            Image::new(texture, AUTO_SIZE).build(ui);
        });
        entry_id.pop(ui);
    }

    fn render_menu_bar(&mut self, ui: &Ui) {
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
        ui.push_style_var(StyleVar::WindowRounding(0.0));
        Window::new(im_str!("Desktop Background Manager"))
            .position([0.0, 0.0], Condition::FirstUseEver)
            .size(ui.io().display_size, Condition::Always)
            .flags(WindowFlags::NO_TITLE_BAR | WindowFlags::NO_DECORATION | WindowFlags::NO_MOVE | WindowFlags::MENU_BAR)
            .build(ui, || {
                ui.push_style_var(StyleVar::WindowRounding(1.0));
                contents()
            });
    }
}


