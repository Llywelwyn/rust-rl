use super::{spawner, Map, Position, Rect, TileType};
mod simple_map;
use simple_map::SimpleMapBuilder;
mod common;
use common::*;
use rltk::RandomNumberGenerator;
use specs::prelude::*;

trait MapBuilder {
    fn build(rng: &mut RandomNumberGenerator, new_depth: i32) -> (Map, Position);
}

pub fn build_random_map(rng: &mut RandomNumberGenerator, new_depth: i32) -> (Map, Position) {
    SimpleMapBuilder::build(rng, new_depth)
}
