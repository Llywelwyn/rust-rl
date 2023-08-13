use super::Map;
use serde::{Deserialize, Serialize};
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
