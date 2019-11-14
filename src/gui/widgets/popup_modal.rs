#![allow(unused_mut, dead_code)]

use crate::gui::prelude::*;

use imgui::PopupModal;

pub struct PopupModal2<'ui, 'p> {
    inner: PopupModal<'ui, 'p>,
    size: Option<[f32; 2]>
}

impl<'ui, 'p> PopupModal2<'ui, 'p> {
    pub fn new(ui: &Ui<'ui>, label: &'p ImStr) -> Self {
        PopupModal2 { inner: PopupModal::new(ui, label), size: None }
    }

    fn map<T>(self, f: fn(PopupModal<'ui, 'p>, T) -> PopupModal<'ui, 'p>, param: T) -> PopupModal2<'ui, 'p> {
        PopupModal2{ inner: f(self.inner, param), ..self }
    }

    /// Pass a mutable boolean which will be updated to refer to the current
    /// "open" state of the modal.
    pub fn opened(mut self, opened: &'p mut bool) -> Self {
        self.map(PopupModal::opened, opened)
    }
    pub fn flags(mut self, flags: WindowFlags) -> Self {
        self.map(PopupModal::flags, flags)
    }
    pub fn title_bar(mut self, value: bool) -> Self {
        self.map(PopupModal::title_bar, value)
    }
    pub fn resizable(mut self, value: bool) -> Self {
        self.map(PopupModal::resizable, value)
    }
    pub fn movable(mut self, value: bool) -> Self {
        self.map(PopupModal::movable, value)
    }
    pub fn scroll_bar(mut self, value: bool) -> Self {
        self.map(PopupModal::scroll_bar, value)
    }
    pub fn scrollable(mut self, value: bool) -> Self {
        self.map(PopupModal::scrollable, value)
    }
    pub fn collapsible(mut self, value: bool) -> Self {
        self.map(PopupModal::collapsible, value)
    }
    pub fn always_auto_resize(mut self, value: bool) -> Self {
        self.map(PopupModal::always_auto_resize, value)
    }
    pub fn save_settings(mut self, value: bool) -> Self {
        self.map(PopupModal::save_settings, value)
    }
    pub fn inputs(mut self, value: bool) -> Self {
        self.map(PopupModal::inputs, value)
    }
    pub fn menu_bar(mut self, value: bool) -> Self {
        self.map(PopupModal::menu_bar, value)
    }
    pub fn horizontal_scrollbar(mut self, value: bool) -> Self {
        self.map(PopupModal::horizontal_scrollbar, value)
    }
    pub fn no_focus_on_appearing(mut self, value: bool) -> Self {
        self.map(PopupModal::no_focus_on_appearing, value)
    }
    pub fn no_bring_to_front_on_focus(mut self, value: bool) -> Self {
        self.map(PopupModal::no_bring_to_front_on_focus, value)
    }
    pub fn always_vertical_scrollbar(mut self, value: bool) -> Self {
        self.map(PopupModal::always_vertical_scrollbar, value)
    }
    pub fn always_horizontal_scrollbar(mut self, value: bool) -> Self {
        self.map(PopupModal::always_horizontal_scrollbar, value)
    }
    pub fn always_use_window_padding(mut self, value: bool) -> Self {
        self.map(PopupModal::always_use_window_padding, value)
    }

    pub fn size(mut self, size: [f32; 2]) -> Self {
        self.size = Some(size);
        self.always_auto_resize(false).resizable(false)
    }

    /// Consume and draw the PopupModal.
    pub fn build<F: FnOnce()>(self, ui: &'ui Ui, f: F) {
        if let Some(size) = self.size {
            let display_size = ui.io().display_size;
            unsafe {
                sys::igSetNextWindowSize(size.into(), Condition::Always as i32);
                sys::igSetNextWindowPos([display_size[0] / 2.0, display_size[1] / 2.0].into(), Condition::Always as i32, [0.5, 0.5].into());
            }
        }
        self.inner.build(f)
    }
}