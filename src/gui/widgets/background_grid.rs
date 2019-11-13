use crate::background::BackgroundSet;

use crate::gui::prelude::*;

use widgets::background_card::*;

pub struct BackgroundGrid<'a> {
    pub id: &'a ImStr,
    pub entries: Vec<(usize, Option<CardOriginalInfo>)>,
    pub card_width: f32,
    pub max_dimensions: [usize; 2],
}

impl<'a> BackgroundGrid<'a> {
    pub fn draw<T: ?Sized>(self, set: &mut BackgroundSet, frame: Frame<T>) {
        let Frame { ui, resources, .. } = frame;
        let num_backgrounds = self.entries.len();
        if num_backgrounds == 0 { return; }

        let columns = usize::min(num_backgrounds, self.max_dimensions[1]);
        let rows = usize::min(f32::ceil(num_backgrounds as f32 / columns as f32) as usize, self.max_dimensions[0]);

        let card_size = BackgroundCard::size(ui, self.card_width);
        let spacing = ui.clone_style().item_spacing[1];
        let height = rows as f32 * (card_size[1] + spacing) - spacing;

        ChildWindow::new(self.id).border(true).content_size([0.0, height]).build(ui, || {
            for (i, (id, original)) in self.entries.into_iter().enumerate() {
                let card = BackgroundCard {
                    id: &im_str!("Background{}", id),
                    resources: &resources,
                    background: &set.backgrounds[id],
                    original,
                    editable: false,
                    width: self.card_width,
                };
                card.draw(ui);
                if (i + 1) % columns != 0 { ui.same_line(0.0); }
            }
        });
    }
}