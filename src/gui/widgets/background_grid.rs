use gui::prelude::*;
use widgets::background_card::*;

pub struct BackgroundGrid<'a> {
    id: &'a ImStr,
    entries: Vec<Option<CardOriginalInfo>>,
}

impl<'a> BackgroundGrid<'a> {
    pub fn draw(self, ui: &Ui) {
        if self.entries.len() == 0 { return; }
        let columns = usize::min(self.entries.len(), ui.io().display_size[0] / 300.0);
        ChildWindow::new(id).border(true).build(ui, || {
            for entry in self.entries {
                unimplemented!()
            }
        });
    }
}