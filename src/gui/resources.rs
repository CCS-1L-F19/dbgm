use imgui::TextureId;
use super::utils::Textures;

macro_rules! load_internal_texture {
    {$textures:ident, $path:tt} => { {
        let image = image::load_from_memory(include_bytes!($path)).expect("Failed to load internal texture!");
        $textures.create_texture(&image).expect("Failed to create internal texture!")
    } }
}

macro_rules! gui_resources {
    { $($name:ident: $path:tt),* } => {
        pub struct GuiResources {
            $(pub $name: TextureId),*
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
    blue_plus: "../resources/blue_plus.png"
}