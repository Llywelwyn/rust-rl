use super::{
    ai::NORMAL_SPEED, gamesystem, gamesystem::attr_bonus, random_table::RandomTable, raws, Attribute, Attributes,
    Clock, Energy, EquipmentChanged, Faction, HungerClock, HungerState, Map, Name, Player, Pool, Pools, Position, Rect,
    Renderable, SerializeMe, Skill, Skills, TileType, Viewshed,
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

    // We only create the player once, so create the Clock here for counting turns too.
    ecs.create_entity().with(Clock {}).with(Energy { current: 0, speed: NORMAL_SPEED }).build();
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
        .with(Faction { name: "player".to_string() })
        .with(Viewshed { visible_tiles: Vec::new(), range: 12, dirty: true })
        .with(Name { name: "you".to_string(), plural: "you".to_string() })
        .with(HungerClock { state: HungerState::Satiated, duration: 200 })
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
            weight: 0.0,
            god: false,
        })
        .with(EquipmentChanged {})
        .with(skills)
        .with(Energy { current: 0, speed: NORMAL_SPEED })
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
        "food_rations",
        raws::SpawnType::Carried { by: player },
        0,
    );
    raws::spawn_named_entity(
        &raws::RAWS.lock().unwrap(),
        ecs,
        "food_apple",
        raws::SpawnType::Carried { by: player },
        0,
    );
    raws::spawn_named_entity(
        &raws::RAWS.lock().unwrap(),
        ecs,
        "food_apple",
        raws::SpawnType::Carried { by: player },
        0,
    );
    raws::spawn_named_entity(
        &raws::RAWS.lock().unwrap(),
        ecs,
        "scroll_mass_health",
        raws::SpawnType::Carried { by: player },
        0,
    );
    raws::spawn_named_entity(
        &raws::RAWS.lock().unwrap(),
        ecs,
        "wand_fireball",
        raws::SpawnType::Carried { by: player },
        0,
    );
    raws::spawn_named_entity(
        &raws::RAWS.lock().unwrap(),
        ecs,
        "wand_fireball",
        raws::SpawnType::Carried { by: player },
        0,
    );
    raws::spawn_named_entity(
        &raws::RAWS.lock().unwrap(),
        ecs,
        "wand_confusion",
        raws::SpawnType::Carried { by: player },
        0,
    );

    return player;
}

/// Fills a room with stuff!
pub fn spawn_room(
    map: &Map,
    rng: &mut RandomNumberGenerator,
    room: &Rect,
    spawn_list: &mut Vec<(usize, String)>,
    player_level: i32,
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

    spawn_region(map, rng, &possible_targets, spawn_list, player_level);
}

pub fn spawn_region(
    map: &Map,
    rng: &mut RandomNumberGenerator,
    area: &[usize],
    spawn_list: &mut Vec<(usize, String)>,
    player_level: i32,
) {
    let mut spawn_points: HashMap<usize, String> = HashMap::new();
    let mut areas: Vec<usize> = Vec::from(area);
    let difficulty = (map.difficulty + player_level) / 2;
    // If no area, log and return.
    if areas.len() == 0 {
        rltk::console::log("DEBUGINFO: No areas capable of spawning mobs!");
        return;
    }
    // Get num of each entity type.
    let spawn_mob: bool = rng.roll_dice(1, 3) == 1;
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
    if spawn_mob {
        let key = mob_table(difficulty).roll(rng);
        let spawn_type = raws::get_mob_spawn_type(&raws::RAWS.lock().unwrap(), &key);
        let roll = raws::get_mob_spawn_amount(rng, &spawn_type, player_level);
        for _i in 0..roll {
            entity_to_spawn_list(rng, &mut areas, key.clone(), &mut spawn_points);
        }
    }
    for _i in 0..num_traps {
        let key = trap_table(difficulty).roll(rng);
        entity_to_spawn_list(rng, &mut areas, key, &mut spawn_points);
    }
    for _i in 0..num_items {
        // Player level isn't taken into account for item spawning, to encourage
        // delving deeper to gear up more quickly.
        let key = get_random_item_category(rng, map.difficulty).roll(rng);
        entity_to_spawn_list(rng, &mut areas, key, &mut spawn_points);
    }
    // Push entities and their spawn points to map's spawn list
    for spawn in spawn_points.iter() {
        spawn_list.push((*spawn.0, spawn.1.to_string()));
    }
}

fn entity_to_spawn_list(
    rng: &mut RandomNumberGenerator,
    possible_areas: &mut Vec<usize>,
    key: String,
    spawn_points: &mut HashMap<usize, String>,
) {
    if possible_areas.len() == 0 {
        return;
    }
    let array_idx =
        if possible_areas.len() == 1 { 0usize } else { (rng.roll_dice(1, possible_areas.len() as i32) - 1) as usize };
    let map_idx = possible_areas[array_idx];
    spawn_points.insert(map_idx, key);
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

/// Locks RAWS, and provides access to master list of all mobs.
pub fn mob_table(difficulty: i32) -> RandomTable {
    raws::table_by_name(&raws::RAWS.lock().unwrap(), "mobs", difficulty)
}

pub fn trap_table(difficulty: i32) -> RandomTable {
    raws::table_by_name(&raws::RAWS.lock().unwrap(), "traps", difficulty)
}
