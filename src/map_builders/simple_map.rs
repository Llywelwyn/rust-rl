use super::{ BuilderMap, InitialMapBuilder, Rect };
use rltk::RandomNumberGenerator;

pub struct SimpleMapBuilder {
    room_params: (i32, i32, i32),
}

impl InitialMapBuilder for SimpleMapBuilder {
    #[allow(dead_code)]
    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data: &mut BuilderMap) {
        self.build_rooms(rng, build_data);
    }
}

impl SimpleMapBuilder {
    #[allow(dead_code)]
    pub fn new(room_params: Option<(i32, i32, i32)>) -> Box<SimpleMapBuilder> {
        const DEFAULT_MAX_ROOMS: i32 = 40;
        const DEFAULT_MIN_SIZE: i32 = 6;
        const DEFAULT_MAX_SIZE: i32 = 16;

        let (max_rooms, min_size, max_size);

        if let Some(room_params) = room_params {
            (max_rooms, min_size, max_size) = (room_params.0, room_params.1, room_params.2);
        } else {
            (max_rooms, min_size, max_size) = (
                DEFAULT_MAX_ROOMS,
                DEFAULT_MIN_SIZE,
                DEFAULT_MAX_SIZE,
            );
        }

        Box::new(SimpleMapBuilder { room_params: (max_rooms, min_size, max_size) })
    }

    fn build_rooms(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        let mut rooms: Vec<Rect> = Vec::new();

        for _i in 0..self.room_params.0 {
            let w = rng.range(self.room_params.1, self.room_params.2);
            let h = rng.range(self.room_params.1, self.room_params.2);
            let x = rng.roll_dice(1, build_data.map.width - w - 1) - 1;
            let y = rng.roll_dice(1, build_data.map.height - h - 1) - 1;
            let new_room = Rect::with_size(x, y, w, h);
            let mut ok = true;
            for other_room in rooms.iter() {
                if new_room.intersect(other_room) {
                    ok = false;
                }
            }
            if ok {
                rooms.push(new_room);
            }
        }
        build_data.rooms = Some(rooms);
    }
}
