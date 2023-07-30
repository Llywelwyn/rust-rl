use super::{
    gamesystem, gamesystem::attr_bonus, random_table::RandomTable, raws, Attribute, Attributes, HungerClock,
    HungerState, Map, Name, Player, Pool, Pools, Position, Rect, Renderable, SerializeMe, Skill, Skills, TileType,
    Viewshed,
};
use rltk::{RandomNumberGenerator, RGB};
use specs::prelude::*;
use specs::saveload::{MarkedBuilder, SimpleMarker};
use std::collections::HashMap;

/// Spawns the player and returns his/her entity object.
pub fn player(ecs: &mut World, player_x: i32, player_y: i32) -> Entity {
    let mut skills = Skills { skills: HashMap::new() };
    skills.skills.insert(Skill::Melee, 0);
    skills.skills.insert(Skill::Defence, 0);
    skills.skills.insert(Skill::Magic, 0);

    let mut rng = ecs.write_resource::<rltk::RandomNumberGenerator>();
    let str = gamesystem::roll_4d6(&mut rng);
    let dex = gamesystem::roll_4d6(&mut rng);
    let con = gamesystem::roll_4d6(&mut rng);
    let int = gamesystem::roll_4d6(&mut rng);
    let wis = gamesystem::roll_4d6(&mut rng);
    let cha = gamesystem::roll_4d6(&mut rng);
    std::mem::drop(rng);

    // d8 hit die - but always maxxed at level 1, so player doesn't have to roll.
    let player = ecs
        .create_entity()
        .with(Position { x: player_x, y: player_y })
        .with(Renderable {
            glyph: rltk::to_cp437('@'),
            fg: RGB::named(rltk::YELLOW),
            bg: RGB::named(rltk::BLACK),
            render_order: 0,
        })
        .with(Player {})
        .with(Viewshed { visible_tiles: Vec::new(), range: 12, dirty: true })
        .with(Name { name: "you".to_string(), plural: "you".to_string() })
        .with(HungerClock { state: HungerState::Satiated, duration: 50 })
        .with(Attributes {
            strength: Attribute { base: str, modifiers: 0, bonus: attr_bonus(str) },
            dexterity: Attribute { base: dex, modifiers: 0, bonus: attr_bonus(dex) },
            constitution: Attribute { base: con, modifiers: 0, bonus: attr_bonus(con) },
            intelligence: Attribute { base: int, modifiers: 0, bonus: attr_bonus(int) },
            wisdom: Attribute { base: wis, modifiers: 0, bonus: attr_bonus(wis) },
            charisma: Attribute { base: cha, modifiers: 0, bonus: attr_bonus(cha) },
        })
        .with(Pools {
            hit_points: Pool { current: 10 + attr_bonus(con), max: 10 + attr_bonus(con) },
            mana: Pool { current: 2 + attr_bonus(int), max: 2 + attr_bonus(int) },
            xp: 0,
            level: 1,
            bac: 10,
        })
        .with(skills)
        .marked::<SimpleMarker<SerializeMe>>()
        .build();

    raws::spawn_named_entity(
        &raws::RAWS.lock().unwrap(),
        ecs,
        "equip_dagger",
        raws::SpawnType::Equipped { by: player },
        0,
    );
    raws::spawn_named_entity(
        &raws::RAWS.lock().unwrap(),
        ecs,
        "food_apple",
        raws::SpawnType::Carried { by: player },
        0,
    );

    return player;
}

/// Fills a room with stuff!
pub fn spawn_room(map: &Map, rng: &mut RandomNumberGenerator, room: &Rect, spawn_list: &mut Vec<(usize, String)>) {
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

    spawn_region(map, rng, &possible_targets, spawn_list);
}

