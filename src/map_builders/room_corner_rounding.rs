use super::{ BuilderMap, MetaMapBuilder, Rect, TileType };
use rltk::RandomNumberGenerator;

pub struct RoomCornerRounder {}

impl MetaMapBuilder for RoomCornerRounder {
    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data: &mut BuilderMap) {
        self.build(rng, build_data);
    }
}

impl RoomCornerRounder {
    #[allow(dead_code)]
    pub fn new() -> Box<RoomCornerRounder> {
        return Box::new(RoomCornerRounder {});
    }

    fn fill_if_corner(&mut self, x: i32, y: i32, build_data: &mut BuilderMap) {
        let w = build_data.map.width;
        let h = build_data.map.height;
        let idx = build_data.map.xy_idx(x, y);
        let mut neighbour_walls = 0;
        if x > 0 && build_data.map.tiles[idx - 1] == TileType::Wall {
            neighbour_walls += 1;
        }
        if y > 0 && build_data.map.tiles[idx - (w as usize)] == TileType::Wall {
            neighbour_walls += 1;
        }
        if x < w - 2 && build_data.map.tiles[idx + 1] == TileType::Wall {
            neighbour_walls += 1;
        }
        if y < h - 2 && build_data.map.tiles[idx + (w as usize)] == TileType::Wall {
            neighbour_walls += 1;
        }
        if neighbour_walls == 2 {
            build_data.map.tiles[idx] = TileType::Wall;
        }
    }

    fn build(&mut self, _rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        let rooms: Vec<Rect>;
        if let Some(rooms_builder) = &build_data.rooms {
            rooms = rooms_builder.clone();
        } else {
            panic!("RoomCornerRounding requires a builder with rooms.");
        }

        for room in rooms.iter() {
            self.fill_if_corner(room.x1 + 1, room.y1 + 1, build_data);
            self.fill_if_corner(room.x2, room.y1 + 1, build_data);
            self.fill_if_corner(room.x1 + 1, room.y2, build_data);
            self.fill_if_corner(room.x2, room.y2, build_data);

            build_data.take_snapshot();
        }
    }
}
