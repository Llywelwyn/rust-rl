use super::{ spawner, BuilderMap, MetaMapBuilder };
use bracket_lib::prelude::*;

pub struct RoomBasedSpawner {}

impl MetaMapBuilder for RoomBasedSpawner {
    fn build_map(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        self.build(rng, build_data);
    }
}

impl RoomBasedSpawner {
    #[allow(dead_code)]
    pub fn new() -> Box<RoomBasedSpawner> {
        Box::new(RoomBasedSpawner {})
    }

    fn build(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        if let Some(rooms) = &build_data.rooms {
            for room in rooms.iter().skip(1) {
                spawner::spawn_room(
                    &build_data.map,
                    rng,
                    room,
                    &mut build_data.spawn_list,
                    build_data.initial_player_level
                );
            }
        } else {
            unreachable!("RoomBasedSpawner tried to run without any rooms.");
        }
    }
}
