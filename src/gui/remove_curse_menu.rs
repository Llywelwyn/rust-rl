use super::{
    get_max_inventory_width,
    item_colour_ecs,
    obfuscate_name_ecs,
    print_options,
    renderable_colour,
    ItemMenuResult,
    UniqueInventoryItem,
};
use crate::{ gamelog, Beatitude, Entity, Equipped, InBackpack, Item, Name, Renderable, State, BUC };
use rltk::prelude::*;
use specs::prelude::*;
use std::collections::BTreeMap;

/// Handles the Remove Curse menu.
pub fn remove_curse(gs: &mut State, ctx: &mut Rltk) -> (ItemMenuResult, Option<Entity>) {
    let player_entity = gs.ecs.fetch::<Entity>();
    let equipped = gs.ecs.read_storage::<Equipped>();
    let backpack = gs.ecs.read_storage::<InBackpack>();
    let entities = gs.ecs.entities();
    let items = gs.ecs.read_storage::<Item>();
    let beatitudes = gs.ecs.read_storage::<Beatitude>();
    let names = gs.ecs.read_storage::<Name>();
    let renderables = gs.ecs.read_storage::<Renderable>();

    let build_cursed_iterator = || {
        (&entities, &items, &beatitudes, &renderables, &names).join().filter(|(item_entity, _i, b, _r, _n)| {
            // Set all items to FALSE initially.
            let mut keep = false;
            // If found in the player's backpack, set to TRUE
            if let Some(bp) = backpack.get(*item_entity) {
                if bp.owner == *player_entity {
                    keep = true;
                }
            }
            // If found in the player's equipslot, set to TRUE
            if let Some(equip) = equipped.get(*item_entity) {
                if equip.owner == *player_entity {
                    keep = true;
                }
            }
            // If it's not OUR item, RETURN FALSE.
            if !keep {
                return false;
            }
            // If it's identified as noncursed, RETURN FALSE.
            if b.known && b.buc != BUC::Cursed {
                return false;
            }
            // Otherwise, return: returns any items that are unidentified,
            // or identified as being cursed.
            return true;
        })
    };

    // Build list of items to display
    let count = build_cursed_iterator().count();
    // If no items, return nothing, wasting the scroll.
    if count == 0 {
        gamelog::Logger::new().append("You've got nothing to decurse! What a waste.").log();
        return (ItemMenuResult::Cancel, None);
    }
    // If only one item, return it.
    if count == 1 {
        let item = build_cursed_iterator().nth(0).unwrap().0;
        gamelog::Logger
            ::new()
            .append("You decurse the")
            .colour(item_colour_ecs(&gs.ecs, item))
            .append_n(obfuscate_name_ecs(&gs.ecs, item).0)
            .colour(WHITE)
            .append("!")
            .log();
        return (ItemMenuResult::Selected, Some(item));
    }
    let mut player_inventory: BTreeMap<UniqueInventoryItem, i32> = BTreeMap::new();
    let mut inventory_ids: BTreeMap<String, Entity> = BTreeMap::new();
    for (entity, _i, _b, renderable, name) in build_cursed_iterator() {
        let (singular, plural) = obfuscate_name_ecs(&gs.ecs, entity);
        player_inventory
            .entry(UniqueInventoryItem {
                display_name: super::DisplayName { singular: singular.clone(), plural: plural.clone() },
                rgb: item_colour_ecs(&gs.ecs, entity),
                renderables: renderable_colour(&renderables, entity),
                glyph: renderable.glyph,
                name: name.name.clone(),
            })
            .and_modify(|count| {
                *count += 1;
            })
            .or_insert(1);
        inventory_ids.entry(singular).or_insert(entity);
    }
    // Get display args
    let width = get_max_inventory_width(&player_inventory);
    let (_, _, _, _, x_offset, y_offset) = crate::camera::get_screen_bounds(&gs.ecs, ctx);
    let (x, y) = (x_offset + 1, y_offset + 3);
    // Draw menu
    ctx.print_color(
        1 + x_offset,
        1 + y_offset,
        RGB::named(rltk::WHITE),
        RGB::named(rltk::BLACK),
        "Decurse which item? [aA-zZ][Esc.]"
    );
    ctx.draw_box(x, y, width + 2, count + 1, RGB::named(WHITE), RGB::named(BLACK));
    print_options(player_inventory, x + 1, y + 1, ctx);
    // Input
    match ctx.key {
        None => (ItemMenuResult::NoResponse, None),
        Some(key) =>
            match key {
                VirtualKeyCode::Escape => (ItemMenuResult::Cancel, None),
                _ => {
                    let selection = rltk::letter_to_option(key);
                    if selection > -1 && selection < (count as i32) {
                        let item = inventory_ids
                            .iter()
                            .nth(selection as usize)
                            .unwrap().1;
                        gamelog::Logger
                            ::new()
                            .append("You decurse the")
                            .colour(item_colour_ecs(&gs.ecs, *item))
                            .append_n(obfuscate_name_ecs(&gs.ecs, *item).0)
                            .colour(WHITE)
                            .append("!")
                            .log();
                        return (ItemMenuResult::Selected, Some(*item));
                    }
                    (ItemMenuResult::NoResponse, None)
                }
            }
    }
}
