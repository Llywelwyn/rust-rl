use super::{
    remove_unreachable_areas_returning_most_distant, spawner, Map, MapBuilder, Position, TileType, SHOW_MAPGEN,
};
use rltk::RandomNumberGenerator;
use specs::prelude::*;

#[allow(dead_code)]
#[derive(PartialEq, Clone)]
pub enum PrefabMode {
    RexLevel { template: &'static str },
}

pub struct PrefabBuilder {
    map: Map,
    starting_position: Position,
    depth: i32,
    history: Vec<Map>,
    mode: PrefabMode,
    spawns: Vec<(usize, String)>,
}

impl MapBuilder for PrefabBuilder {
    fn build_map(&mut self, rng: &mut RandomNumberGenerator) {
        return self.build(rng);
    }
    fn spawn_entities(&mut self, ecs: &mut World) {
        for entity in self.spawns.iter() {
            spawner::spawn_entity(ecs, &(&entity.0, &entity.1));
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

impl PrefabBuilder {
    pub fn new(new_depth: i32) -> PrefabBuilder {
        PrefabBuilder {
            map: Map::new(new_depth),
            starting_position: Position { x: 0, y: 0 },
            depth: new_depth,
            history: Vec::new(),
            mode: PrefabMode::RexLevel { template: "../resources/wfc-populated.xp" },
            spawns: Vec::new(),
        }
    }

    fn build(&mut self, rng: &mut RandomNumberGenerator) {
        match self.mode {
            PrefabMode::RexLevel { template } => self.load_rex_map(&template),
        }
        self.take_snapshot();

        // Find starting pos by starting at middle and walking left until finding a floor tile
        if self.starting_position.x == 0 {
            self.starting_position = Position { x: self.map.width / 2, y: self.map.height / 2 };
            let mut start_idx = self.map.xy_idx(self.starting_position.x, self.starting_position.y);
            while self.map.tiles[start_idx] != TileType::Floor {
                self.starting_position.x -= 1;
                start_idx = self.map.xy_idx(self.starting_position.x, self.starting_position.y);
            }
            self.take_snapshot();

            // Find all tiles we can reach from the starting point
            let exit_tile = remove_unreachable_areas_returning_most_distant(&mut self.map, start_idx);
            self.take_snapshot();

            // Place the stairs
            self.map.tiles[exit_tile] = TileType::DownStair;
            self.take_snapshot();
        }
    }

    #[allow(dead_code)]
    fn load_rex_map(&mut self, path: &str) {
        let xp_file = rltk::rex::XpFile::from_resource(path).unwrap();

        for layer in &xp_file.layers {
            for y in 0..layer.height {
                for x in 0..layer.width {
                    let cell = layer.get(x, y).unwrap();
                    if x < self.map.width as usize && y < self.map.height as usize {
                        // Saving these for later, for flipping the pref horizontally/vertically/both.
                        // let flipped_x = (self.map.width - 1) - x as i32;
                        // let flipped_y = (self.map.height - 1) - y as i32;
                        let idx = self.map.xy_idx(x as i32, y as i32);
                        match (cell.ch as u8) as char {
                            '@' => {
                                self.map.tiles[idx] = TileType::Floor;
                                self.starting_position = Position { x: x as i32, y: y as i32 }
                            }
                            ' ' => self.map.tiles[idx] = TileType::Floor,
                            '#' => self.map.tiles[idx] = TileType::Wall,
                            '>' => self.map.tiles[idx] = TileType::DownStair,
                            'g' => {
                                self.map.tiles[idx] = TileType::Floor;
                                self.spawns.push((idx, "goblin".to_string()));
                            }
                            'o' => {
                                self.map.tiles[idx] = TileType::Floor;
                                self.spawns.push((idx, "orc".to_string()));
                            }
                            '^' => {
                                self.map.tiles[idx] = TileType::Floor;
                                self.spawns.push((idx, "bear trap".to_string()));
                            }
                            '%' => {
                                self.map.tiles[idx] = TileType::Floor;
                                self.spawns.push((idx, "rations".to_string()));
                            }
                            '!' => {
                                self.map.tiles[idx] = TileType::Floor;
                                self.spawns.push((idx, "health potion".to_string()));
                            }
                            _ => {
                                rltk::console::log(format!(
                                    "Unknown glyph {} when loading map",
                                    (cell.ch as u8) as char
                                ));
                            }
                        }
                    }
                }
            }
        }
    }
}
