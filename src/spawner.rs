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

// Consts
const MAX_ENTITIES: i32 = 2;

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

    if areas.len() == 0 {
        rltk::console::log("DEBUGINFO: No areas capable of spawning mobs!");
        return;
    }

    if rng.roll_dice(1, 3) == 1 {
        let array_idx = if areas.len() == 1 { 0usize } else { (rng.roll_dice(1, areas.len() as i32) - 1) as usize };
        let map_idx = areas[array_idx];
        spawn_points.insert(map_idx, mob_table(difficulty).roll(rng));
        areas.remove(array_idx);
    }

    let num_spawns = i32::min(areas.len() as i32, rng.roll_dice(1, MAX_ENTITIES + 2) - 2);
    if num_spawns <= 0 {
        return;
    }

    for _i in 0..num_spawns {
        let category = category_table().roll(rng);
        let spawn_table;
        match category.as_ref() {
            "item" => {
                let item_category = item_category_table().roll(rng);
                match item_category.as_ref() {
                    "equipment" => spawn_table = equipment_table(difficulty),
                    "potion" => spawn_table = potion_table(difficulty),
                    "scroll" => spawn_table = scroll_table(difficulty),
                    "wand" => spawn_table = wand_table(difficulty),
                    _ => spawn_table = debug_table(),
                }
            }
            "food" => spawn_table = food_table(difficulty),
            "trap" => spawn_table = trap_table(difficulty),
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

// 3 items : 1 food : 1 trap
fn category_table() -> RandomTable {
    return RandomTable::new().add("item", 3).add("food", 1).add("trap", 1);
}

// 3 scrolls : 3 potions : 1 equipment : 1 wand?
fn item_category_table() -> RandomTable {
    return RandomTable::new().add("equipment", 1).add("potion", 3).add("scroll", 3).add("wand", 1);
}

fn debug_table() -> RandomTable {
    return RandomTable::new().add("debug", 1);
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
