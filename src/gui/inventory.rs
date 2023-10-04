use notan::prelude::*;
use notan::draw::{ Draw, Font };
use specs::prelude::*;
use super::TILESIZE;
use crate::Fonts;

pub fn draw_inventory(ecs: &World, draw: &mut Draw, font: &Fonts, x: i32, y: i32) {
    let inv = super::get_player_inventory(ecs);
    let offsets = crate::camera::get_offset();
    super::print_options(
        draw,
        font,
        &inv,
        ((x as f32) + (offsets.x as f32)) * TILESIZE,
        ((y as f32) + (offsets.y as f32)) * TILESIZE
    );
}
