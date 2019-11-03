use crate::gui::prelude::*;

/// Builder for an image button widget
#[derive(Copy, Clone, Debug)]
#[must_use]
pub struct ImageDropdown<'a> {
    id: &'a ImStr,
    size: [f32; 2],
    button: ImageButton
}

impl<'a> ImageDropdown<'a> {
    /// Creates a new image button builder with the given texture and size
    pub fn new(id: &ImStr, texture_id: TextureId, size: [f32; 2]) -> ImageDropdown {
        ImageDropdown {
            id: id,
            size: size,
            button: ImageButton::new(texture_id, size),
        }
    }

    fn map<T>(self, f: fn(ImageButton, T) -> ImageButton, param: T) -> ImageDropdown<'a> {
        ImageDropdown { id: self.id, size: self.size, button: f(self.button, param) }
    }

    /// Sets the image button size
    pub fn size(mut self, size: [f32; 2]) -> Self {
        self.map(ImageButton::size, size)
    }
    /// Sets uv0 (default `[0.0, 0.0]`)
    pub fn uv0(mut self, uv0: [f32; 2]) -> Self {
        self.map(ImageButton::uv0, uv0)
    }
    /// Sets uv1 (default `[1.0, 1.0]`)
    pub fn uv1(mut self, uv1: [f32; 2]) -> Self {
        self.map(ImageButton::uv1, uv1)
    }
    /// Sets the frame padding (default: uses frame padding from style).
    ///
    /// - `< 0`: uses frame padding from style (default)
    /// - `= 0`: no framing
    /// - `> 0`: set framing size
    pub fn frame_padding(mut self, frame_padding: i32) -> Self {
        self.map(ImageButton::frame_padding, frame_padding)
    }
    /// Sets the background color (default: no background color)
    pub fn background_col(mut self, bg_col: [f32; 4]) -> Self {
        self.map(ImageButton::background_col, bg_col)
    }
    /// Sets the tint color (default: no tint color)
    pub fn tint_col(mut self, tint_col: [f32; 4]) -> Self {
        self.map(ImageButton::tint_col, tint_col)
    }
    /// Builds the image button
    pub fn build(self, ui: &Ui, dropdown: impl FnOnce()) {
        let bottom_right = [
            ui.window_pos()[0] + ui.cursor_pos()[0] + self.size[0],
            ui.window_pos()[1] + ui.cursor_pos()[1] + self.size[1]
        ];
        if self.button.build(ui) {
            ui.open_popup(self.id);
        }
        unsafe {
            imgui::sys::igSetNextWindowPos(bottom_right.into(), Condition::Always as i32, [1.0, 0.0].into());
        }
        ui.popup(self.id, dropdown);
    }
}