use super::{ Map, TileType };
use crate::{ gamelog, map_builders, OtherLevelPosition, Position, Telepath, Viewshed };
use bracket_lib::prelude::*;
use serde::{ Deserialize, Serialize };
use specs::prelude::*;
use std::collections::{ HashMap, HashSet };
use crate::data::events::*;

#[derive(Default, Serialize, Deserialize, Clone)]
pub struct MasterDungeonMap {
    maps: HashMap<i32, Map>,
    pub identified_items: HashSet<String>,
    pub scroll_map: HashMap<String, String>,
    pub potion_map: HashMap<String, String>,
    pub wand_map: HashMap<String, String>,
}

impl MasterDungeonMap {
    /// Initialises a blank MasterDungeonMap
    pub fn new() -> MasterDungeonMap {
        let mut dm = MasterDungeonMap {
            maps: HashMap::new(),
            identified_items: HashSet::new(),
            scroll_map: HashMap::new(),
            potion_map: HashMap::new(),
            wand_map: HashMap::new(),
        };
        // TODO: Use stored RNG
        let mut rng = RandomNumberGenerator::new();
        for scroll_tag in crate::raws::get_scroll_tags().iter() {
            let unid_singular = make_scroll_name(&mut rng);
            dm.scroll_map.insert(scroll_tag.to_string(), unid_singular);
        }
        let mut used_potion_names: HashSet<String> = HashSet::new();
        for potion_tag in crate::raws::get_potion_tags().iter() {
            let unid_singular = make_potion_name(&mut rng, &mut used_potion_names);
            dm.potion_map.insert(potion_tag.to_string(), unid_singular);
        }
        let mut used_wand_names: HashSet<String> = HashSet::new();
        for wand_tag in crate::raws::get_wand_tags().iter() {
            let unid_singular = make_wand_name(&mut rng, &mut used_wand_names);
            dm.wand_map.insert(wand_tag.to_string(), unid_singular);
        }

        return dm;
    }
    /// Stores the given map in the MasterDungeonMap
    pub fn store_map(&mut self, map: &Map) {
        self.maps.insert(map.id, map.clone());
    }
    /// Gets a map by ID from the MasterDungeonMap
    pub fn get_map(&self, id: i32) -> Option<Map> {
        if self.maps.contains_key(&id) {
            let result = self.maps[&id].clone();
            return Some(result);
        } else {
            return None;
        }
    }
}

fn make_scroll_name(rng: &mut RandomNumberGenerator) -> String {
    let len = 4 + rng.roll_dice(1, 6);
    let space_at_i = if len > 6 && rng.roll_dice(1, 2) == 1 {
        rng.roll_dice(1, len - 6) + 3
    } else {
        -1
    };
    let offset = rng.roll_dice(1, 2) - 1;
    let mut name = "".to_string();
    for i in 0..len {
        if i == space_at_i {
            name += " ";
        }
        if i % 2 == 0 + offset {
            let char = match rng.roll_dice(1, 5) {
                1 => "A",
                2 => "E",
                3 => "I",
                4 => "O",
                _ => "U",
            };
            name += char;
        } else {
            let char = match rng.roll_dice(1, 21) {
                1 => "B",
                2 => "C",
                3 => "D",
                4 => "F",
                5 => "G",
                6 => "H",
                7 => "J",
                8 => "K",
                9 => "L",
                10 => "M",
                11 => "N",
                12 => "P",
                13 => "Q",
                14 => "R",
                15 => "S",
                16 => "T",
                17 => "V",
                18 => "W",
                19 => "X",
                20 => "Y",
                _ => "Z",
            };
            name += char;
        }
    }
    name += " scroll";
    return name;
}

const POTION_COLOURS: &[&str] = &[
    "red",
    "orange",
    "yellow",
    "green",
    "blue",
    "indigo",
    "violet",
    "black",
    "white",
    "silver",
    "gold",
    "rainbow",
    "blood",
    "purple",
    "cyan",
    "brown",
    "grey",
    "octarine",
];
const POTION_ADJECTIVES: &[&str] = &[
    "swirling",
    "viscous",
    "effervescent",
    "slimy",
    "oily",
    "metallic",
    "prismatic",
    "goopy",
];

