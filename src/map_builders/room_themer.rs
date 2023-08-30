use super::{ BuilderMap, MetaMapBuilder, Rect, TileType };
use crate::tile_walkable;
use crate::data::messages::{ FEATURE_TREANTS, FEATURE_BARRACKS_GOBLIN, FEATURE_BARRACKS_KOBOLD, FEATURE_BARRACKS_ORC };
use crate::raws;
use rltk::RandomNumberGenerator;

pub enum Theme {
    Grass,
    Barrack,
}

pub struct ThemeRooms {
    pub theme: Theme,
    pub percent: i32,
}

impl MetaMapBuilder for ThemeRooms {
    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data: &mut BuilderMap) {
        self.build(rng, build_data);
    }
}

impl ThemeRooms {
    #[allow(dead_code)]
    pub fn grass(percent: i32) -> Box<ThemeRooms> {
        return Box::new(ThemeRooms { theme: Theme::Grass, percent });
    }
    pub fn barracks(percent: i32) -> Box<ThemeRooms> {
        return Box::new(ThemeRooms { theme: Theme::Barrack, percent });
    }

    fn grassify(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap, room: &Rect) {
        let (var_x, var_y) = (rng.roll_dice(1, 3), rng.roll_dice(1, 3));
        let x1 = if room.x1 - var_x > 0 { room.x1 - var_x } else { room.x1 };
        let x2 = if room.x2 + var_x < build_data.map.width - 1 { room.x2 + var_x } else { room.x2 };
        let y1 = if room.y1 - var_y > 0 { room.y1 - var_y } else { room.y1 };
        let y2 = if room.y2 + var_y < build_data.map.height - 1 { room.y2 + var_y } else { room.y2 };
        for x in x1..x2 {
            for y in y1..y2 {
                let idx = build_data.map.xy_idx(x, y);
                if tile_walkable(build_data.map.tiles[idx]) && build_data.map.tiles[idx] != TileType::DownStair {
                    let tar = if x < room.x1 || x > room.x2 || y < room.y1 || y > room.y2 { 45 } else { 90 };
                    let roll = rng.roll_dice(1, 100);
                    if roll <= tar {
                        match rng.roll_dice(1, 6) {
                            1..=4 => {
                                build_data.map.tiles[idx] = TileType::Grass;
                            }
                            5 => {
                                build_data.map.tiles[idx] = TileType::Foliage;
                            }
                            _ => {
                                build_data.map.tiles[idx] = TileType::HeavyFoliage;
                            }
                        }
                        if roll < 5 {
                            build_data.spawn_list.push((idx, "treant_small".to_string()));
                        }
                    }
                }
            }
        }
        build_data.map.messages.insert(FEATURE_TREANTS.to_string());
    }

    fn place_barracks(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap, room: &Rect) {
        let mut possible: Vec<usize> = Vec::new();
        let (x1, x2, y1, y2) = (room.x1 + 1, room.x2 - 1, room.y1 + 1, room.y2 - 1);
        for x in x1..x2 {
            for y in y1..y2 {
                let idx = build_data.map.xy_idx(x, y);
                if tile_walkable(build_data.map.tiles[idx]) && build_data.map.tiles[idx] != TileType::DownStair {
                    possible.push(idx);
                }
            }
        }

        let mut needs_captain = if rng.roll_dice(1, 3) == 1 { false } else { true };
        let (captain, squad) = match rng.roll_dice(1, 4) {
            1 => {
                build_data.map.messages.insert(FEATURE_BARRACKS_GOBLIN.to_string());
                ("goblin_chieftain", "squad_goblin")
            }
            2 => {
                build_data.map.messages.insert(FEATURE_BARRACKS_KOBOLD.to_string());
                ("kobold_captain", "squad_kobold")
            }
            _ => {
                build_data.map.messages.insert(FEATURE_BARRACKS_ORC.to_string());
                ("orc_captain", "squad_orc")
            }
        };
        for idx in possible {
            if idx % 2 == 0 && rng.roll_dice(1, 2) == 1 {
                build_data.spawn_list.push((idx, "prop_bed".to_string()));
            } else if rng.roll_dice(1, 5) == 1 {
                let mob = if needs_captain {
                    captain.to_string()
                } else {
                    raws::table_by_name(&raws::RAWS.lock().unwrap(), squad, None).roll(rng)
                };
                needs_captain = false;
                build_data.spawn_list.push((idx, mob));
            }
        }
    }

    fn build(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        let rooms: Vec<Rect>;
        if let Some(rooms_builder) = &build_data.rooms {
            rooms = rooms_builder.clone();
        } else {
            panic!("RoomCornerRounding requires a builder with rooms.");
        }

        for room in rooms.iter() {
            if rng.roll_dice(1, 100) < self.percent {
                match self.theme {
                    Theme::Grass => self.grassify(rng, build_data, room),
                    Theme::Barrack => self.place_barracks(rng, build_data, room),
                    _ => {}
                }
                build_data.take_snapshot();
            }
        }
    }
}
