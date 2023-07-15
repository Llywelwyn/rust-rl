use super::{spawner, Map, Position, Rect, TileType, SHOW_MAPGEN};
mod bsp_dungeon;
mod bsp_interior;
mod cellular_automata;
mod common;
mod drunkard;
mod simple_map;
use common::*;
use rltk::RandomNumberGenerator;
use specs::prelude::*;

pub trait MapBuilder {
    fn build_map(&mut self, rng: &mut RandomNumberGenerator);
    fn spawn_entities(&mut self, ecs: &mut World);
    fn get_map(&mut self) -> Map;
    fn get_starting_pos(&mut self) -> Position;
    fn get_snapshot_history(&self) -> Vec<Map>;
    fn take_snapshot(&mut self);
}

pub fn random_builder(new_depth: i32) -> Box<dyn MapBuilder> {
    let mut rng = rltk::RandomNumberGenerator::new();
    let builder = rng.roll_dice(1, 7);
    match builder {
        1 => Box::new(bsp_dungeon::BspDungeonBuilder::new(new_depth)),
        2 => Box::new(bsp_interior::BspInteriorBuilder::new(new_depth)),
        3 => Box::new(cellular_automata::CellularAutomataBuilder::new(new_depth)),
        4 => Box::new(drunkard::DrunkardsWalkBuilder::open_area(new_depth)),
        5 => Box::new(drunkard::DrunkardsWalkBuilder::open_halls(new_depth)),
        6 => Box::new(drunkard::DrunkardsWalkBuilder::winding_passages(new_depth)),
        _ => Box::new(simple_map::SimpleMapBuilder::new(new_depth)),
    }
}