fn make_potion_name(rng: &mut RandomNumberGenerator, used_names: &mut HashSet<String>) -> String {
    loop {
        let mut name: String =
            POTION_ADJECTIVES[
                (rng.roll_dice(1, POTION_ADJECTIVES.len() as i32) as usize) - 1
            ].to_string();
        name += " ";
        name += POTION_COLOURS[(rng.roll_dice(1, POTION_COLOURS.len() as i32) as usize) - 1];
        name += " potion";

        if !used_names.contains(&name) {
            used_names.insert(name.clone());
            return name;
        }
    }
}

const WAND_TYPES: &[&str] = &[
    // Shapes
    "curved",
    "squircle",
    "hexagonal",
    "octagonal",
    "pointed",
    "long",
    // Metals
    "iron",
    "steel",
    "silver",
    "gold",
    "lead",
    "tungsten",
    // Other
    "mahogany",
    "ebony",
    "crystalline",
    "jeweled",
];

fn make_wand_name(rng: &mut RandomNumberGenerator, used_names: &mut HashSet<String>) -> String {
    loop {
        let mut name: String =
            WAND_TYPES[(rng.roll_dice(1, WAND_TYPES.len() as i32) as usize) - 1].to_string();
        name += " wand";

        if !used_names.contains(&name) {
            used_names.insert(name.clone());
            return name;
        }
    }
}

pub fn level_transition(ecs: &mut World, new_id: i32, dest_tile: TileType) -> Option<Vec<Map>> {
    // Obtain master
    let dungeon_master = ecs.read_resource::<MasterDungeonMap>();
    if dungeon_master.get_map(new_id).is_some() {
        std::mem::drop(dungeon_master);
        transition_to_existing_map(ecs, new_id, dest_tile);
        return None;
    } else {
        std::mem::drop(dungeon_master);
        return Some(transition_to_new_map(ecs, new_id, dest_tile));
    }
}

fn transition_to_existing_map(ecs: &mut World, new_id: i32, dest_tile: TileType) {
    let mut dungeon_master = ecs.write_resource::<MasterDungeonMap>();
    // Unwrapping here panics if new_id isn't present. But this should
    // never be called without new_id being present by level_transition.
    let map = dungeon_master.get_map(new_id).unwrap();
    let mut worldmap_resource = ecs.write_resource::<Map>();
    let player_entity = ecs.fetch::<Entity>();

    let w = map.width;
    let mut possible_destinations: Vec<usize> = Vec::new();
    for (idx, tt) in map.tiles.iter().enumerate() {
        if *tt == dest_tile {
            possible_destinations.push(idx);
        }
    }
    if possible_destinations.is_empty() {
        console::log("WARNING: No destination tiles found on map transition.");
        match dest_tile {
            TileType::DownStair => console::log("DESTINATION: DownStair"),
            TileType::UpStair => console::log("DESTINATION: UpStair"),
            TileType::ToOvermap(id) => console::log(format!("DESTINATION: ToOvermap({})", id)),
            TileType::ToLocal(id) => console::log(format!("DESTINATION: ToLocal({})", id)),
            _ => console::log("DESTINATION: Unknown"),
        }
        possible_destinations.push(((map.width * map.height) as usize) / 2); // Centre of map
    }
    let mut rng = ecs.write_resource::<RandomNumberGenerator>();
    let idx =
        possible_destinations[(rng.roll_dice(1, possible_destinations.len() as i32) as usize) - 1];
    let mut player_position = ecs.write_resource::<Point>();
    *player_position = Point::new((idx as i32) % w, (idx as i32) / w);
    let mut position_components = ecs.write_storage::<Position>();
    let player_pos_component = position_components.get_mut(*player_entity);
    if let Some(player_pos_component) = player_pos_component {
        player_pos_component.x = (idx as i32) % w;
        player_pos_component.y = (idx as i32) / w;
    }
    dungeon_master.store_map(&worldmap_resource);
    *worldmap_resource = map;
    // Dirtify viewsheds (forces refresh)
    let mut viewshed_components = ecs.write_storage::<Viewshed>();
    let mut telepath_components = ecs.write_storage::<Telepath>();
    let vision_vs = viewshed_components.get_mut(*player_entity);
    let telepath_vs = telepath_components.get_mut(*player_entity);
    if let Some(vs) = vision_vs {
        vs.dirty = true;
    }
    if let Some(vs) = telepath_vs {
        vs.dirty = true;
    }
}

