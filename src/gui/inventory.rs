use notan::prelude::*;
use notan::draw::{ Draw, Font };
use specs::prelude::*;
use super::TILESIZE;

pub fn draw_inventory(ecs: &World, draw: &mut Draw, font: &Font) {
    let inv = super::get_player_inventory(ecs);
    let offsets = crate::camera::get_offset();
    super::print_options(
        draw,
        font,
        &inv,
        (offsets.x as f32) * TILESIZE,
        (offsets.y as f32) * TILESIZE
    );
}
