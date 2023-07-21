use super::{spawner, Map, MapBuilder, Position, Rect, TileType, SHOW_MAPGEN};
use rltk::RandomNumberGenerator;
use specs::prelude::*;

pub struct BspInteriorBuilder {
    map: Map,
    starting_position: Position,
    depth: i32,
    rooms: Vec<Rect>,
    history: Vec<Map>,
    rects: Vec<Rect>,
    spawn_list: Vec<(usize, String)>,
}

impl MapBuilder for BspInteriorBuilder {
    fn build_map(&mut self, rng: &mut RandomNumberGenerator) {
        return self.build(rng);
    }
    //  Getters
    fn get_map(&mut self) -> Map {
        return self.map.clone();
    }
    fn get_starting_pos(&mut self) -> Position {
        return self.starting_position.clone();
    }
    fn get_spawn_list(&self) -> &Vec<(usize, String)> {
        return &self.spawn_list;
    }
    // Mapgen visualisation stuff
    fn get_snapshot_history(&self) -> Vec<Map> {
        return self.history.clone();
    }
    fn take_snapshot(&mut self) {
        if SHOW_MAPGEN {
            let mut snapshot = self.map.clone();
            for v in snapshot.revealed_tiles.iter_mut() {
                *v = true;
            }
            self.history.push(snapshot);
        }
    }
}

impl BspInteriorBuilder {
    pub fn new(new_depth: i32) -> BspInteriorBuilder {
        BspInteriorBuilder {
            map: Map::new(new_depth),
            starting_position: Position { x: 0, y: 0 },
            depth: new_depth,
            rooms: Vec::new(),
            history: Vec::new(),
            rects: Vec::new(),
            spawn_list: Vec::new(),
        }
    }

    fn build(&mut self, rng: &mut RandomNumberGenerator) {
        self.rects.clear();
        self.rects.push(Rect::new(1, 1, self.map.width - 2, self.map.height - 2)); // Start with a single map-sized rectangle
        let first_room = self.rects[0];
        self.add_subrects(first_room, rng); // Divide the first room

        let rooms = self.rects.clone();
        for r in rooms.iter() {
            let room = *r;
            self.rooms.push(room);
            for y in room.y1..room.y2 {
                for x in room.x1..room.x2 {
                    let idx = self.map.xy_idx(x, y);
                    if idx > 0 && idx < ((self.map.width * self.map.height) - 1) as usize {
                        self.map.tiles[idx] = TileType::Floor;
                    }
                }
            }
            self.take_snapshot();
        }

        let start = self.rooms[0].centre();
        self.starting_position = Position { x: start.0, y: start.1 };

        // Now we want corridors
        for i in 0..self.rooms.len() - 1 {
            let room = self.rooms[i];
            let next_room = self.rooms[i + 1];
            let start_x = room.x1 + (rng.roll_dice(1, i32::abs(room.x1 - room.x2)) - 1);
            let start_y = room.y1 + (rng.roll_dice(1, i32::abs(room.y1 - room.y2)) - 1);
            let end_x = next_room.x1 + (rng.roll_dice(1, i32::abs(next_room.x1 - next_room.x2)) - 1);
            let end_y = next_room.y1 + (rng.roll_dice(1, i32::abs(next_room.y1 - next_room.y2)) - 1);
            self.draw_corridor(start_x, start_y, end_x, end_y);
            self.take_snapshot();
        }

        // Don't forget the stairs
        let stairs = self.rooms[self.rooms.len() - 1].centre();
        let stairs_idx = self.map.xy_idx(stairs.0, stairs.1);
        self.map.tiles[stairs_idx] = TileType::DownStair;

        // Spawn entities
        for room in self.rooms.iter().skip(1) {
            spawner::spawn_room(&self.map, rng, room, self.depth, &mut self.spawn_list);
        }
    }

    fn add_subrects(&mut self, rect: Rect, rng: &mut RandomNumberGenerator) {
        const MIN_ROOM_SIZE: i32 = 6;
        // Remove last rect
        if !self.rects.is_empty() {
            self.rects.remove(self.rects.len() - 1);
        }

        // Calc bounds
        let w = rect.x2 - rect.x1;
        let h = rect.y2 - rect.y1;
        let half_w = w / 2;
        let half_h = h / 2;

        let split = rng.roll_dice(1, 4);

        if split <= 2 {
            // Horizontal split
            let h1 = Rect::new(rect.x1, rect.y1, half_w - 1, h);
            self.rects.push(h1);
            if half_w > MIN_ROOM_SIZE {
                self.add_subrects(h1, rng);
            }
            let h2 = Rect::new(rect.x1 + half_w, rect.y1, half_w, h);
            self.rects.push(h2);
            if half_w > MIN_ROOM_SIZE {
                self.add_subrects(h2, rng);
            }
        } else {
            // Vertical split
            let v1 = Rect::new(rect.x1, rect.y1, w, half_h - 1);
            self.rects.push(v1);
            if half_h > MIN_ROOM_SIZE {
                self.add_subrects(v1, rng);
            }
            let v2 = Rect::new(rect.x1, rect.y1 + half_h, w, half_h);
            self.rects.push(v2);
            if half_h > MIN_ROOM_SIZE {
                self.add_subrects(v2, rng);
            }
        }
    }

    fn draw_corridor(&mut self, x1: i32, y1: i32, x2: i32, y2: i32) {
        let mut x = x1;
        let mut y = y1;

        while x != x2 || y != y2 {
            if x < x2 {
                x += 1;
            } else if x > x2 {
                x -= 1;
            } else if y < y2 {
                y += 1;
            } else if y > y2 {
                y -= 1;
            }

            let idx = self.map.xy_idx(x, y);
            self.map.tiles[idx] = TileType::Floor;
        }
    }
}
