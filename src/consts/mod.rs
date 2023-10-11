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

pub struct Spritesize {
    pub x: f32,
    pub y: f32,
    pub sprite_x: f32,
    pub sprite_y: f32,
}

pub const TILESIZE: Spritesize = Spritesize {
    x: 16.0,
    y: 16.0,
    sprite_x: 16.0 * ZOOM_FACTOR,
    sprite_y: 24.0 * ZOOM_FACTOR,
};
pub const ZOOM_FACTOR: f32 = 2.0;
pub const FONTSIZE: f32 = 16.0;

pub const DISPLAYWIDTH: u32 = 120;
pub const DISPLAYHEIGHT: u32 = 67;
