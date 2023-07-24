use rltk::{Algorithm2D, BaseMap, Point};
use serde::{Deserialize, Serialize};
use specs::prelude::*;
use std::collections::HashSet;

#[derive(PartialEq, Eq, Hash, Copy, Clone, Serialize, Deserialize)]
pub enum TileType {
    Wall,
    Floor,
    DownStair,
}

// FIXME: If the map size gets too small, entities stop being rendered starting from the right.
// i.e. on a map size of 40*40, only entities to the left of the player are rendered.
//      on a map size of 42*42, the player can see entities up to 2 tiles to their right.

#[derive(Default, Serialize, Deserialize, Clone)]
pub struct Map {
    pub tiles: Vec<TileType>,
    pub width: i32,
    pub height: i32,
    pub revealed_tiles: Vec<bool>,
    pub visible_tiles: Vec<bool>,
    pub lit_tiles: Vec<bool>,
    pub telepath_tiles: Vec<bool>,
    // Combine these offsets into one Vec<(u8, u8, u8)>
    pub red_offset: Vec<u8>,
    pub green_offset: Vec<u8>,
    pub blue_offset: Vec<u8>,
    pub blocked: Vec<bool>,
    pub depth: i32,
    pub bloodstains: HashSet<usize>,
    pub view_blocked: HashSet<usize>,

    #[serde(skip_serializing)]
    #[serde(skip_deserializing)]
    pub tile_content: Vec<Vec<Entity>>,
}

impl Map {
    pub fn xy_idx(&self, x: i32, y: i32) -> usize {
        (y as usize) * (self.width as usize) + (x as usize)
    }

    pub fn new(new_depth: i32, width: i32, height: i32) -> Map {
        let map_tile_count = (width * height) as usize;
        let mut map = Map {
            tiles: vec![TileType::Wall; map_tile_count],
            width: width,
            height: height,
            revealed_tiles: vec![false; map_tile_count],
            visible_tiles: vec![false; map_tile_count],
            lit_tiles: vec![true; map_tile_count], // NYI: Light sources. Once those exist, we can set this to false.
            telepath_tiles: vec![false; map_tile_count],
            red_offset: vec![0; map_tile_count],
            green_offset: vec![0; map_tile_count],
            blue_offset: vec![0; map_tile_count],
            blocked: vec![false; map_tile_count],
            depth: new_depth,
            bloodstains: HashSet::new(),
            view_blocked: HashSet::new(),
            tile_content: vec![Vec::new(); map_tile_count],
        };

        const MAX_OFFSET: u8 = 32;
        let mut rng = rltk::RandomNumberGenerator::new();

        for idx in 0..map.red_offset.len() {
            let roll = rng.roll_dice(1, MAX_OFFSET as i32);
            map.red_offset[idx] = roll as u8;
        }
        for idx in 0..map.green_offset.len() {
            let roll = rng.roll_dice(1, MAX_OFFSET as i32);
            map.green_offset[idx] = roll as u8;
        }
        for idx in 0..map.blue_offset.len() {
            let roll = rng.roll_dice(1, MAX_OFFSET as i32);
            map.blue_offset[idx] = roll as u8;
        }

        return map;
    }

    /// Takes an index, and calculates if it can be entered.
    fn is_exit_valid(&self, x: i32, y: i32) -> bool {
        if x < 1 || x > self.width - 1 || y < 1 || y > self.height - 1 {
            return false;
        }
        let idx = self.xy_idx(x, y);
        !self.blocked[idx]
    }

    pub fn populate_blocked(&mut self) {
        for (i, tile) in self.tiles.iter_mut().enumerate() {
            self.blocked[i] = *tile == TileType::Wall;
        }
    }

    pub fn clear_content_index(&mut self) {
        for content in self.tile_content.iter_mut() {
            content.clear();
        }
    }
}

impl Algorithm2D for Map {
    fn dimensions(&self) -> Point {
        Point::new(self.width, self.height)
    }
}

impl BaseMap for Map {
    fn is_opaque(&self, idx: usize) -> bool {
        let idx_u = idx as usize;
        return self.tiles[idx_u] == TileType::Wall || self.view_blocked.contains(&idx_u);
    }

    fn get_pathing_distance(&self, idx1: usize, idx2: usize) -> f32 {
        let w = self.width as usize;
        let p1 = Point::new(idx1 % w, idx1 / w);
        let p2 = Point::new(idx2 % w, idx2 / w);
        rltk::DistanceAlg::Pythagoras.distance2d(p1, p2)
    }

    /// Evaluate every possible exit from a given tile in a cardinal direction, and return it as a vector.
    fn get_available_exits(&self, idx: usize) -> rltk::SmallVec<[(usize, f32); 10]> {
        let mut exits = rltk::SmallVec::new();
        let x = (idx as i32) % self.width;
        let y = (idx as i32) / self.width;
        let w = self.width as usize;

        // Cardinal directions
        if self.is_exit_valid(x - 1, y) {
            exits.push((idx - 1, 1.0));
        }
        if self.is_exit_valid(x + 1, y) {
            exits.push((idx + 1, 1.0));
        }
        if self.is_exit_valid(x, y - 1) {
            exits.push((idx - w, 1.0));
        }
        if self.is_exit_valid(x, y + 1) {
            exits.push((idx + w, 1.0));
        }

        // Diagonals
        if self.is_exit_valid(x - 1, y - 1) {
            exits.push((idx - w - 1, 1.45));
        }
        if self.is_exit_valid(x + 1, y - 1) {
            exits.push((idx - w + 1, 1.45));
        }
        if self.is_exit_valid(x - 1, y + 1) {
            exits.push((idx + w - 1, 1.45));
        }
        if self.is_exit_valid(x + 1, y + 1) {
            exits.push((idx + w + 1, 1.45));
        }

        exits
    }
}
