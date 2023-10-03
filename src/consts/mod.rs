pub mod entity;
pub mod visuals;
pub mod messages;
pub mod char_create;
pub mod events;
pub mod ids;
pub mod names;
pub mod sprites;

pub mod prelude {
    pub use super::visuals::{ TILE_LAYER, ENTITY_LAYER, TEXT_LAYER, HP_BAR_LAYER };
    pub use super::visuals::{ VIEWPORT_H, VIEWPORT_W };
}

pub const TILESIZE: f32 = 16.0;
pub const FONTSIZE: f32 = 16.0;
pub const DISPLAYWIDTH: u32 = 100;
pub const DISPLAYHEIGHT: u32 = 56;
