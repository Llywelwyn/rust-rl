use super::{ BuilderMap, MetaMapBuilder, TileType };
use bracket_lib::prelude::*;

pub struct DoorPlacement {}

impl MetaMapBuilder for DoorPlacement {
    #[allow(dead_code)]
    fn build_map(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        self.doors(rng, build_data);
    }
}

impl DoorPlacement {
    #[allow(dead_code)]
    pub fn new() -> Box<DoorPlacement> {
        Box::new(DoorPlacement {})
    }

    fn doors(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        if let Some(halls_original) = &build_data.corridors {
            let halls = halls_original.clone(); // Avoids nested borrow
            for hall in halls.iter() {
                if hall.len() > 2 {
                    if self.door_possible(build_data, hall[0]) {
                        build_data.spawn_list.push((hall[0], "door".to_string()));
                    }
                }
            }
        } else {
            // There are no corridors - scan for possible places
            let tiles = build_data.map.tiles.clone();
            for (i, tile) in tiles.iter().enumerate() {
                if
                    *tile == TileType::Floor &&
                    self.door_possible(build_data, i) &&
                    rng.roll_dice(1, 6) == 1
                {
                    build_data.spawn_list.push((i, "door".to_string()));
                }
            }
        }
    }

    fn door_possible(&self, build_data: &mut BuilderMap, idx: usize) -> bool {
        // Iterate through spawn list. If another entity wants to spawn on this tile, return false
        for spawn in build_data.spawn_list.iter() {
            if spawn.0 == idx {
                return false;
            }
        }

        let x = idx % (build_data.map.width as usize);
        let y = idx / (build_data.map.width as usize);

        // Check for east-west door possibility
        if
            build_data.map.tiles[idx] == TileType::Floor &&
            x > 1 &&
            build_data.map.tiles[idx - 1] == TileType::Floor &&
            x < (build_data.map.width as usize) - 2 &&
            build_data.map.tiles[idx + 1] == TileType::Floor &&
            y > 1 &&
            build_data.map.tiles[idx - (build_data.map.width as usize)] == TileType::Wall &&
            y < (build_data.map.height as usize) - 2 &&
            build_data.map.tiles[idx + (build_data.map.width as usize)] == TileType::Wall
        {
            return true;
        }

        // Check for north-south door possibility
        if
            build_data.map.tiles[idx] == TileType::Floor &&
            x > 1 &&
            build_data.map.tiles[idx - 1] == TileType::Wall &&
            x < (build_data.map.width as usize) - 2 &&
            build_data.map.tiles[idx + 1] == TileType::Wall &&
            y > 1 &&
            build_data.map.tiles[idx - (build_data.map.width as usize)] == TileType::Floor &&
            y < (build_data.map.height as usize) - 2 &&
            build_data.map.tiles[idx + (build_data.map.width as usize)] == TileType::Floor
        {
            return true;
        }

        false
    }
}
