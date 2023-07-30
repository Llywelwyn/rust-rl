use rltk::{Algorithm2D, BaseMap, Point};
use serde::{Deserialize, Serialize};
use specs::prelude::*;
use std::collections::HashSet;
pub mod colours;
mod glyphs;
mod tiletype;
pub use tiletype::{tile_cost, tile_opaque, tile_walkable, TileType};
pub mod themes;

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
    pub colour_offset: Vec<(f32, f32, f32)>,
    pub additional_fg_offset: rltk::RGB,
    pub blocked: Vec<bool>,
    pub id: i32,
    pub name: String,
    pub difficulty: i32,
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

    pub fn new<S: ToString>(new_id: i32, width: i32, height: i32, difficulty: i32, name: S) -> Map {
        let map_tile_count = (width * height) as usize;
        let mut map = Map {
            tiles: vec![TileType::Wall; map_tile_count],
            width: width,
            height: height,
            revealed_tiles: vec![false; map_tile_count],
            visible_tiles: vec![false; map_tile_count],
            lit_tiles: vec![true; map_tile_count], // NYI: Light sources. Once those exist, we can set this to false.
            telepath_tiles: vec![false; map_tile_count],
            colour_offset: vec![(1.0, 1.0, 1.0); map_tile_count],
            additional_fg_offset: rltk::RGB::from_u8(OFFSET_PERCENT as u8, OFFSET_PERCENT as u8, OFFSET_PERCENT as u8),
            blocked: vec![false; map_tile_count],
            id: new_id,
            name: name.to_string(),
            difficulty: difficulty,
            bloodstains: HashSet::new(),
            view_blocked: HashSet::new(),
            tile_content: vec![Vec::new(); map_tile_count],
        };

        const OFFSET_PERCENT: i32 = 10;
        const TWICE_OFFSET: i32 = OFFSET_PERCENT * 2;
        let mut rng = rltk::RandomNumberGenerator::new();

        for idx in 0..map.colour_offset.len() {
            let red_roll: f32 = (rng.roll_dice(1, TWICE_OFFSET - 1) + 1 - OFFSET_PERCENT) as f32 / 100f32 + 1.0;
            let green_roll: f32 = (rng.roll_dice(1, TWICE_OFFSET - 1) + 1 - OFFSET_PERCENT) as f32 / 100f32 + 1.0;
            let blue_roll: f32 = (rng.roll_dice(1, TWICE_OFFSET - 1) + 1 - OFFSET_PERCENT) as f32 / 100f32 + 1.0;
            map.colour_offset[idx] = (red_roll, green_roll, blue_roll);
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
            self.blocked[i] = !tile_walkable(*tile);
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
        if idx_u > 0 && idx_u < self.tiles.len() {
            return tile_opaque(self.tiles[idx_u]) || self.view_blocked.contains(&idx_u);
        } else {
            return true;
        }
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
        let tt = self.tiles[idx as usize];

        // Cardinal directions
        if self.is_exit_valid(x - 1, y) {
            exits.push((idx - 1, tile_cost(tt)));
        }
        if self.is_exit_valid(x + 1, y) {
            exits.push((idx + 1, tile_cost(tt)));
        }
        if self.is_exit_valid(x, y - 1) {
            exits.push((idx - w, tile_cost(tt)));
        }
        if self.is_exit_valid(x, y + 1) {
            exits.push((idx + w, tile_cost(tt)));
        }

        // Diagonals
        if self.is_exit_valid(x - 1, y - 1) {
            exits.push((idx - w - 1, tile_cost(tt) * 1.45));
        }
        if self.is_exit_valid(x + 1, y - 1) {
            exits.push((idx - w + 1, tile_cost(tt) * 1.45));
        }
        if self.is_exit_valid(x - 1, y + 1) {
            exits.push((idx + w - 1, tile_cost(tt) * 1.45));
        }
        if self.is_exit_valid(x + 1, y + 1) {
            exits.push((idx + w + 1, tile_cost(tt) * 1.45));
        }

        exits
    }
}
