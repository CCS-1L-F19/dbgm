use crate::gui::prelude::*;
use crate::background::CropRegion;

#[must_use]
pub struct CroppableImage {
    texture: Texture,
    size: Vec2,
}

impl CroppableImage {
    pub fn new(texture: Texture, size: impl Into<Vec2>) -> Self {
        CroppableImage {
            texture,
            size: size.into(),
        }
    }

    pub fn build(self, ui: &Ui, mut region: CropRegion) {
        let base = vec2![1.0, 1.0] + ui.cursor_pos() + ui.window_pos();
        Image::new(self.texture.id, self.size.into()).border_col(ui.style_color(StyleColor::Border)).build(ui);
        if ui.is_item_hovered() {
            if ui.is_mouse_down(MouseButton::Left) {
                *region.center = self.window_coord_to_tex(base, ui.io().mouse_pos);
            }
            println!("{}", ui.io().mouse_wheel);
            region.clip();
        }
        let top_left = self.tex_coord_to_window(base, region.top_left());
        let bottom_right = self.tex_coord_to_window(base, region.bottom_right());
        let center = (top_left + bottom_right) / 2.0;
        let draw_list = ui.get_window_draw_list();
        draw_list.add_rect(top_left.into(), bottom_right.into(), [1.0, 0.0, 0.0]).build();
        draw_list.add_circle(center.into(), 10.0, [0.8, 0.8, 0.8]).build();
    }

    fn tex_coord_to_window(&self, base: impl Into<Vec2>, point: impl Into<Vec2>) -> Vec2 {
        let (base, point) = (base.into(), point.into());
        base + point.scale(self.size - [IMAGE_BORDER_WIDTH, IMAGE_BORDER_WIDTH]).scale_inv(self.texture.size)
    }

    fn window_coord_to_tex(&self, base: impl Into<Vec2>, point: impl Into<Vec2>) -> Vec2 {
        let (base, point) = (base.into(), point.into());
        (point - base).scale(self.texture.size).scale_inv(self.size - [IMAGE_BORDER_WIDTH, IMAGE_BORDER_WIDTH])
    }
}