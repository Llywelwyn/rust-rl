use bracket_lib::prelude::*;
use serde::{ Deserialize, Serialize };
use std::collections::{ HashSet, HashMap };
mod tiletype;
pub use tiletype::{
    tile_cost,
    tile_opaque,
    tile_walkable,
    tile_blocks_telepathy,
    TileType,
    get_dest,
    Destination,
};
mod interval_spawning_system;
pub use interval_spawning_system::{ maybe_map_message, try_spawn_interval };
pub mod dungeon;
pub use dungeon::{ level_transition, MasterDungeonMap };
pub mod themes;
use super::consts::visuals::{
    BRIGHTEN_FG_COLOUR_BY,
    GLOBAL_OFFSET_MIN_CLAMP,
    GLOBAL_OFFSET_MAX_CLAMP,
};

// FIXME: If the map size gets too small, entities stop being rendered starting from the right.
// i.e. on a map size of 40*40, only entities to the left of the player are rendered.
//      on a map size of 42*42, the player can see entities up to 2 tiles to their right.

#[derive(Default, Serialize, Deserialize, Clone)]
pub struct Map {
    pub overmap: bool,
    pub tiles: Vec<TileType>,
    pub width: i32,
    pub height: i32,
    pub revealed_tiles: Vec<bool>,
    pub visible_tiles: Vec<bool>,
    pub lit_tiles: Vec<bool>,
    pub telepath_tiles: Vec<bool>,
    pub colour_offset: Vec<((f32, f32, f32), (f32, f32, f32))>,
    pub additional_fg_offset: RGB,
    pub id: i32,
    pub name: String,
    pub short_name: String,
    pub depth: i32,
    pub messages: HashSet<String>,
    pub difficulty: i32,
    pub bloodstains: HashMap<usize, RGB>,
    pub view_blocked: HashSet<usize>,
}

impl Map {
    pub fn xy_idx(&self, x: i32, y: i32) -> usize {
        (y as usize) * (self.width as usize) + (x as usize)
    }

    pub fn new<S: ToString>(
        overmap: bool,
        new_id: i32,
        width: i32,
        height: i32,
        difficulty: i32,
        name: S,
        short_name: S,
        depth: i32
    ) -> Map {
        let map_tile_count = (width * height) as usize;
        crate::spatial::set_size(map_tile_count);
        let mut map = Map {
            overmap: overmap,
            tiles: vec![TileType::Wall; map_tile_count],
            width: width,
            height: height,
            revealed_tiles: vec![false; map_tile_count],
            visible_tiles: vec![false; map_tile_count],
            lit_tiles: vec![true; map_tile_count], // NYI: Light sources. Once those exist, we can set this to false.
            telepath_tiles: vec![false; map_tile_count],
            colour_offset: vec![((0.0, 0.0, 0.0), (0.0, 0.0, 0.0)); map_tile_count],
            additional_fg_offset: RGB::from_u8(
                BRIGHTEN_FG_COLOUR_BY as u8,
                BRIGHTEN_FG_COLOUR_BY as u8,
                BRIGHTEN_FG_COLOUR_BY as u8
            ),
            id: new_id,
            name: name.to_string(),
            short_name: short_name.to_string(),
            messages: HashSet::new(),
            depth: depth,
            difficulty: difficulty,
            bloodstains: HashMap::new(),
            view_blocked: HashSet::new(),
        };

        let mut rng = RandomNumberGenerator::new();

        for idx in 0..map.colour_offset.len() {
            map.colour_offset[idx].0 = (
                rng.range(GLOBAL_OFFSET_MIN_CLAMP, GLOBAL_OFFSET_MAX_CLAMP),
                rng.range(GLOBAL_OFFSET_MIN_CLAMP, GLOBAL_OFFSET_MAX_CLAMP),
                rng.range(GLOBAL_OFFSET_MIN_CLAMP, GLOBAL_OFFSET_MAX_CLAMP),
            );
            map.colour_offset[idx].1 = (
                rng.range(GLOBAL_OFFSET_MIN_CLAMP, GLOBAL_OFFSET_MAX_CLAMP),
                rng.range(GLOBAL_OFFSET_MIN_CLAMP, GLOBAL_OFFSET_MAX_CLAMP),
                rng.range(GLOBAL_OFFSET_MIN_CLAMP, GLOBAL_OFFSET_MAX_CLAMP),
            );
        }

        return map;
    }

    /// Takes an index, and calculates if it can be entered.
    fn is_exit_valid(&self, x: i32, y: i32) -> bool {
        if x < 1 || x > self.width - 1 || y < 1 || y > self.height - 1 {
            return false;
        }
        let idx = self.xy_idx(x, y);
        return !crate::spatial::is_blocked(idx);
    }

    pub fn populate_blocked(&mut self) {
        crate::spatial::populate_blocked_from_map(self);
    }

    pub fn clear_content_index(&mut self) {
        crate::spatial::clear();
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
        DistanceAlg::Pythagoras.distance2d(p1, p2)
    }

    /// Evaluate every possible exit from a given tile in a cardinal direction, and return it as a vector.
    fn get_available_exits(&self, idx: usize) -> SmallVec<[(usize, f32); 10]> {
        let mut exits = SmallVec::new();
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
