#[macro_use] mod utils;
mod bglist;
mod resources;
mod modals;
mod widgets;
mod state;
mod editor;

pub mod draw;

mod prelude {
    pub(in super) use imgui::*;
    pub(in super) use crate::math::*;
    pub(in super) use crate::renderer::{Texture, Textures};
    pub(in super) use super::{modals, widgets, utils};
    pub(in super) use super::resources::GuiResources;
    pub(in super) use super::state::{GuiState, Operation, ActiveSet};
    pub(in super) use super::utils::{UiExt, ImageCache, AUTO_SIZE, IMAGE_BORDER_WIDTH};
    pub(in super) use super::widgets::PopupModal2 as PopupModal;
    pub(in super) use super::draw::Frame;
}

pub use draw::{draw_state, Frame};
pub use state::GuiState;
pub use resources::GuiResources;