use super::{BuilderMap, MetaMapBuilder, Rect, TileType};
use rltk::RandomNumberGenerator;
use std::collections::HashSet;

pub struct BresenhamCorridors {}

impl MetaMapBuilder for BresenhamCorridors {
    #[allow(dead_code)]
    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data: &mut BuilderMap) {
        self.corridors(rng, build_data);
    }
}

impl BresenhamCorridors {
    #[allow(dead_code)]
    pub fn new() -> Box<BresenhamCorridors> {
        Box::new(BresenhamCorridors {})
    }

    fn corridors(&mut self, _rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        let rooms: Vec<Rect>;
        if let Some(rooms_builder) = &build_data.rooms {
            rooms = rooms_builder.clone();
        } else {
            panic!("BresenhamCorridors require a builder with room structures");
        }

        let mut connected: HashSet<usize> = HashSet::new();
        let mut corridors: Vec<Vec<usize>> = Vec::new();
        for (i, room) in rooms.iter().enumerate() {
            let mut room_distance: Vec<(usize, f32)> = Vec::new();
            let room_centre = room.centre();
            let room_centre_pt = rltk::Point::new(room_centre.0, room_centre.1);
            for (j, other_room) in rooms.iter().enumerate() {
                if i != j && !connected.contains(&j) {
                    let other_centre = other_room.centre();
                    let other_centre_pt = rltk::Point::new(other_centre.0, other_centre.1);
                    let distance = rltk::DistanceAlg::Pythagoras.distance2d(room_centre_pt, other_centre_pt);
                    room_distance.push((j, distance));
                }
            }

            if !room_distance.is_empty() {
                room_distance.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
                let dest_centre = rooms[room_distance[0].0].centre();
                let line = rltk::line2d(
                    rltk::LineAlg::Bresenham,
                    room_centre_pt,
                    rltk::Point::new(dest_centre.0, dest_centre.1),
                );
                let mut corridor = Vec::new();
                for cell in line.iter() {
                    let idx = build_data.map.xy_idx(cell.x, cell.y);
                    build_data.map.tiles[idx] = TileType::Floor;
                    corridor.push(idx);
                }
                corridors.push(corridor);
                connected.insert(i);
                build_data.take_snapshot();
            }
        }
        build_data.corridors = Some(corridors);
    }
}
