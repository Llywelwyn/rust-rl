use super::{ BuilderMap, MetaMapBuilder, Rect, TileType };
use bracket_lib::prelude::*;

pub struct RoomDrawer {}

impl MetaMapBuilder for RoomDrawer {
    fn build_map(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        self.build(rng, build_data);
    }
}

impl RoomDrawer {
    #[allow(dead_code)]
    pub fn new() -> Box<RoomDrawer> {
        return Box::new(RoomDrawer {});
    }

    fn rectangle(&mut self, build_data: &mut BuilderMap, room: &Rect) {
        for y in room.y1 + 1..=room.y2 {
            for x in room.x1 + 1..=room.x2 {
                let idx = build_data.map.xy_idx(x, y);
                if idx > 0 && idx < ((build_data.map.width * build_data.map.height - 1) as usize) {
                    build_data.map.tiles[idx] = TileType::Floor;
                }
            }
        }
    }

    fn circle(&mut self, build_data: &mut BuilderMap, room: &Rect) {
        let radius = (i32::min(room.x2 - room.x1, room.y2 - room.y1) as f32) / 2.0;
        let center = room.center();
        let center_pt = Point::new(center.x, center.y);
        for y in room.y1..=room.y2 {
            for x in room.x1..=room.x2 {
                let idx = build_data.map.xy_idx(x, y);
                let distance = DistanceAlg::Pythagoras.distance2d(center_pt, Point::new(x, y));
                if
                    idx > 0 &&
                    idx < ((build_data.map.width * build_data.map.height - 1) as usize) &&
                    distance <= radius
                {
                    build_data.map.tiles[idx] = TileType::Floor;
                }
            }
        }
    }

    fn build(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        let rooms: Vec<Rect>;
        if let Some(rooms_builder) = &build_data.rooms {
            rooms = rooms_builder.clone();
        } else {
            panic!("RoomDrawer require a builder with rooms");
        }

        for room in rooms.iter() {
            let room_type = rng.roll_dice(1, 4);
            match room_type {
                1 => self.circle(build_data, room),
                _ => self.rectangle(build_data, room),
            }
            build_data.take_snapshot();
        }
    }
}
