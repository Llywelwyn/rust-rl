use super::{
    apply_horizontal_tunnel, apply_room_to_map, apply_vertical_tunnel, spawner, Map, MapBuilder, Position, Rect,
    TileType,
};
use rltk::RandomNumberGenerator;
use specs::prelude::*;

pub struct SimpleMapBuilder {}

impl MapBuilder for SimpleMapBuilder {
    fn build(rng: &mut RandomNumberGenerator, new_depth: i32) -> (Map, Position) {
        let mut map = Map::new(new_depth);
        let player_pos = SimpleMapBuilder::rooms_and_corridors(rng, &mut map);

        return (map, player_pos);
    }
}

impl SimpleMapBuilder {
    fn rooms_and_corridors(rng: &mut RandomNumberGenerator, map: &mut Map) -> Position {
        const MAX_ROOMS: i32 = 30;
        const MIN_SIZE: i32 = 6;
        const MAX_SIZE: i32 = 10;
        const MAX_OFFSET: u8 = 32;

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

        for _i in 0..MAX_ROOMS {
            let w = rng.range(MIN_SIZE, MAX_SIZE);
            let h = rng.range(MIN_SIZE, MAX_SIZE);
            let x = rng.roll_dice(1, map.width - w - 1) - 1;
            let y = rng.roll_dice(1, map.height - h - 1) - 1;
            let new_room = Rect::new(x, y, w, h);
            let mut ok = true;
            for other_room in map.rooms.iter() {
                if new_room.intersect(other_room) {
                    ok = false
                }
            }
            if ok {
                apply_room_to_map(map, &new_room);

                if !map.rooms.is_empty() {
                    let (new_x, new_y) = new_room.centre();
                    let (prev_x, prev_y) = map.rooms[map.rooms.len() - 1].centre();
                    if rng.range(0, 2) == 1 {
                        apply_horizontal_tunnel(map, prev_x, new_x, prev_y);
                        apply_vertical_tunnel(map, prev_y, new_y, new_x);
                    } else {
                        apply_vertical_tunnel(map, prev_y, new_y, prev_x);
                        apply_horizontal_tunnel(map, prev_x, new_x, new_y);
                    }
                }

                map.rooms.push(new_room);
            }
        }

        let stairs_position = map.rooms[map.rooms.len() - 1].centre();
        let stairs_idx = map.xy_idx(stairs_position.0, stairs_position.1);
        map.tiles[stairs_idx] = TileType::DownStair;

        let start_pos = map.rooms[0].centre();
        return Position { x: start_pos.0, y: start_pos.1 };
    }
}
