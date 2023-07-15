use super::{
    generate_voronoi_spawn_regions, remove_unreachable_areas_returning_most_distant, spawner, Map, MapBuilder,
    Position, TileType, SHOW_MAPGEN,
};
use rltk::RandomNumberGenerator;
use specs::prelude::*;
use std::collections::HashMap;

const PASSES: i32 = 15;

pub struct CellularAutomataBuilder {
    map: Map,
    starting_position: Position,
    depth: i32,
    history: Vec<Map>,
    noise_areas: HashMap<i32, Vec<usize>>,
}

impl MapBuilder for CellularAutomataBuilder {
    fn build_map(&mut self, rng: &mut RandomNumberGenerator) {
        return self.build(rng);
    }
    fn spawn_entities(&mut self, ecs: &mut World) {
        for area in self.noise_areas.iter() {
            spawner::spawn_region(ecs, area.1, self.depth);
        }
    }
    //  Getters
    fn get_map(&mut self) -> Map {
        return self.map.clone();
    }
    fn get_starting_pos(&mut self) -> Position {
        return self.starting_position.clone();
    }
    // Mapgen visualisation stuff
    fn get_snapshot_history(&self) -> Vec<Map> {
        return self.history.clone();
    }
    fn take_snapshot(&mut self) {
        if SHOW_MAPGEN {
            let mut snapshot = self.map.clone();
            for v in snapshot.revealed_tiles.iter_mut() {
                *v = true;
            }
            self.history.push(snapshot);
        }
    }
}

impl CellularAutomataBuilder {
    pub fn new(new_depth: i32) -> CellularAutomataBuilder {
        CellularAutomataBuilder {
            map: Map::new(new_depth),
            starting_position: Position { x: 0, y: 0 },
            depth: new_depth,
            history: Vec::new(),
            noise_areas: HashMap::new(),
        }
    }

    fn build(&mut self, rng: &mut RandomNumberGenerator) {
        // Set 55% of map to floor
        for y in 1..self.map.height - 1 {
            for x in 1..self.map.width - 1 {
                let roll = rng.roll_dice(1, 100);
                let idx = self.map.xy_idx(x, y);
                if roll > 55 {
                    self.map.tiles[idx] = TileType::Floor
                } else {
                    self.map.tiles[idx] = TileType::Wall
                }
            }
        }
        self.take_snapshot();

        // Iteratively apply cellular automata rules
        for _i in 0..PASSES {
            let mut newtiles = self.map.tiles.clone();

            for y in 1..self.map.height - 1 {
                for x in 1..self.map.width - 1 {
                    let idx = self.map.xy_idx(x, y);
                    let mut neighbours = 0;
                    if self.map.tiles[idx - 1] == TileType::Wall {
                        neighbours += 1;
                    }
                    if self.map.tiles[idx + 1] == TileType::Wall {
                        neighbours += 1;
                    }
                    if self.map.tiles[idx - self.map.width as usize] == TileType::Wall {
                        neighbours += 1;
                    }
                    if self.map.tiles[idx + self.map.width as usize] == TileType::Wall {
                        neighbours += 1;
                    }
                    if self.map.tiles[idx - (self.map.width as usize - 1)] == TileType::Wall {
                        neighbours += 1;
                    }
                    if self.map.tiles[idx - (self.map.width as usize + 1)] == TileType::Wall {
                        neighbours += 1;
                    }
                    if self.map.tiles[idx + (self.map.width as usize - 1)] == TileType::Wall {
                        neighbours += 1;
                    }
                    if self.map.tiles[idx + (self.map.width as usize + 1)] == TileType::Wall {
                        neighbours += 1;
                    }

                    if neighbours > 4 || neighbours == 0 {
                        newtiles[idx] = TileType::Wall;
                    } else {
                        newtiles[idx] = TileType::Floor;
                    }
                }
            }
            self.map.tiles = newtiles.clone();
            self.take_snapshot();
        }

        // Find a starting point; start at the middle and walk left until we find an open tile
        self.starting_position = Position { x: self.map.width / 2, y: self.map.height / 2 };
        let mut start_idx = self.map.xy_idx(self.starting_position.x, self.starting_position.y);
        while self.map.tiles[start_idx] != TileType::Floor {
            self.starting_position.x -= 1;
            start_idx = self.map.xy_idx(self.starting_position.x, self.starting_position.y);
        }

        // Find all tiles reachable from starting pos
        let exit_tile = remove_unreachable_areas_returning_most_distant(&mut self.map, start_idx);
        self.take_snapshot();

        // Place stairs
        self.map.tiles[exit_tile] = TileType::DownStair;
        self.take_snapshot();

        // Noise map for spawning entities
        self.noise_areas = generate_voronoi_spawn_regions(&self.map, rng);
    }
}
