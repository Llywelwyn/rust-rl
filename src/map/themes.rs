use super::{ Map, Point, TileType };
use crate::data::visuals::*;
use crate::config::CONFIG;
use rltk::prelude::*;
use std::ops::{ Add, Mul };

/// Gets the renderables for a tile, with darkening/offset/post-processing/etc. Passing a val for "debug" will ignore viewshed.
pub fn get_tile_renderables_for_id(
    idx: usize,
    map: &Map,
    other_pos: Option<Point>,
    debug: Option<bool>
) -> (rltk::FontCharType, RGB, RGB) {
    let (glyph, mut fg, mut bg, offsets, bg_main_col) = match map.id {
        3 => get_forest_theme_renderables(idx, map, debug),
        _ => get_default_theme_renderables(idx, map, debug),
    };

    // If one of the colours was left blank, make them the same.
    if fg == RGB::new() {
        fg = bg;
    } else if bg == RGB::new() {
        bg = fg;
    }

    fg = fg.add(map.additional_fg_offset);
    (fg, bg) = apply_colour_offset(fg, bg, map, idx, offsets, bg_main_col);
    if CONFIG.visuals.with_scanlines && WITH_SCANLINES_BRIGHTEN_AMOUNT > 0.0 {
        (fg, bg) = brighten_by(fg, bg, WITH_SCANLINES_BRIGHTEN_AMOUNT);
    }
    bg = apply_bloodstain_if_necessary(bg, map, idx);
    let (mut multiplier, mut nonvisible, mut darken) = (1.0, false, false);
    if !map.visible_tiles[idx] {
        multiplier = if CONFIG.visuals.with_scanlines {
            NON_VISIBLE_MULTIPLIER_IF_SCANLINES
        } else {
            NON_VISIBLE_MULTIPLIER
        };
        nonvisible = true;
    }
    if other_pos.is_some() && WITH_DARKEN_BY_DISTANCE && !nonvisible {
        let distance = darken_by_distance(
            Point::new((idx as i32) % map.width, (idx as i32) / map.width),
            other_pos.unwrap()
        );
        multiplier = distance.clamp(
            if CONFIG.visuals.with_scanlines {
                NON_VISIBLE_MULTIPLIER_IF_SCANLINES
            } else {
                NON_VISIBLE_MULTIPLIER
            },
            1.0
        );
        darken = true;
    }
    if nonvisible || darken {
        (fg, bg) = (fg.mul(multiplier), bg.mul(multiplier));
    }
    return (glyph, fg, bg);
}

