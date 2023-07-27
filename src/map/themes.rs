use super::{Map, TileType};
use rltk::RGB;
use std::ops::{Add, Mul};

pub fn offset(rgb: rltk::RGB, offsets: (f32, f32, f32)) -> RGB {
    let r = rgb.r * offsets.0;
    let g = rgb.g * offsets.1;
    let b = rgb.b * offsets.2;

    return rltk::RGB::from_f32(r, g, b);
}

pub fn get_tile_glyph(idx: usize, map: &Map) -> (rltk::FontCharType, RGB, RGB) {
    let offsets = map.colour_offset[idx];
    let glyph: rltk::FontCharType;
    let mut fg: RGB = RGB::new();
    let mut bg: RGB;

    let default_bg: RGB = RGB::from_u8(39, 67, 67);

    match map.tiles[idx] {
        TileType::Floor => {
            glyph = rltk::to_cp437('.');
            fg = RGB::from_f32(0.1, 0.8, 0.5);
            bg = default_bg;
        }
        TileType::WoodFloor => {
            glyph = rltk::to_cp437('.');
            bg = RGB::from_u8(48, 43, 40);
        }
        TileType::Fence => {
            glyph = rltk::to_cp437('=');
            fg = RGB::from_u8(99, 48, 9);
            bg = RGB::from_u8(48, 43, 33);
        }
        TileType::Wall => {
            let x = idx as i32 % map.width;
            let y = idx as i32 / map.width;
            glyph = wall_glyph(&*map, x, y);
            fg = RGB::from_f32(0.9, 0.75, 0.37);
            bg = default_bg;
        }
        TileType::DownStair => {
            glyph = rltk::to_cp437('>');
            fg = RGB::from_u8(200, 200, 0);
            bg = default_bg;
        }
        TileType::Bridge => {
            glyph = rltk::to_cp437('.');
            bg = RGB::from_u8(59, 49, 43);
        }
        TileType::Gravel => {
            glyph = rltk::to_cp437(';');
            bg = RGB::from_u8(39, 39, 54);
        }
        TileType::Road => {
            glyph = rltk::to_cp437('.');
            //fg = RGB::from_u8(112, 105, 94);
            bg = RGB::from_u8(29, 45, 46);
        }
        TileType::Grass => {
            glyph = rltk::to_cp437('"');
            bg = RGB::from_u8(39, 67, 39);
        }
        TileType::Foliage => {
            glyph = rltk::to_cp437(':');
            bg = RGB::from_u8(35, 62, 36);
        }
        TileType::HeavyFoliage => {
            glyph = rltk::to_cp437(';');
            bg = RGB::from_u8(32, 62, 32);
        }
        TileType::Sand => {
            glyph = rltk::to_cp437('.');
            bg = RGB::from_u8(92, 92, 66);
        }
        TileType::ShallowWater => {
            glyph = rltk::to_cp437('~');
            bg = RGB::from_u8(51, 63, 91);
        }
        TileType::DeepWater => {
            glyph = rltk::to_cp437('~');
            bg = RGB::from_u8(36, 45, 61);
        }
    }
    if map.bloodstains.contains(&idx) {
        bg = bg.add(RGB::from_f32(0.6, 0., 0.));
    }

    // If the foreground hasn't been changed, just add
    // the bg to it. Otherwise, leave it as is.
    if fg == RGB::new() {
        fg = fg.add(bg).add(map.additional_fg_offset).add(map.additional_fg_offset);
    }

    fg = offset(fg, offsets);
    bg = offset(bg, offsets);

    if !map.visible_tiles[idx] {
        fg = fg.mul(0.7);
        bg = bg.mul(0.7);
    }

    return (glyph, fg, bg);
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
        0 => 254,  // ■ (254) square pillar; but maybe ○ (9) looks better
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
        61 => 205,  // NEW cases
        125 => 205, // NEW cases
        189 => 205, // NEW cases
        206 => 205,
        207 => 202,
        222 => 205,
        238 => 205,
        253 => 205,
        254 => 205,
        167 => 186, // NSW, NW, SW
        91 => 186,  // NSE, NE, SE
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
        _ => 35,    // We missed one?
    }
}
