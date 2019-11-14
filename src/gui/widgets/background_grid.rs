use crate::background::BackgroundSet;

use crate::gui::prelude::*;

use widgets::background_card::*;

pub struct BackgroundGrid<'a> {
    pub id: &'a ImStr,
    pub entries: Vec<(usize, Option<CardOriginalInfo>)>,
    pub card_width: f32,
    pub max_size: [f32; 2],
}

impl<'a> BackgroundGrid<'a> {    
    fn dimensions_and_size(&self, ui: &Ui) -> ([usize; 2], [f32; 2]) {
        let max_width = if self.max_size[0] != 0.0 { self.max_size[0] } else { ui.content_region_avail()[0] };
        
        let Style { item_spacing, window_padding, window_border_size, scrollbar_size, .. } = ui.clone_style();
        let card_size = BackgroundCard::size(ui, self.card_width);
        let extra = [2.0 * (window_padding[0] + window_border_size), 2.0 * (window_padding[1] + window_border_size)];

        let num_cards = self.entries.len();

        let calc_columns = |available| {
            usize::min(num_cards, f32::min(1.0, (available + item_spacing[0]) / (card_size[0] + item_spacing[0])).floor() as usize)
        };

        let grid_size = |dim, len| len * (card_size[dim] + item_spacing[dim]) - item_spacing[dim] + extra[dim];
        
        let (columns, rows, scrollbar) = {
            let mut columns = calc_columns(max_width - extra[0]);
            let mut rows = (num_cards as f32 / columns as f32).ceil();
            let scrollbar = self.max_size[1] > 0.0 && grid_size(1, rows) > self.max_size[1];
            if scrollbar {
                columns = calc_columns(max_width - extra[0] - scrollbar_size);
                rows = (num_cards as f32 / columns as f32).ceil();
            }
            (columns, rows as usize, scrollbar)
        };

        let size = if !scrollbar { 
            [grid_size(0, columns as f32), grid_size(1, rows as f32)] 
        } else {
            [grid_size(0, columns as f32) + scrollbar_size, self.max_size[1]]
        };

        ([columns, rows], size)
    }

    pub fn dimensions(&self, ui: &Ui) -> ([usize; 2]) {
        self.dimensions_and_size(ui).0
    }

    pub fn size(&self, ui: &Ui) -> [f32; 2] {
        self.dimensions_and_size(ui).1
    }

    pub fn draw<T: ?Sized>(self, set: &mut BackgroundSet, frame: Frame<T>) {
        if self.entries.len() == 0 { return; }
        let Frame { ui, resources, .. } = frame;

        let (dimensions, size) = self.dimensions_and_size(ui);

        ChildWindow::new(self.id).border(true).size(size).build(ui, || {
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
                if (i + 1) % dimensions[0] != 0 { ui.same_line(0.0); }
            }
        });
    }
}