#[rustfmt::skip]
pub fn get_default_theme_renderables(idx: usize, map: &Map, debug: Option<bool>) -> (rltk::FontCharType, RGB, RGB, (i32, i32, i32), bool) {
    let glyph: rltk::FontCharType;
    #[allow(unused_assignments)]
    let mut fg: RGB = RGB::new();
    #[allow(unused_assignments)]
    let mut bg: RGB = RGB::new();
    let mut offsets: (i32, i32, i32) = (0, 0, 0);
    let mut bg_main_col = true;

    match map.tiles[idx] {
        TileType::Floor => { glyph = rltk::to_cp437(FLOOR_GLYPH); fg = RGB::named(FLOOR_COLOUR); bg = RGB::named(DEFAULT_BG_COLOUR); offsets = FLOOR_OFFSETS; }
        TileType::WoodFloor => { glyph = rltk::to_cp437(WOOD_FLOOR_GLYPH); bg = RGB::named(WOOD_FLOOR_COLOUR); offsets = WOOD_FLOOR_OFFSETS; }
        TileType::Fence => { glyph = rltk::to_cp437(FENCE_GLYPH); fg = RGB::named(FENCE_FG_COLOUR); bg = RGB::named(FENCE_COLOUR); offsets = FENCE_OFFSETS; }
        TileType::Wall => { let x = idx as i32 % map.width; let y = idx as i32 / map.width; glyph = wall_glyph(&*map, x, y, debug); fg = RGB::named(WALL_COLOUR); bg = RGB::named(DEFAULT_BG_COLOUR); offsets = WALL_OFFSETS; bg_main_col = false; }
        TileType::DownStair => { glyph = rltk::to_cp437(DOWN_STAIR_GLYPH); fg = RGB::named(STAIR_COLOUR); bg = RGB::named(DEFAULT_BG_COLOUR); offsets = STAIR_OFFSETS;}
        TileType::UpStair => { glyph = rltk::to_cp437(UP_STAIR_GLYPH); fg = RGB::named(STAIR_COLOUR); bg = RGB::named(DEFAULT_BG_COLOUR); offsets = STAIR_OFFSETS; }
        TileType::Bridge => { glyph = rltk::to_cp437(BRIDGE_GLYPH); bg = RGB::named(BRIDGE_COLOUR); offsets = BRIDGE_OFFSETS; }
        TileType::Gravel => { glyph = rltk::to_cp437(GRAVEL_GLYPH); bg = RGB::named(GRAVEL_COLOUR); offsets = GRAVEL_OFFSETS;}
        TileType::Road => { glyph = rltk::to_cp437(ROAD_GLYPH); bg = RGB::named(ROAD_COLOUR); offsets = ROAD_OFFSETS;}
        TileType::Grass => { glyph = rltk::to_cp437(GRASS_GLYPH); bg = RGB::named(GRASS_COLOUR); offsets = GRASS_OFFSETS; }
        TileType::Foliage => { glyph = rltk::to_cp437(FOLIAGE_GLYPH); bg = RGB::named(FOLIAGE_COLOUR); offsets = FOLIAGE_OFFSETS; }
        TileType::HeavyFoliage => { glyph = rltk::to_cp437(HEAVY_FOLIAGE_GLYPH); bg = RGB::named(HEAVY_FOLIAGE_COLOUR); offsets = HEAVY_FOLIAGE_OFFSETS; }
        TileType::Sand => { glyph = rltk::to_cp437(SAND_GLYPH); bg = RGB::named(SAND_COLOUR); offsets = SAND_OFFSETS; }
        TileType::ShallowWater => { glyph = rltk::to_cp437(SHALLOW_WATER_GLYPH); bg = RGB::named(SHALLOW_WATER_COLOUR); offsets = SHALLOW_WATER_OFFSETS; }
        TileType::DeepWater => { glyph = rltk::to_cp437(DEEP_WATER_GLYPH); bg = RGB::named(DEEP_WATER_COLOUR); offsets = DEEP_WATER_OFFSETS; }
        TileType::Bars => { glyph = rltk::to_cp437(BARS_GLYPH); fg = RGB::named(BARS_COLOUR); bg = RGB::named(FLOOR_COLOUR); }
        TileType::ImpassableMountain => { glyph = rltk::to_cp437(IMPASSABLE_MOUNTAIN_GLYPH); bg = RGB::named(IMPASSABLE_MOUNTAIN_COLOUR); offsets = IMPASSABLE_MOUNTAIN_OFFSETS }
    }
    return (glyph, fg, bg, offsets, bg_main_col);
}

#[rustfmt::skip]
fn get_forest_theme_renderables(idx:usize, map: &Map, debug: Option<bool>) -> (rltk::FontCharType, RGB, RGB, (i32, i32, i32), bool) {
    let glyph;
    #[allow(unused_assignments)]
    let mut fg = RGB::new();
    #[allow(unused_assignments)]
    let mut bg = RGB::new();
    let mut offsets: (i32, i32, i32) = (0, 0, 0);
    let mut bg_main_col = true;

    match map.tiles[idx] {
        TileType::Wall => { glyph = rltk::to_cp437(FOREST_WALL_GLYPH); fg = RGB::named(FOREST_WALL_COLOUR); bg = RGB::named(GRASS_COLOUR) }
        TileType::Road => { glyph = rltk::to_cp437(ROAD_GLYPH); bg = RGB::named(ROAD_COLOUR); }
        TileType::ShallowWater => { glyph = rltk::to_cp437(SHALLOW_WATER_GLYPH); bg = RGB::named(SHALLOW_WATER_COLOUR); }
        _ => { (glyph, fg, _, offsets, bg_main_col) = get_default_theme_renderables(idx, map, debug); bg = RGB::named(GRASS_COLOUR) }
    }

    (glyph, fg, bg, offsets, bg_main_col)
}

