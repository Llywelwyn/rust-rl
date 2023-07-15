use super::{spawner, Map, Position, Rect, TileType};
mod simple_map;
use simple_map::SimpleMapBuilder;
mod common;
use common::*;
use rltk::RandomNumberGenerator;
use specs::prelude::*;

pub trait MapBuilder {
    fn build_map(&mut self, rng: &mut RandomNumberGenerator);
    fn spawn_entities(&mut self, ecs: &mut World);
    fn get_map(&mut self) -> Map;
    fn get_starting_pos(&mut self) -> Position;
}

pub fn random_builder(new_depth: i32) -> Box<dyn MapBuilder> {
    return Box::new(SimpleMapBuilder::new(new_depth));
}
