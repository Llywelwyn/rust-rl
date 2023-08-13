use super::{Map, TileType};
use crate::{gamelog, map_builders, Position, Telepath, Viewshed};
use rltk::prelude::*;
use serde::{Deserialize, Serialize};
use specs::prelude::*;
use std::collections::HashMap;

#[derive(Default, Serialize, Deserialize, Clone)]
pub struct MasterDungeonMap {
    maps: HashMap<i32, Map>,
}

impl MasterDungeonMap {
    /// Initialises a blank MasterDungeonMap
    pub fn new() -> MasterDungeonMap {
        return MasterDungeonMap { maps: HashMap::new() };
    }
    /// Stores the given map in the MasterDungeonMap
    pub fn store_map(&mut self, map: &Map) {
        self.maps.insert(map.id, map.clone());
    }
    /// Gets a map by ID from the MasterDungeonMap
    pub fn get_map(&self, id: i32) -> Option<Map> {
        if self.maps.contains_key(&id) {
            let mut result = self.maps[&id].clone();
            result.tile_content = vec![Vec::new(); (result.width * result.height) as usize];
            return Some(result);
        } else {
            return None;
        }
    }
}

pub fn level_transition(ecs: &mut World, new_id: i32, offset: i32) -> Option<Vec<Map>> {
    // Obtain master
    let dungeon_master = ecs.read_resource::<MasterDungeonMap>();
    if dungeon_master.get_map(new_id).is_some() {
        std::mem::drop(dungeon_master);
        transition_to_existing_map(ecs, new_id, offset);
        return None;
    } else {
        std::mem::drop(dungeon_master);
        return Some(transition_to_new_map(ecs, new_id));
    }
}

fn transition_to_existing_map(ecs: &mut World, new_id: i32, offset: i32) {
    let dungeon_master = ecs.read_resource::<MasterDungeonMap>();
    // Unwrapping here panics if new_id isn't present. But this should
    // never be called without new_id being present by level_transition.
    let map = dungeon_master.get_map(new_id).unwrap();
    let mut worldmap_resource = ecs.write_resource::<Map>();
    let player_entity = ecs.fetch::<Entity>();
    // Find down stairs, place player
    let w = map.width;
    let stair_type = if offset < 0 { TileType::DownStair } else { TileType::UpStair };
    for (idx, tt) in map.tiles.iter().enumerate() {
        if *tt == stair_type {
            let mut player_position = ecs.write_resource::<Point>();
            *player_position = Point::new(idx as i32 % w, idx as i32 / w);
            let mut position_components = ecs.write_storage::<Position>();
            let player_pos_component = position_components.get_mut(*player_entity);
            if let Some(player_pos_component) = player_pos_component {
                player_pos_component.x = idx as i32 % w;
                player_pos_component.y = idx as i32 / w;
            }
        }
    }
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

fn transition_to_new_map(ecs: &mut World, new_id: i32) -> Vec<Map> {
    let mut rng = ecs.write_resource::<rltk::RandomNumberGenerator>();
    // Might need this to fallback to 1, but if player
    // level isn't found at all, there's a bigger concern
    // concern than just this function not working.
    let player_level = gamelog::get_event_count("player_level");
    let mut builder = map_builders::level_builder(new_id, &mut rng, 100, 50, player_level);
    builder.build_map(&mut rng);
    std::mem::drop(rng);
    if new_id > 1 {
        if let Some(pos) = &builder.build_data.starting_position {
            let up_idx = builder.build_data.map.xy_idx(pos.x, pos.y);
            builder.build_data.map.tiles[up_idx] = TileType::UpStair;
        }
    }
    let mapgen_history = builder.build_data.history.clone();
    let player_start;
    {
        let mut worldmap_resource = ecs.write_resource::<Map>();
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
    dungeon_master.store_map(&builder.build_data.map);
    return mapgen_history;
}
