use crate::renderer::{Textures, Texture};

macro_rules! load_internal_texture {
    {$textures:ident, $path:tt} => { {
        let image = image::load_from_memory(include_bytes!($path)).expect("Failed to load internal texture!");
        $textures.create_texture(&image).expect("Failed to create internal texture!")
    } }
}

macro_rules! gui_resources {
    { $($name:ident: $path:tt),* } => {
        pub struct GuiResources {
            $(pub $name: Texture),*
        }

        impl GuiResources {
            pub fn load<T: Textures + ?Sized>(textures: &mut T) -> GuiResources {
                GuiResources {
                    $($name: load_internal_texture!(textures, $path)),*
                }
            }
        }
    }
}

gui_resources! {
    missing_image: "../resources/missing.png",
    blue_plus: "../resources/blue_plus.png",
    hidden: "../resources/hidden.png",
    not_hidden: "../resources/not_hidden.png",
    unavailable: "../resources/unavailable.png",
    edited: "../resources/edited.png",
    unedited: "../resources/unedited.png",
    filter: "../resources/filter.png",
    reload_small: "../resources/reload_small.png",
    blue_x: "../resources/blue_x.png"
    // reload: "../resources/reload.png"
}