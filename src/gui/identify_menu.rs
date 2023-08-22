use super::{
    get_max_inventory_width, item_colour_ecs, obfuscate_name_ecs, print_options, renderable_colour, ItemMenuResult,
    UniqueInventoryItem,
};
use crate::{
    gamelog, Beatitude, Entity, Equipped, InBackpack, Item, MasterDungeonMap, Name, ObfuscatedName, Renderable, State,
    BUC,
};
use rltk::prelude::*;
use specs::prelude::*;
use std::collections::BTreeMap;

/// Handles the Identify menu.
pub fn identify(gs: &mut State, ctx: &mut Rltk) -> (ItemMenuResult, Option<Entity>) {
    let player_entity = gs.ecs.fetch::<Entity>();
    let equipped = gs.ecs.read_storage::<Equipped>();
    let backpack = gs.ecs.read_storage::<InBackpack>();
    let entities = gs.ecs.entities();
    let items = gs.ecs.read_storage::<Item>();
    let obfuscated = gs.ecs.read_storage::<ObfuscatedName>();
    let dm = gs.ecs.fetch::<MasterDungeonMap>();
    let names = gs.ecs.read_storage::<Name>();
    let renderables = gs.ecs.read_storage::<Renderable>();

    let build_identify_iterator = || {
        (&entities, &items, &renderables, &names).join().filter(|(item_entity, _i, _r, n)| {
            // If not owned by the player, return false.
            let mut keep = false;
            if let Some(bp) = backpack.get(*item_entity) {
                if bp.owner == *player_entity {
                    keep = true;
                }
            }
            // If not equipped by the player, return false.
            if let Some(equip) = equipped.get(*item_entity) {
                if equip.owner == *player_entity {
                    keep = true;
                }
            }
            if !keep {
                return false;
            }
            // If not obfuscated, or already identified, return false.
            if !obfuscated.get(*item_entity).is_some() || dm.identified_items.contains(&n.name) {
                return false;
            }
            return true;
        })
    };

    // Build list of items to display
    let count = build_identify_iterator().count();
    // If no items, return nothing, wasting the scroll.
    if count == 0 {
        gamelog::Logger::new().append("You've got nothing to identify! Know-it-all.").log();
        return (ItemMenuResult::Cancel, None);
    }
    // If only one item, return it.
    if count == 1 {
        let item = build_identify_iterator().nth(0).unwrap().0;
        gamelog::Logger::new()
            .append("You identify the")
            .colour(item_colour_ecs(&gs.ecs, item))
            .append_n(obfuscate_name_ecs(&gs.ecs, item).0)
            .colour(WHITE)
            .append("!")
            .log();
        return (ItemMenuResult::Selected, Some(build_identify_iterator().nth(0).unwrap().0));
    }
    let mut player_inventory: BTreeMap<UniqueInventoryItem, i32> = BTreeMap::new();
    let mut inventory_ids: BTreeMap<String, Entity> = BTreeMap::new();
    for (entity, _i, renderable, name) in build_identify_iterator() {
        let (singular, plural) = obfuscate_name_ecs(&gs.ecs, entity);
        player_inventory
            .entry(UniqueInventoryItem {
                display_name: super::DisplayName { singular: singular.clone(), plural: plural.clone() },
                rgb: item_colour_ecs(&gs.ecs, entity),
                renderables: renderable_colour(&renderables, entity),
                glyph: renderable.glyph,
                name: name.name.clone(),
            })
            .and_modify(|count| *count += 1)
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
        "Identify which item? [aA-zZ][Esc.]",
    );
    ctx.draw_box(x, y, width + 2, count + 1, RGB::named(WHITE), RGB::named(BLACK));
    print_options(player_inventory, x + 1, y + 1, ctx);
    // Input
    match ctx.key {
        None => (ItemMenuResult::NoResponse, None),
        Some(key) => match key {
            VirtualKeyCode::Escape => (ItemMenuResult::Cancel, None),
            _ => {
                let selection = rltk::letter_to_option(key);
                if selection > -1 && selection < count as i32 {
                    let item = inventory_ids.iter().nth(selection as usize).unwrap().1;
                    gamelog::Logger::new()
                        .append("You identify the")
                        .colour(item_colour_ecs(&gs.ecs, *item))
                        .append_n(obfuscate_name_ecs(&gs.ecs, *item).0)
                        .colour(WHITE)
                        .append("!")
                        .log();
                    return (ItemMenuResult::Selected, Some(*item));
                }
                (ItemMenuResult::NoResponse, None)
            }
        },
    }
}
