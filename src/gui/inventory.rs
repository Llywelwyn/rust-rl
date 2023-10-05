use notan::prelude::*;
use notan::draw::{ Draw, Font, DrawTextSection };
use specs::prelude::*;
use super::TILESIZE;
use crate::{ Fonts, camera::get_offset };
use super::{ items, Filter, print_options, ItemType };

pub fn draw_inventory(ecs: &World, draw: &mut Draw, font: &Fonts, x: i32, y: i32) {
    let inv = items(ecs, Filter::Backpack);
    let offsets = crate::camera::get_offset();
    print_options(
        ecs,
        draw,
        font,
        &inv,
        ((x as f32) + (offsets.x as f32)) * TILESIZE,
        ((y as f32) + (offsets.y as f32)) * TILESIZE
    );
}

pub fn draw_all(ecs: &World, draw: &mut Draw, font: &Fonts, x: f32, y: f32) {
    let mut y = y;

    let itemtypes = vec![
        ItemType::Weapon,
        ItemType::Armour,
        ItemType::Comestible,
        ItemType::Potion,
        ItemType::Scroll,
        ItemType::Spellbook,
        ItemType::Wand,
        ItemType::Amulet,
        ItemType::Ring
    ];

    for itemtype in itemtypes {
        let inv = items(ecs, Filter::Category(itemtype));
        if inv.is_empty() {
            continue;
        }
        draw.text(&font.b(), itemtype.string()).position(x, y).color(Color::WHITE);
        y += TILESIZE;
        y = print_options(ecs, draw, font, &inv, x, y) + TILESIZE;
    }
}
