use super::Rect;
use rltk::{Algorithm2D, BaseMap, Point, Rltk, RGB};
use serde::{Deserialize, Serialize};
use specs::prelude::*;
use std::collections::HashSet;
use std::ops::{Add, Mul};

#[derive(PartialEq, Copy, Clone, Serialize, Deserialize)]
pub enum TileType {
    Wall,
    Floor,
    DownStair,
}

pub const MAPWIDTH: usize = 80;
pub const MAPHEIGHT: usize = 43;
pub const MAPCOUNT: usize = MAPHEIGHT * MAPWIDTH;

#[derive(Default, Serialize, Deserialize, Clone)]
pub struct Map {
    pub tiles: Vec<TileType>,
    pub rooms: Vec<Rect>,
    pub width: i32,
    pub height: i32,
    pub revealed_tiles: Vec<bool>,
    pub visible_tiles: Vec<bool>,
    pub lit_tiles: Vec<bool>,
    pub telepath_tiles: Vec<bool>,
    pub red_offset: Vec<u8>,
    pub green_offset: Vec<u8>,
    pub blue_offset: Vec<u8>,
    pub blocked: Vec<bool>,
    pub depth: i32,
    pub bloodstains: HashSet<usize>,

    #[serde(skip_serializing)]
    #[serde(skip_deserializing)]
    pub tile_content: Vec<Vec<Entity>>,
}

impl Map {
    pub fn xy_idx(&self, x: i32, y: i32) -> usize {
        (y as usize) * (self.width as usize) + (x as usize)
    }

    pub fn new(new_depth: i32) -> Map {
        Map {
            tiles: vec![TileType::Wall; MAPCOUNT],
            rooms: Vec::new(),
            width: MAPWIDTH as i32,
            height: MAPHEIGHT as i32,
            revealed_tiles: vec![false; MAPCOUNT],
            visible_tiles: vec![false; MAPCOUNT],
            lit_tiles: vec![true; MAPCOUNT], // NYI: Light sources. Once those exist, we can set this to false.
            telepath_tiles: vec![false; MAPCOUNT],
            red_offset: vec![0; MAPCOUNT],
            green_offset: vec![0; MAPCOUNT],
            blue_offset: vec![0; MAPCOUNT],
            blocked: vec![false; MAPCOUNT],
            depth: new_depth,
            bloodstains: HashSet::new(),
            tile_content: vec![Vec::new(); MAPCOUNT],
        }
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
        self.tiles[idx as usize] == TileType::Wall
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

pub fn draw_map(ecs: &World, ctx: &mut Rltk) {
    let map = ecs.fetch::<Map>();

    let mut y = 0;
    let mut x = 0;
    for (idx, tile) in map.tiles.iter().enumerate() {
        // Get our colour offsets. Credit to Brogue for the inspiration here.
        let offsets = RGB::from_u8(map.red_offset[idx], map.green_offset[idx], map.blue_offset[idx]);
        if map.revealed_tiles[idx] {
            let mut fg = offsets.mul(2.0);
            // Right now, everything always has the same background. It's a
            // very dark green, just to distinguish it slightly from the
            // black that is tiles we've *never* seen.
            let mut bg = offsets.add(RGB::from_u8(26, 45, 45));
            let glyph;
            match tile {
                TileType::Floor => {
                    glyph = rltk::to_cp437('.');
                    fg = fg.add(RGB::from_f32(0.1, 0.8, 0.5));
                }
                TileType::Wall => {
                    glyph = wall_glyph(&*map, x, y);
                    fg = fg.add(RGB::from_f32(0.6, 0.5, 0.25));
                }
                TileType::DownStair => {
                    glyph = rltk::to_cp437('>');
                    fg = RGB::from_f32(0., 1., 1.);
                }
            }
            if map.bloodstains.contains(&idx) {
                bg = bg.add(RGB::from_f32(0.6, 0., 0.));
            }
            if !map.visible_tiles[idx] {
                fg = fg.mul(0.6);
                bg = bg.mul(0.6);
            }
            ctx.set(x, y, fg, bg, glyph);
        }

        // Move the coordinates
        x += 1;
        if x > (MAPWIDTH as i32) - 1 {
            x = 0;
            y += 1;
        }
    }
}

fn is_revealed_and_wall(map: &Map, x: i32, y: i32) -> bool {
    let idx = map.xy_idx(x, y);
    map.tiles[idx] == TileType::Wall && map.revealed_tiles[idx]
}

fn wall_glyph(map: &Map, x: i32, y: i32) -> rltk::FontCharType {
    if x < 1 || x > map.width - 2 || y < 1 || y > map.height - 2 as i32 {
        return 35;
    }
    let mut mask: u8 = 0;

    if is_revealed_and_wall(map, x, y - 1) {
        mask += 1;
    }
    if is_revealed_and_wall(map, x, y + 1) {
        mask += 2;
    }
    if is_revealed_and_wall(map, x - 1, y) {
        mask += 4;
    }
    if is_revealed_and_wall(map, x + 1, y) {
        mask += 8;
    }

    match mask {
        0 => 9,    // Pillar because we can't see neighbors
        1 => 186,  // Wall only to the north
        2 => 186,  // Wall only to the south
        3 => 186,  // Wall to the north and south
        4 => 205,  // Wall only to the west
        5 => 188,  // Wall to the north and west
        6 => 187,  // Wall to the south and west
        7 => 185,  // Wall to the north, south and west
        8 => 205,  // Wall only to the east
        9 => 200,  // Wall to the north and east
        10 => 201, // Wall to the south and east
        11 => 204, // Wall to the north, south and east
        12 => 205, // Wall to the east and west
        13 => 202, // Wall to the east, west, and south
        14 => 203, // Wall to the east, west, and north
        15 => 206, // ╬ Wall on all sides
        _ => 35,   // We missed one?
    }
}