fn transition_to_new_map(ecs: &mut World, new_id: i32, _dest_tile: TileType) -> Vec<Map> {
    let mut rng = ecs.write_resource::<RandomNumberGenerator>();
    // Might need this to fallback to 1, but if player
    // level isn't found at all, there's a bigger concern
    // concern than just this function not working.
    let player_level = gamelog::get_event_count(EVENT::COUNT_LEVEL);
    let mut builder = map_builders::level_builder(new_id, &mut rng, 100, 50, player_level);
    builder.build_map(&mut rng);
    std::mem::drop(rng);
    let mapgen_history = builder.build_data.history.clone();
    let player_start;
    let old_map: Map;
    {
        let mut worldmap_resource = ecs.write_resource::<Map>();
        old_map = worldmap_resource.clone();
        // If there is zero overmap involvement, place an upstair where we ended up.
        // Otherwise, this should be hand-placed.
        if !old_map.overmap && !builder.build_data.map.overmap {
            if let Some(pos) = &builder.build_data.starting_position {
                let up_idx = builder.build_data.map.xy_idx(pos.x, pos.y);
                builder.build_data.map.tiles[up_idx] = TileType::UpStair;
            }
        }
        *worldmap_resource = builder.build_data.map.clone();
        // Unwrap so we get a CTD if there's no starting pos.
        player_start = builder.build_data.starting_position.as_mut().unwrap().clone();
    }
    // Spawn entities
    builder.spawn_entities(ecs);
    // Place player and update resources
    let mut player_position = ecs.write_resource::<Point>();
    *player_position = Point::new(player_start.x, player_start.y);
    let mut position_components = ecs.write_storage::<Position>();
    let player_entity = ecs.fetch::<Entity>();
    let player_pos_component = position_components.get_mut(*player_entity);
    if let Some(player_pos_component) = player_pos_component {
        player_pos_component.x = player_start.x;
        player_pos_component.y = player_start.y;
    }
    // Mark viewshed as dirty (force refresh)
    let mut viewshed_components = ecs.write_storage::<Viewshed>();
    let mut telepath_components = ecs.write_storage::<Telepath>();
    let vision_vs = viewshed_components.get_mut(*player_entity);
    let telepath_vs = telepath_components.get_mut(*player_entity);
    if let Some(vs) = vision_vs {
        vs.dirty = true;
    }
    if let Some(vs) = telepath_vs {
        vs.dirty = true;
    }
    // Store newly minted map
    let mut dungeon_master = ecs.write_resource::<MasterDungeonMap>();
    dungeon_master.store_map(&old_map);
    dungeon_master.store_map(&builder.build_data.map);
    return mapgen_history;
}

/// Iterate through entities on the current level, save the current position and floor
/// of each entity-to-be-frozen, and then delete their current position.
pub fn freeze_entities(ecs: &mut World) {
    // Obtain reqs from ECS
    let entities = ecs.entities();
    let mut positions = ecs.write_storage::<Position>();
    let mut other_positions = ecs.write_storage::<OtherLevelPosition>();
    let player_entity = ecs.fetch::<Entity>();
    let map_id = ecs.fetch::<Map>().id;
    // Save Positions and mark for deletion
    let mut pos_to_delete: Vec<Entity> = Vec::new();
    for (entity, pos) in (&entities, &positions).join() {
        if entity != *player_entity {
            other_positions
                .insert(entity, OtherLevelPosition { x: pos.x, y: pos.y, id: map_id })
                .expect("Failed to insert OtherLevelPosition");
            pos_to_delete.push(entity);
        }
    }
    for p in pos_to_delete.iter() {
        positions.remove(*p);
    }
}

/// Iterate through entities, and insert a Position component if the
/// entity has an OtherLevelPosition for the new map id.
pub fn thaw_entities(ecs: &mut World) {
    // Obtain reqs from ECS
    let entities = ecs.entities();
    let mut positions = ecs.write_storage::<Position>();
    let mut other_positions = ecs.write_storage::<OtherLevelPosition>();
    let player_entity = ecs.fetch::<Entity>();
    let map_id = ecs.fetch::<Map>().id;
    // Save Positions and mark for deletion
    let mut pos_to_delete: Vec<Entity> = Vec::new();
    for (entity, pos) in (&entities, &other_positions).join() {
        if entity != *player_entity && pos.id == map_id {
            positions
                .insert(entity, Position { x: pos.x, y: pos.y })
                .expect("Failed to insert OtherLevelPosition");
            pos_to_delete.push(entity);
        }
    }
    for p in pos_to_delete.iter() {
        other_positions.remove(*p);
    }
}
