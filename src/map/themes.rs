use super::{ Map, Point, TileType };
use crate::config::glyphs::*;
use crate::config::visuals::*;
use rltk::prelude::*;
use std::ops::{ Add, Mul };

const DARKEN_TILES_BY_DISTANCE: bool = true;

pub fn get_tile_renderables_for_id(idx: usize, map: &Map, other_pos: Option<Point>) -> (rltk::FontCharType, RGB, RGB) {
    let (glyph, mut fg, mut bg) = match map.id {
        2 => get_forest_theme_renderables(idx, map),
        _ => get_default_theme_renderables(idx, map),
    };

    // If one of the colours was left blank, make them the same.
    if fg == RGB::new() {
        fg = bg;
    } else if bg == RGB::new() {
        bg = fg;
    }

    fg = fg.add(map.additional_fg_offset);
    (fg, bg) = apply_colour_offset(fg, bg, map, idx);
    bg = apply_bloodstain_if_necessary(bg, map, idx);
    (fg, bg) = darken_if_not_visible(fg, bg, map, idx);
    if other_pos.is_some() && DARKEN_TILES_BY_DISTANCE {
        let distance = darken_by_distance(
            Point::new((idx as i32) % map.width, (idx as i32) / map.width),
            other_pos.unwrap()
        );
        (fg, bg) = (fg.mul(distance), bg.mul(distance));
    }

    return (glyph, fg, bg);
}

#[rustfmt::skip]
pub fn get_default_theme_renderables(idx: usize, map: &Map) -> (rltk::FontCharType, RGB, RGB) {
    let glyph: rltk::FontCharType;
    #[allow(unused_assignments)]
    let mut fg: RGB = RGB::new();
    #[allow(unused_assignments)]
    let mut bg: RGB = RGB::new();

    match map.tiles[idx] {
        TileType::Floor => { glyph = rltk::to_cp437(FLOOR_GLYPH); fg = RGB::named(FLOOR_COLOUR); bg = RGB::named(DEFAULT_BG_COLOUR); }
        TileType::WoodFloor => { glyph = rltk::to_cp437(WOOD_FLOOR_GLYPH); bg = RGB::named(WOOD_FLOOR_COLOUR); }
        TileType::Fence => { glyph = rltk::to_cp437(FENCE_GLYPH); fg = RGB::named(FENCE_FG_COLOUR); bg = RGB::named(FENCE_COLOUR); }
        TileType::Wall => { let x = idx as i32 % map.width; let y = idx as i32 / map.width; glyph = wall_glyph(&*map, x, y); fg = RGB::named(WALL_COLOUR); bg = RGB::named(DEFAULT_BG_COLOUR); }
        TileType::DownStair => { glyph = rltk::to_cp437(DOWN_STAIR_GLYPH); fg = RGB::named(STAIR_COLOUR); bg = RGB::named(DEFAULT_BG_COLOUR); }
        TileType::UpStair => { glyph = rltk::to_cp437(UP_STAIR_GLYPH); fg = RGB::named(STAIR_COLOUR); bg = RGB::named(DEFAULT_BG_COLOUR); }
        TileType::Bridge => { glyph = rltk::to_cp437(BRIDGE_GLYPH); bg = RGB::named(BRIDGE_COLOUR); }
        TileType::Gravel => { glyph = rltk::to_cp437(GRAVEL_GLYPH); bg = RGB::named(GRAVEL_COLOUR); }
        TileType::Road => { glyph = rltk::to_cp437(ROAD_GLYPH); bg = RGB::named(ROAD_COLOUR); }
        TileType::Grass => { glyph = rltk::to_cp437(GRASS_GLYPH); bg = RGB::named(GRASS_COLOUR); }
        TileType::Foliage => { glyph = rltk::to_cp437(FOLIAGE_GLYPH); bg = RGB::named(FOLIAGE_COLOUR); }
        TileType::HeavyFoliage => { glyph = rltk::to_cp437(HEAVY_FOLIAGE_GLYPH); bg = RGB::named(HEAVY_FOLIAGE_COLOUR); }
        TileType::Sand => { glyph = rltk::to_cp437(SAND_GLYPH); bg = RGB::named(SAND_COLOUR); }
        TileType::ShallowWater => { glyph = rltk::to_cp437(SHALLOW_WATER_GLYPH); bg = RGB::named(SHALLOW_WATER_COLOUR); }
        TileType::DeepWater => { glyph = rltk::to_cp437(DEEP_WATER_GLYPH); bg = RGB::named(DEEP_WATER_COLOUR); }
        TileType::Bars => { glyph = rltk::to_cp437(BARS_GLYPH); fg = RGB::named(BARS_COLOUR); bg = RGB::named(FLOOR_COLOUR); }
    }
    return (glyph, fg, bg);
}

