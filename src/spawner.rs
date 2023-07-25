use super::{
    random_table::RandomTable, raws, Attribute, Attributes, CombatStats, HungerClock, HungerState, Map, Name, Player,
    Position, Rect, Renderable, SerializeMe, TileType, Viewshed,
};
use rltk::{RandomNumberGenerator, RGB};
use specs::prelude::*;
use specs::saveload::{MarkedBuilder, SimpleMarker};
use std::collections::HashMap;

/// Spawns the player and returns his/her entity object.
pub fn player(ecs: &mut World, player_x: i32, player_y: i32) -> Entity {
    // d8 hit die - but always maxxed at level 1, so player doesn't have to roll.
    ecs.create_entity()
        .with(Position { x: player_x, y: player_y })
        .with(Renderable {
            glyph: rltk::to_cp437('@'),
            fg: RGB::named(rltk::YELLOW),
            bg: RGB::named(rltk::BLACK),
            render_order: 0,
        })
        .with(Player {})
        .with(Viewshed { visible_tiles: Vec::new(), range: 12, dirty: true })
        .with(Name { name: "wanderer".to_string(), plural: "wanderers".to_string() })
        .with(CombatStats { max_hp: 12, hp: 12, defence: 0, power: 4 })
        .with(HungerClock { state: HungerState::Satiated, duration: 50 })
        .with(Attributes {
            strength: Attribute { base: 10, modifiers: 0, bonus: 0 },
            dexterity: Attribute { base: 10, modifiers: 0, bonus: 0 },
            constitution: Attribute { base: 10, modifiers: 0, bonus: 0 },
            intelligence: Attribute { base: 10, modifiers: 0, bonus: 0 },
            wisdom: Attribute { base: 10, modifiers: 0, bonus: 0 },
            charisma: Attribute { base: 10, modifiers: 0, bonus: 0 },
        })
        .marked::<SimpleMarker<SerializeMe>>()
        .build()
}

// Consts
const MAX_ENTITIES: i32 = 4;

/// Fills a room with stuff!
pub fn spawn_room(
    map: &Map,
    rng: &mut RandomNumberGenerator,
    room: &Rect,
    map_depth: i32,
    spawn_list: &mut Vec<(usize, String)>,
) {
    let mut possible_targets: Vec<usize> = Vec::new();
    {
        // Borrow scope - to keep access to the map separated
        for y in room.y1 + 1..room.y2 {
            for x in room.x1 + 1..room.x2 {
                let idx = map.xy_idx(x, y);
                if map.tiles[idx] == TileType::Floor {
                    possible_targets.push(idx);
                }
            }
        }
    }

    spawn_region(map, rng, &possible_targets, map_depth, spawn_list);
}

pub fn spawn_region(
    _map: &Map,
    rng: &mut RandomNumberGenerator,
    area: &[usize],
    map_depth: i32,
    spawn_list: &mut Vec<(usize, String)>,
) {
    let mut spawn_points: HashMap<usize, String> = HashMap::new();
    let mut areas: Vec<usize> = Vec::from(area);

    let num_spawns = i32::min(areas.len() as i32, rng.roll_dice(1, MAX_ENTITIES + 2) - 2);
    if num_spawns <= 0 {
        return;
    }

    for _i in 0..num_spawns {
        let category = category_table().roll(rng);
        let spawn_table;
        match category.as_ref() {
            "mob" => spawn_table = mob_table(map_depth),
            "item" => {
                let item_category = item_category_table().roll(rng);
                match item_category.as_ref() {
                    "equipment" => spawn_table = equipment_table(map_depth),
                    "potion" => spawn_table = potion_table(map_depth),
                    "scroll" => spawn_table = scroll_table(map_depth),
                    "wand" => spawn_table = wand_table(map_depth),
                    _ => spawn_table = debug_table(),
                }
            }
            "food" => spawn_table = food_table(map_depth),
            "trap" => spawn_table = trap_table(map_depth),
            _ => spawn_table = debug_table(),
        }
        let array_idx = if areas.len() == 1 { 0usize } else { (rng.roll_dice(1, areas.len() as i32) - 1) as usize };
        let map_idx = areas[array_idx];
        spawn_points.insert(map_idx, spawn_table.roll(rng));
        areas.remove(array_idx);
    }

    for spawn in spawn_points.iter() {
        spawn_list.push((*spawn.0, spawn.1.to_string()));
    }
}

/// Spawns a named entity (name in tuple.1) at the location in (tuple.0)
pub fn spawn_entity(ecs: &mut World, spawn: &(&usize, &String)) {
    let map = ecs.fetch::<Map>();
    let width = map.width as usize;
    let x = (*spawn.0 % width) as i32;
    let y = (*spawn.0 / width) as i32;
    std::mem::drop(map);

    let spawn_result = raws::spawn_named_entity(
        &raws::RAWS.lock().unwrap(),
        ecs.create_entity(),
        &spawn.1,
        raws::SpawnType::AtPosition { x, y },
        &mut rltk::RandomNumberGenerator::new(),
    );
    if spawn_result.is_some() {
        return;
    }

    rltk::console::log(format!("WARNING: We don't know how to spawn [{}]!", spawn.1));
}

// 12 mobs : 6 items : 2 food : 1 trap
fn category_table() -> RandomTable {
    return RandomTable::new().add("mob", 12).add("item", 6).add("food", 2).add("trap", 1);
}

// 3 scrolls : 3 potions : 1 equipment : 1 wand?
fn item_category_table() -> RandomTable {
    return RandomTable::new().add("equipment", 1).add("potion", 3).add("scroll", 3).add("wand", 1);
}

fn debug_table() -> RandomTable {
    return RandomTable::new().add("debug", 1);
}

pub fn equipment_table(map_depth: i32) -> RandomTable {
    raws::table_by_name(&raws::RAWS.lock().unwrap(), "equipment", map_depth)
}

pub fn potion_table(map_depth: i32) -> RandomTable {
    raws::table_by_name(&raws::RAWS.lock().unwrap(), "potions", map_depth)
}

pub fn scroll_table(map_depth: i32) -> RandomTable {
    raws::table_by_name(&raws::RAWS.lock().unwrap(), "scrolls", map_depth)
}

pub fn wand_table(map_depth: i32) -> RandomTable {
    raws::table_by_name(&raws::RAWS.lock().unwrap(), "wands", map_depth)
}

pub fn food_table(map_depth: i32) -> RandomTable {
    raws::table_by_name(&raws::RAWS.lock().unwrap(), "food", map_depth)
}

pub fn mob_table(map_depth: i32) -> RandomTable {
    raws::table_by_name(&raws::RAWS.lock().unwrap(), "mobs", map_depth)
}

pub fn trap_table(map_depth: i32) -> RandomTable {
    raws::table_by_name(&raws::RAWS.lock().unwrap(), "traps", map_depth)
}
