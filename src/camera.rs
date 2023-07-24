use super::{Door, Hidden, Map, Mind, Position, Renderable, TileType};
use rltk::{Point, Rltk, RGB};
use specs::prelude::*;
use std::ops::{Add, Mul};

const SHOW_BOUNDARIES: bool = true;

pub fn get_screen_bounds(ecs: &World, ctx: &mut Rltk) -> (i32, i32, i32, i32) {
    let player_pos = ecs.fetch::<Point>();
    let (x_chars, y_chars) = ctx.get_char_size();

    let centre_x = (x_chars / 2) as i32;
    let centre_y = (y_chars / 2) as i32;

    let min_x = player_pos.x - centre_x;
    let min_y = player_pos.y - centre_y;
    let max_x = min_x + x_chars as i32;
    let max_y = min_y + y_chars as i32;

    (min_x, max_x, min_y, max_y)
}

pub fn render_camera(ecs: &World, ctx: &mut Rltk) {
    let map = ecs.fetch::<Map>();
    let (min_x, max_x, min_y, max_y) = get_screen_bounds(ecs, ctx);

    // Might need to -1 here?
    let map_width = map.width;
    let map_height = map.height;

    // Render map
    let mut y = 0;
    for t_y in min_y..max_y {
        let mut x = 0;
        for t_x in min_x..max_x {
            if t_x >= 0 && t_x < map.width && t_y >= 0 && t_y < map_height {
                let idx = map.xy_idx(t_x, t_y);
                if map.revealed_tiles[idx] {
                    let (glyph, fg, bg) = get_tile_glyph(idx, &*map);
                    ctx.set(x, y, fg, bg, glyph);
                }
            } else if SHOW_BOUNDARIES {
                ctx.set(x, y, RGB::named(rltk::DARKSLATEGRAY), RGB::named(rltk::BLACK), rltk::to_cp437('#'));
            }
            x += 1;
        }
        y += 1;
    }

    // Render entities
    {
        let positions = ecs.read_storage::<Position>();
        let renderables = ecs.read_storage::<Renderable>();
        let minds = ecs.read_storage::<Mind>();
        let hidden = ecs.read_storage::<Hidden>();
        let doors = ecs.write_storage::<Door>();
        let map = ecs.fetch::<Map>();
        let entities = ecs.entities();

        let mut data = (&positions, &renderables, &entities, !&hidden).join().collect::<Vec<_>>();
        data.sort_by(|&a, &b| b.1.render_order.cmp(&a.1.render_order));
        for (pos, render, ent, _hidden) in data.iter() {
            let idx = map.xy_idx(pos.x, pos.y);
            let entity_offset_x = pos.x - min_x;
            let entity_offset_y = pos.y - min_y;
            if entity_offset_x > 0 && entity_offset_x < map_width && entity_offset_y > 0 && entity_offset_y < map_height
            {
                let offsets = RGB::from_u8(map.red_offset[idx], map.green_offset[idx], map.blue_offset[idx]);
                let mut draw = false;
                let mut fg = render.fg;
                let mut bg = render.bg.add(RGB::from_u8(26, 45, 45)).add(offsets);
                // Get bloodstain colours
                if map.bloodstains.contains(&idx) {
                    bg = bg.add(RGB::from_f32(0.6, 0., 0.));
                }
                // Draw entities on visible tiles
                if map.visible_tiles[idx] {
                    draw = true;
                }
                // Draw entities with minds within telepath range
                if map.telepath_tiles[idx] {
                    let has_mind = minds.get(*ent);
                    if let Some(_) = has_mind {
                        draw = true;
                    }
                }
                // Draw all doors
                let is_door = doors.get(*ent);
                if let Some(_) = is_door {
                    if map.revealed_tiles[idx] {
                        if !map.visible_tiles[idx] {
                            fg = fg.mul(0.6);
                            bg = bg.mul(0.6);
                        }
                        draw = true;
                    }
                }
                if draw {
                    ctx.set(entity_offset_x, entity_offset_y, fg, bg, render.glyph);
                }
            }
        }
    }
}

fn get_tile_glyph(idx: usize, map: &Map) -> (rltk::FontCharType, RGB, RGB) {
    let offsets = RGB::from_u8(map.red_offset[idx], map.green_offset[idx], map.blue_offset[idx]);
    let glyph;
    let mut fg = offsets.mul(2.0);
    let mut bg = offsets.add(RGB::from_u8(26, 45, 45));

    match map.tiles[idx] {
        TileType::Floor => {
            glyph = rltk::to_cp437('.');
            fg = fg.add(RGB::from_f32(0.1, 0.8, 0.5));
        }
        TileType::Wall => {
            let x = idx as i32 % map.width;
            let y = idx as i32 / map.width;
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