#[rustfmt::skip]
fn get_forest_theme_renderables(idx:usize, map: &Map) -> (rltk::FontCharType, RGB, RGB) {
    let glyph;
    #[allow(unused_assignments)]
    let mut fg = RGB::new();
    #[allow(unused_assignments)]
    let mut bg = RGB::new();

    match map.tiles[idx] {
        TileType::Wall => { glyph = rltk::to_cp437(FOREST_WALL_GLYPH); fg = RGB::named(FOREST_WALL_COLOUR); bg = RGB::named(GRASS_COLOUR) }
        TileType::Road => { glyph = rltk::to_cp437(ROAD_GLYPH); bg = RGB::named(ROAD_COLOUR); }
        TileType::ShallowWater => { glyph = rltk::to_cp437(SHALLOW_WATER_GLYPH); bg = RGB::named(SHALLOW_WATER_COLOUR); }
        _ => { (glyph, fg, _) = get_default_theme_renderables(idx, map); bg = RGB::named(GRASS_COLOUR) }
    }

    (glyph, fg, bg)
}

fn is_revealed_and_wall(map: &Map, x: i32, y: i32) -> bool {
    let idx = map.xy_idx(x, y);
    map.tiles[idx] == TileType::Wall && map.revealed_tiles[idx]
}

