use super::{Map, Rect, TileType};
use std::cmp::{max, min};
use std::collections::HashMap;

pub fn apply_room_to_map(map: &mut Map, room: &Rect) {
    for y in room.y1 + 1..=room.y2 {
        for x in room.x1 + 1..=room.x2 {
            let idx = map.xy_idx(x, y);
            map.tiles[idx] = TileType::Floor;
        }
    }
}

pub fn apply_horizontal_tunnel(map: &mut Map, x1: i32, x2: i32, y: i32) {
    for x in min(x1, x2)..=max(x1, x2) {
        let idx = map.xy_idx(x, y);
        if idx > 0 && idx < (map.width as usize) * (map.height as usize) {
            map.tiles[idx as usize] = TileType::Floor;
        }
    }
}

pub fn apply_vertical_tunnel(map: &mut Map, y1: i32, y2: i32, x: i32) {
    for y in min(y1, y2)..=max(y1, y2) {
        let idx = map.xy_idx(x, y);
        if idx > 0 && idx < (map.width as usize) * (map.height as usize) {
            map.tiles[idx as usize] = TileType::Floor;
        }
    }
}

pub fn remove_unreachable_areas_returning_most_distant(map: &mut Map, start_idx: usize) -> usize {
    map.populate_blocked();
    let map_starts: Vec<usize> = vec![start_idx];
    let dijkstra_map = rltk::DijkstraMap::new(map.width as usize, map.height as usize, &map_starts, map, 200.0);
    let mut exit_tile = (0, 0.0f32);
    for (i, tile) in map.tiles.iter_mut().enumerate() {
        if *tile == TileType::Floor {
            let distance_to_start = dijkstra_map.map[i];
            // We can't get to this tile - so we'll make it a wall
            if distance_to_start == std::f32::MAX {
                *tile = TileType::Wall;
            } else {
                // If it is further away than our current exit candidate, move the exit
                if distance_to_start > exit_tile.1 {
                    exit_tile.0 = i;
                    exit_tile.1 = distance_to_start;
                }
            }
        }
    }
    return exit_tile.0;
}

#[allow(clippy::map_entry)]
pub fn generate_voronoi_spawn_regions(map: &Map, rng: &mut rltk::RandomNumberGenerator) -> HashMap<i32, Vec<usize>> {
    let mut noise_areas: HashMap<i32, Vec<usize>> = HashMap::new();
    let mut noise = rltk::FastNoise::seeded(rng.roll_dice(1, 65536) as u64);
    noise.set_noise_type(rltk::NoiseType::Cellular);
    noise.set_frequency(0.08);
    noise.set_cellular_distance_function(rltk::CellularDistanceFunction::Manhattan);

    for y in 1..map.height - 1 {
        for x in 1..map.width - 1 {
            let idx = map.xy_idx(x, y);
            if map.tiles[idx] == TileType::Floor {
                let cell_value_f = noise.get_noise(x as f32, y as f32) * 10240.0;
                let cell_value = cell_value_f as i32;

                if noise_areas.contains_key(&cell_value) {
                    noise_areas.get_mut(&cell_value).unwrap().push(idx);
                } else {
                    noise_areas.insert(cell_value, vec![idx]);
                }
            }
        }
    }
    return noise_areas;
}

#[allow(dead_code)]
#[derive(PartialEq, Copy, Clone)]
pub enum Symmetry {
    None,
    Horizontal,
    Vertical,
    Both,
}

pub fn paint(map: &mut Map, mode: Symmetry, brush_size: i32, x: i32, y: i32) {
    match mode {
        Symmetry::None => apply_paint(map, brush_size, x, y),
        Symmetry::Horizontal => {
            let centre_x = map.width / 2;
            if x == centre_x {
                apply_paint(map, brush_size, x, y);
            } else {
                let dist_x = i32::abs(centre_x - x);
                apply_paint(map, brush_size, centre_x + dist_x, y);
                apply_paint(map, brush_size, centre_x - dist_x, y);
            }
        }
        Symmetry::Vertical => {
            let centre_y = map.height / 2;
            if y == centre_y {
                apply_paint(map, brush_size, x, y);
            } else {
                let dist_y = i32::abs(centre_y - y);
                apply_paint(map, brush_size, x, centre_y + dist_y);
                apply_paint(map, brush_size, x, centre_y - dist_y);
            }
        }
        Symmetry::Both => {
            let centre_x = map.width / 2;
            let centre_y = map.height / 2;
            if x == centre_x && y == centre_y {
                apply_paint(map, brush_size, x, y);
            } else {
                let dist_x = i32::abs(centre_x - x);
                apply_paint(map, brush_size, centre_x + dist_x, y);
                apply_paint(map, brush_size, centre_x - dist_x, y);
                let dist_y = i32::abs(centre_y - y);
                apply_paint(map, brush_size, x, centre_y + dist_y);
                apply_paint(map, brush_size, x, centre_y - dist_y);
            }
        }
    }
}

fn apply_paint(map: &mut Map, brush_size: i32, x: i32, y: i32) {
    match brush_size {
        1 => {
            let digger_idx = map.xy_idx(x, y);
            map.tiles[digger_idx] = TileType::Floor;
        }
        _ => {
            let half_brush_size = brush_size / 2;
            for brush_y in y - half_brush_size..y + half_brush_size {
                for brush_x in x - half_brush_size..x + half_brush_size {
                    if brush_x > 1 && brush_x < map.width - 1 && brush_y > 1 && brush_y < map.height - 1 {
                        let idx = map.xy_idx(brush_x, brush_y);
                        map.tiles[idx] = TileType::Floor;
                    }
                }
            }
        }
    }
}
