use notan::prelude::*;
use notan::draw::{ Draw, Font, DrawTextSection };
use specs::prelude::*;
use super::TILESIZE;
use crate::{ Fonts, camera::get_offset };
use super::{ items, Filter, print_options, ItemType, FONTSIZE };

pub enum Location {
    All,
    Backpack,
    Equipped,
}

pub fn all_itemtypes() -> Vec<ItemType> {
    vec![
        ItemType::Weapon,
        ItemType::Armour,
        ItemType::Comestible,
        ItemType::Potion,
        ItemType::Scroll,
        ItemType::Spellbook,
        ItemType::Wand,
        ItemType::Amulet,
        ItemType::Ring
    ]
}

pub fn draw_items(
    ecs: &World,
    draw: &mut Draw,
    font: &Fonts,
    x: f32,
    y: f32,
    loc: Location,
    itemtypes: Option<Vec<ItemType>>
) {
    let mut y = y;
    if let Some(itemtypes) = itemtypes {
        for itemtype in itemtypes {
            let filter = match loc {
                Location::All => Filter::All(Some(itemtype)),
                Location::Backpack => Filter::Backpack(Some(itemtype)),
                Location::Equipped => Filter::Equipped,
            };
            let inv = items(ecs, filter);
            if inv.is_empty() {
                continue;
            }
            draw.text(&font.b(), itemtype.string())
                .position(x, y)
                .color(Color::WHITE)
                .size(FONTSIZE);
            y += TILESIZE.x;
            y = print_options(ecs, draw, font, &inv, x, y) + TILESIZE.x;
        }
    } else {
        let filter = match loc {
            Location::All => Filter::All(None),
            Location::Backpack => Filter::Backpack(None),
            Location::Equipped => Filter::Equipped,
        };
        let inv = items(ecs, filter);
        if inv.is_empty() {
            return;
        }
        y = print_options(ecs, draw, font, &inv, x, y) + TILESIZE.x;
    }
}

pub fn draw_all_items(ecs: &World, draw: &mut Draw, font: &Fonts, x: f32, y: f32) {
    draw_items(ecs, draw, font, x, y, Location::All, Some(all_itemtypes()));
}

pub fn draw_backpack_items(ecs: &World, draw: &mut Draw, font: &Fonts, x: f32, y: f32) {
    draw_items(ecs, draw, font, x, y, Location::Backpack, Some(all_itemtypes()));
}
