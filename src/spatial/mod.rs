use crate::{ tile_walkable, Map, RunState };
use specs::prelude::*;
use std::sync::Mutex;

mod map_indexing_system;
pub use map_indexing_system::MapIndexingSystem;

struct SpatialMap {
    blocked: Vec<(bool, bool)>,
    tile_content: Vec<Vec<(Entity, bool)>>,
}

impl SpatialMap {
    fn new() -> Self {
        return Self { blocked: Vec::new(), tile_content: Vec::new() };
    }
}

lazy_static! {
    static ref SPATIAL_MAP: Mutex<SpatialMap> = Mutex::new(SpatialMap::new());
}

/// Sets the size of the SpatialMap.
pub fn set_size(map_tile_count: usize) {
    let mut lock = SPATIAL_MAP.lock().unwrap();
    lock.blocked = vec![(false, false); map_tile_count];
    lock.tile_content = vec![Vec::new(); map_tile_count];
}

/// Clears the SpatialMap. Blocked is set to (false, false),
/// and all tile content is cleared.
pub fn clear() {
    let mut lock = SPATIAL_MAP.lock().unwrap();
    lock.blocked.iter_mut().for_each(|b| {
        b.0 = false;
        b.1 = false;
    });
    for content in lock.tile_content.iter_mut() {
        content.clear();
    }
}

/// Iterates through every tile in the map, setting the SpatialMap's
/// blocked-by-map tuple entry to true wherever a tile is impassable.
pub fn populate_blocked_from_map(map: &Map) {
    let mut lock = SPATIAL_MAP.lock().unwrap();
    for (i, tile) in map.tiles.iter().enumerate() {
        lock.blocked[i].0 = !tile_walkable(*tile);
    }
}

/// Indexes a new entity within the SpatialMap, storing the entity
/// and their BlocksTile status.
pub fn index_entity(entity: Entity, idx: usize, blocks_tile: bool) {
    let mut lock = SPATIAL_MAP.lock().unwrap();
    lock.tile_content[idx].push((entity, blocks_tile));
    if blocks_tile {
        lock.blocked[idx].1 = true;
    }
}

/// Removes an entity from SpatialMap tilecontent.
pub fn remove_entity(entity: Entity, idx: usize) {
    let mut lock = SPATIAL_MAP.lock().unwrap();
    lock.tile_content[idx].retain(|(e, _)| *e != entity);
    let mut from_blocked = false;
    lock.tile_content[idx].iter().for_each(|(_, blocks)| {
        if *blocks {
            from_blocked = true;
        }
    });
    lock.blocked[idx].1 = from_blocked;
}

/// Returns is_empty on a given tile content idx.
pub fn has_tile_content(idx: usize) -> bool {
    let lock = SPATIAL_MAP.lock().unwrap();
    if lock.tile_content[idx].is_empty() {
        return false;
    }
    return true;
}

/// Returns the number of entries on a given index.
pub fn length(idx: usize) -> usize {
    let lock = SPATIAL_MAP.lock().unwrap();
    return lock.tile_content[idx].len();
}

/// Returns true if the idx is blocked by either a map tile or an entity.
pub fn is_blocked(idx: usize) -> bool {
    let lock = SPATIAL_MAP.lock().unwrap();
    return lock.blocked[idx].0 || lock.blocked[idx].1;
}

/// Calls a function on every entity within a given tile idx.
pub fn for_each_tile_content<F>(idx: usize, mut f: F) where F: FnMut(Entity) {
    let lock = SPATIAL_MAP.lock().unwrap();
    for entity in lock.tile_content[idx].iter() {
        f(entity.0);
    }
}

/// Calls a function on every entity within a given tile idx, with the
/// added ability to return a RunState mid-calc.
pub fn for_each_tile_content_with_runstate<F>(idx: usize, mut f: F) -> Option<RunState>
    where F: FnMut(Entity) -> Option<RunState>
{
    let lock = SPATIAL_MAP.lock().unwrap();
    for entity in lock.tile_content[idx].iter() {
        if let Some(rs) = f(entity.0) {
            return Some(rs);
        }
    }
    return None;
}

/// Calls a function on every entity within a given tile idx, breaking if
/// the closure ever returns false.
pub fn for_each_tile_content_with_bool<F>(idx: usize, mut f: F) where F: FnMut(Entity) -> bool {
    let lock = SPATIAL_MAP.lock().unwrap();
    for entity in lock.tile_content[idx].iter() {
        if !f(entity.0) {
            break;
        }
    }
}

/// Moves an entity from one index to another in the SpatialMap, and
/// recalculates blocks for both affected tiles.
pub fn move_entity(entity: Entity, moving_from: usize, moving_to: usize) {
    let mut lock = SPATIAL_MAP.lock().unwrap();
    let mut entity_blocks = false;
    lock.tile_content[moving_from].retain(|(e, blocks)| {
        if *e == entity {
            entity_blocks = *blocks;
            return false;
        } else {
            return true;
        }
    });
    lock.tile_content[moving_to].push((entity, entity_blocks));
    // Recalculate blocks
    let mut from_blocked = false;
    let mut to_blocked = false;
    lock.tile_content[moving_from].iter().for_each(|(_, blocks)| {
        if *blocks {
            from_blocked = true;
        }
    });
    lock.tile_content[moving_to].iter().for_each(|(_, blocks)| {
        if *blocks {
            to_blocked = true;
        }
    });
    lock.blocked[moving_from].1 = from_blocked;
    lock.blocked[moving_to].1 = to_blocked;
}
