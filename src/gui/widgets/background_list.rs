type BackgroundEntry = (usize, Option<CardOriginalInfo>);

struct BackgroundList {
    pub show_edited: bool,
    pub show_excluded: bool,
}

impl BackgroundList {
    fn generate_background_entries<T: Textures + ?Sized>(&self, set: &mut BackgroundSet, image_cache: &mut ImageCache, textures: &mut T) -> Vec<Vec<BackgroundEntry>> {
        
    }
}