fn is_revealed_and_wall(map: &Map, x: i32, y: i32, debug: Option<bool>) -> bool {
    let idx = map.xy_idx(x, y);
    map.tiles[idx] == TileType::Wall && (if debug.is_none() { map.revealed_tiles[idx] } else { true })
}

fn wall_glyph(map: &Map, x: i32, y: i32, debug: Option<bool>) -> rltk::FontCharType {
    if x < 1 || x > map.width - 2 || y < 1 || y > map.height - (2 as i32) {
        return 35;
    }
    let mut mask: u8 = 0;
    let diagonals_matter: Vec<u8> = vec![7, 11, 13, 14, 15];

    if is_revealed_and_wall(map, x, y - 1, debug) {
        // N
        mask += 1;
    }
    if is_revealed_and_wall(map, x, y + 1, debug) {
        // S
        mask += 2;
    }
    if is_revealed_and_wall(map, x - 1, y, debug) {
        // W
        mask += 4;
    }
    if is_revealed_and_wall(map, x + 1, y, debug) {
        // E
        mask += 8;
    }

    if diagonals_matter.contains(&mask) {
        if is_revealed_and_wall(map, x + 1, y - 1, debug) {
            // Top right
            mask += 16;
        }
        if is_revealed_and_wall(map, x - 1, y - 1, debug) {
            // Top left
            mask += 32;
        }
        if is_revealed_and_wall(map, x + 1, y + 1, debug) {
            // Bottom right
            mask += 64;
        }
        if is_revealed_and_wall(map, x - 1, y + 1, debug) {
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

fn apply_colour_offset(
    mut fg: RGB,
    mut bg: RGB,
    map: &Map,
    idx: usize,
    offset: (i32, i32, i32),
    bg_main_col: bool
) -> (RGB, RGB) {
    let offset_mod = map.colour_offset[idx];
    let fg_offset = (
        (offset.0 as f32) * offset_mod.0.0,
        (offset.1 as f32) * offset_mod.0.1,
        (offset.2 as f32) * offset_mod.0.2,
    );
    fg = add_i32_offsets(fg, fg_offset);
    if bg_main_col {
        let bg_offset = (
            (offset.0 as f32) * offset_mod.1.0,
            (offset.1 as f32) * offset_mod.1.1,
            (offset.2 as f32) * offset_mod.1.2,
        );
        bg = add_i32_offsets(bg, bg_offset);
    }

    return (fg, bg);
}

fn apply_bloodstain_if_necessary(mut bg: RGB, map: &Map, idx: usize) -> RGB {
    if map.bloodstains.contains(&idx) {
        bg = bg.add(RGB::named(BLOODSTAIN_COLOUR));
    }
    return bg;
}

pub fn add_i32_offsets(rgb: RGB, offsets: (f32, f32, f32)) -> RGB {
    let r = rgb.r + (offsets.0 as f32) / 255.0;
    let g = rgb.g + (offsets.1 as f32) / 255.0;
    let b = rgb.b + (offsets.2 as f32) / 255.0;

    return rltk::RGB::from_f32(r, g, b);
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
        ((crate::data::entity::DEFAULT_VIEWSHED_STANDARD as f32) - START_DARKEN_AT_N_TILES);
    let interp_factor = interp_factor.max(0.0).min(1.0); // Clamp [0-1]
    return (
        1.0 -
        interp_factor * (1.0 - (if CONFIG.visuals.with_scanlines { MAX_DARKENING_IF_SCANLINES } else { MAX_DARKENING }))
    );
}

fn brighten_by(mut fg: RGB, mut bg: RGB, amount: f32) -> (RGB, RGB) {
    fg = fg.add(RGB::from_f32(amount, amount, amount));
    bg = bg.add(RGB::from_f32(amount, amount, amount));
    return (fg, bg);
}
