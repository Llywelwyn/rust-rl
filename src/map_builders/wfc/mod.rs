use super::{
    generate_voronoi_spawn_regions, remove_unreachable_areas_returning_most_distant, spawner, Map, MapBuilder,
    Position, TileType, SHOW_MAPGEN,
};
mod image_loader;
use image_loader::load_rex_map;
mod common;
use common::MapChunk;
mod constraints;
mod solver;
use rltk::RandomNumberGenerator;
use solver::Solver;
use specs::prelude::*;
use std::collections::HashMap;

#[derive(PartialEq, Copy, Clone)]
pub enum WaveFunctionMode {
    TestMap,
    Derived,
}

pub struct WaveFunctionCollapseBuilder {
    map: Map,
    starting_position: Position,
    depth: i32,
    history: Vec<Map>,
    noise_areas: HashMap<i32, Vec<usize>>,
    mode: WaveFunctionMode,
    derive_from: Option<Box<dyn MapBuilder>>,
}

impl MapBuilder for WaveFunctionCollapseBuilder {
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

impl WaveFunctionCollapseBuilder {
    pub fn new(
        new_depth: i32,
        mode: WaveFunctionMode,
        derive_from: Option<Box<dyn MapBuilder>>,
    ) -> WaveFunctionCollapseBuilder {
        WaveFunctionCollapseBuilder {
            map: Map::new(new_depth),
            starting_position: Position { x: 0, y: 0 },
            depth: new_depth,
            history: Vec::new(),
            noise_areas: HashMap::new(),
            mode,
            derive_from,
        }
    }
    pub fn test_map(new_depth: i32) -> WaveFunctionCollapseBuilder {
        WaveFunctionCollapseBuilder::new(new_depth, WaveFunctionMode::TestMap, None)
    }
    pub fn derived_map(new_depth: i32, builder: Box<dyn MapBuilder>) -> WaveFunctionCollapseBuilder {
        WaveFunctionCollapseBuilder::new(new_depth, WaveFunctionMode::Derived, Some(builder))
    }

    fn build(&mut self, rng: &mut RandomNumberGenerator) {
        if self.mode == WaveFunctionMode::TestMap {
            self.map =
                load_rex_map(self.depth, &rltk::rex::XpFile::from_resource("../resources/wfc-demo1.xp").unwrap());
            self.take_snapshot();
            return;
        }

        const CHUNK_SIZE: i32 = 8;

        let prebuilder = &mut self.derive_from.as_mut().unwrap();
        prebuilder.build_map(rng);
        self.map = prebuilder.get_map();
        for t in self.map.tiles.iter_mut() {
            if *t == TileType::DownStair {
                *t = TileType::Floor;
            }
        }
        self.take_snapshot();

        let patterns = constraints::build_patterns(&self.map, CHUNK_SIZE, true, true);
        let constraints = common::patterns_to_constraints(patterns, CHUNK_SIZE);
        self.render_tile_gallery(&constraints, CHUNK_SIZE);

        // Call solver
        self.map = Map::new(self.depth);
        loop {
            let mut solver = Solver::new(constraints.clone(), CHUNK_SIZE, &self.map);
            while !solver.iteration(&mut self.map, rng) {
                self.take_snapshot();
            }
            self.take_snapshot();
            if solver.possible {
                break;
            }
        }

        // Find a starting point; start at the middle and walk left until we find an open tile
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

        // Now we build a noise map for use in spawning entities later
        self.noise_areas = generate_voronoi_spawn_regions(&self.map, rng);
    }

    fn render_tile_gallery(&mut self, constraints: &Vec<MapChunk>, chunk_size: i32) {
        self.map = Map::new(0);
        let mut counter = 0;
        let mut x = 1;
        let mut y = 1;
        while counter < constraints.len() {
            constraints::render_pattern_to_map(&mut self.map, &constraints[counter], chunk_size, x, y);
            x += chunk_size + 1;
            if x + chunk_size > self.map.width {
                // Next row
                x = 1;
                y += chunk_size + 1;

                if y + chunk_size > self.map.height {
                    // Next page
                    self.take_snapshot();
                    self.map = Map::new(0);
                    x = 1;
                    y = 1;
                }
            }
            counter += 1;
        }
        self.take_snapshot();
    }
}
