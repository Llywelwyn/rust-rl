use super::{ BuilderMap, MetaMapBuilder, TileType };
use bracket_lib::prelude::*;

pub struct Foliage {
    start_tile: TileType,
    percent: i32,
}

impl MetaMapBuilder for Foliage {
    #[allow(dead_code)]
    fn build_map(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        self.apply(rng, build_data);
    }
}

impl Foliage {
    #[allow(dead_code)]
    pub fn percent(start_tile: TileType, percent: i32) -> Box<Foliage> {
        return Box::new(Foliage { start_tile, percent });
    }

    fn apply(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        for tile in build_data.map.tiles.iter_mut() {
            if *tile == self.start_tile {
                if rng.roll_dice(1, 100) < self.percent {
                    match rng.roll_dice(1, 2) {
                        1 => {
                            *tile = TileType::Foliage;
                        }
                        _ => {
                            *tile = TileType::HeavyFoliage;
                        }
                    };
                }
            }
        }
    }
}