pub fn spawn_region(map: &Map, rng: &mut RandomNumberGenerator, area: &[usize], spawn_list: &mut Vec<(usize, String)>) {
    let mut spawn_points: HashMap<usize, String> = HashMap::new();
    let mut areas: Vec<usize> = Vec::from(area);
    let difficulty = map.difficulty;
    // If no area, log and return.
    if areas.len() == 0 {
        rltk::console::log("DEBUGINFO: No areas capable of spawning mobs!");
        return;
    }
    // Get num of each entity type.
    let num_mobs = match rng.roll_dice(1, 20) {
        1..=4 => 1, // 20% chance of spawning 1 mob.
        5 => 3,     // 5% chance of spawning 3 mobs.
        _ => 0,     // 75% chance of spawning 0
    };
    let num_items = match rng.roll_dice(1, 20) {
        1..=2 => 1, // 10% chance of spawning 1 item
        3 => 2,     // 5% chance of spawning 2 items
        4 => 3,     // 5% chance of spawning 3 items
        _ => 0,     // 80% chance of spawning nothing
    };
    let num_traps = match rng.roll_dice(1, 20) {
        1 => 1, // 5% chance of spawning 1 trap
        2 => 2, // 5% chance of spawning 2 traps
        _ => 0, // 85% chance of spawning nothing
    };
    // Roll on each table, getting an entity + spawn point
    for _i in 0..num_mobs {
        entity_from_table_to_spawn_list(rng, &mut areas, mob_table(difficulty), &mut spawn_points);
    }
    for _i in 0..num_traps {
        entity_from_table_to_spawn_list(rng, &mut areas, trap_table(difficulty), &mut spawn_points);
    }
    for _i in 0..num_items {
        let spawn_table = get_random_item_category(rng, difficulty);
        entity_from_table_to_spawn_list(rng, &mut areas, spawn_table, &mut spawn_points);
    }
    // Push entities and their spawn points to map's spawn list
    for spawn in spawn_points.iter() {
        spawn_list.push((*spawn.0, spawn.1.to_string()));
    }
}

fn entity_from_table_to_spawn_list(
    rng: &mut RandomNumberGenerator,
    possible_areas: &mut Vec<usize>,
    table: RandomTable,
    spawn_points: &mut HashMap<usize, String>,
) {
    if possible_areas.len() == 0 {
        return;
    }
    let array_idx =
        if possible_areas.len() == 1 { 0usize } else { (rng.roll_dice(1, possible_areas.len() as i32) - 1) as usize };
    let map_idx = possible_areas[array_idx];
    spawn_points.insert(map_idx, table.roll(rng));
    possible_areas.remove(array_idx);
}

/// Spawns a named entity (name in tuple.1) at the location in (tuple.0)
pub fn spawn_entity(ecs: &mut World, spawn: &(&usize, &String)) {
    let map = ecs.fetch::<Map>();
    let map_difficulty = map.difficulty;
    let width = map.width as usize;
    let x = (*spawn.0 % width) as i32;
    let y = (*spawn.0 / width) as i32;
    std::mem::drop(map);

    let spawn_result = raws::spawn_named_entity(
        &raws::RAWS.lock().unwrap(),
        ecs,
        &spawn.1,
        raws::SpawnType::AtPosition { x, y },
        map_difficulty,
    );
    if spawn_result.is_some() {
        return;
    }

    rltk::console::log(format!("WARNING: We don't know how to spawn [{}]!", spawn.1));
}

// 3 scrolls : 3 potions : 1 equipment : 1 wand?
fn item_category_table() -> RandomTable {
    return RandomTable::new().add("equipment", 20).add("food", 20).add("potion", 16).add("scroll", 16).add("wand", 4);
}

fn debug_table() -> RandomTable {
    return RandomTable::new().add("debug", 1);
}

fn get_random_item_category(rng: &mut RandomNumberGenerator, difficulty: i32) -> RandomTable {
    let item_category = item_category_table().roll(rng);
    match item_category.as_ref() {
        "equipment" => return equipment_table(difficulty),
        "food" => return food_table(difficulty),
        "potion" => return potion_table(difficulty),
        "scroll" => return scroll_table(difficulty),
        "wand" => return wand_table(difficulty),
        _ => return debug_table(),
    };
}

pub fn equipment_table(difficulty: i32) -> RandomTable {
    raws::table_by_name(&raws::RAWS.lock().unwrap(), "equipment", difficulty)
}

pub fn potion_table(difficulty: i32) -> RandomTable {
    raws::table_by_name(&raws::RAWS.lock().unwrap(), "potions", difficulty)
}

pub fn scroll_table(difficulty: i32) -> RandomTable {
    raws::table_by_name(&raws::RAWS.lock().unwrap(), "scrolls", difficulty)
}

pub fn wand_table(difficulty: i32) -> RandomTable {
    raws::table_by_name(&raws::RAWS.lock().unwrap(), "wands", difficulty)
}

pub fn food_table(difficulty: i32) -> RandomTable {
    raws::table_by_name(&raws::RAWS.lock().unwrap(), "food", difficulty)
}

pub fn mob_table(difficulty: i32) -> RandomTable {
    raws::table_by_name(&raws::RAWS.lock().unwrap(), "mobs", difficulty)
}

pub fn trap_table(difficulty: i32) -> RandomTable {
    raws::table_by_name(&raws::RAWS.lock().unwrap(), "traps", difficulty)
}