fn wall_glyph(map: &Map, x: i32, y: i32) -> rltk::FontCharType {
    if x < 1 || x > map.width - 2 || y < 1 || y > map.height - (2 as i32) {
        return 35;
    }
    let mut mask: u8 = 0;
    let diagonals_matter: Vec<u8> = vec![7, 11, 13, 14, 15];

    if is_revealed_and_wall(map, x, y - 1) {
        // N
        mask += 1;
    }
    if is_revealed_and_wall(map, x, y + 1) {
        // S
        mask += 2;
    }
    if is_revealed_and_wall(map, x - 1, y) {
        // W
        mask += 4;
    }
    if is_revealed_and_wall(map, x + 1, y) {
        // E
        mask += 8;
    }

    if diagonals_matter.contains(&mask) {
        if is_revealed_and_wall(map, x + 1, y - 1) {
            // Top right
            mask += 16;
        }
        if is_revealed_and_wall(map, x - 1, y - 1) {
            // Top left
            mask += 32;
        }
        if is_revealed_and_wall(map, x + 1, y + 1) {
            // Bottom right
            mask += 64;
        }
        if is_revealed_and_wall(map, x - 1, y + 1) {
            // Bottom left
            mask += 128;
        }
    }

    match mask {
        0 => 254, // ■ (254) square pillar; but maybe ○ (9) looks better
        1 => 186, // Wall only to the north
        2 => 186, // Wall only to the south
        3 => 186, // Wall to the north and south
        4 => 205, // Wall only to the west
        5 => 188, // Wall to the north and west
        6 => 187, // Wall to the south and west
        7 => 185, // Wall to the north, south and west
        8 => 205, // Wall only to the east
        9 => 200, // Wall to the north and east
        10 => 201, // Wall to the south and east
        11 => 204, // Wall to the north, south and east
        12 => 205, // Wall to the east and west
        13 => 202, // Wall to the east, west, and north
        14 => 203, // Wall to the east, west, and south
        15 => 206, // ╬ Wall on all sides
        29 => 202,
        31 => 206,
        45 => 202,
        46 => 203,
        47 => 206,
        55 => 185,
        59 => 204,
        63 => 203,
        87 => 185,
        126 => 203,
        143 => 206,
        77 => 202,
        171 => 204,
        187 => 204,
        215 => 185,
        190 => 203,
        237 => 202,
        30 => 203,
        110 => 203,
        111 => 206,
        119 => 185,
        142 => 203,
        158 => 203,
        235 => 204,
        93 => 202,
        109 => 202,
        94 => 203,
        174 => 203,
        159 => 206,
        221 => 202,
        157 => 202,
        79 => 206,
        95 => 185,
        23 => 185, // NSW and NSE + 1 diagonal
        39 => 185,
        71 => 185,
        103 => 185,
        135 => 185,
        151 => 185,
        199 => 185,
        78 => 203,
        27 => 204,
        43 => 204,
        75 => 204,
        107 => 204,
        139 => 204,
        155 => 204,
        173 => 202,
        141 => 202,
        205 => 202,
        175 => 204,
        203 => 204,
        61 => 205, // NEW cases
        125 => 205, // NEW cases
        189 => 205, // NEW cases
        206 => 205,
        207 => 202,
        222 => 205,
        238 => 205,
        253 => 205,
        254 => 205,
        167 => 186, // NSW, NW, SW
        91 => 186, // NSE, NE, SE
        183 => 186, // NSW, NW, SW, NE
        123 => 186, // NSE, NE, SE, NW
        231 => 186, // NSW, NW, SW, SE
        219 => 186, // NSE, NE, SE, SW
        247 => 186,
        251 => 186,
        127 => 187, // Everything except NE
        191 => 201, // Everything except NW
        223 => 188, // Everything except SE
        239 => 200, // Everything except SW
        _ => 35, // We missed one?
    }
}

fn apply_colour_offset(mut fg: RGB, mut bg: RGB, map: &Map, idx: usize) -> (RGB, RGB) {
    let offsets = map.colour_offset[idx];
    fg = multiply_by_float(fg.add(map.additional_fg_offset), offsets);
    bg = multiply_by_float(bg, offsets);
    return (fg, bg);
}

fn darken_if_not_visible(mut fg: RGB, mut bg: RGB, map: &Map, idx: usize) -> (RGB, RGB) {
    if !map.visible_tiles[idx] {
        fg = fg.mul(NON_VISIBLE_MULTIPLIER);
        bg = bg.mul(NON_VISIBLE_MULTIPLIER);
    }
    return (fg, bg);
}

fn apply_bloodstain_if_necessary(mut bg: RGB, map: &Map, idx: usize) -> RGB {
    if map.bloodstains.contains(&idx) {
        bg = bg.add(RGB::named(BLOODSTAIN_COLOUR));
    }
    return bg;
}

pub fn multiply_by_float(rgb: rltk::RGB, offsets: (f32, f32, f32)) -> RGB {
    let r = rgb.r * offsets.0;
    let g = rgb.g * offsets.1;
    let b = rgb.b * offsets.2;

    return rltk::RGB::from_f32(r, g, b);
}

fn darken_by_distance(pos: Point, other_pos: Point) -> f32 {
    let distance = DistanceAlg::Pythagoras.distance2d(pos, other_pos) as f32; // Get distance in tiles.
    let interp_factor =
        (distance - START_DARKEN_AT_N_TILES) /
        (MAX_DARKEN_AT_N_TILES * (crate::config::entity::DEFAULT_VIEWSHED_STANDARD as f32) - START_DARKEN_AT_N_TILES);
    let interp_factor = interp_factor.max(0.0).min(1.0); // Clamp [0-1]
    return 1.0 - interp_factor * (1.0 - MAX_DARKENING);
}
