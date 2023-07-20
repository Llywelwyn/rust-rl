use super::{spawner, Map, Position, Rect, TileType, SHOW_MAPGEN};
mod bsp_dungeon;
mod bsp_interior;
mod cellular_automata;
mod common;
mod dla;
mod drunkard;
mod maze;
mod simple_map;
mod voronoi;
mod wfc;
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

#[rustfmt::skip]
pub fn random_builder(new_depth: i32) -> Box<dyn MapBuilder> {
    let mut rng = rltk::RandomNumberGenerator::new();
    let builder = rng.roll_dice(1, 17);
    let mut result : Box<dyn MapBuilder>;
    match builder {
        1 => { result = Box::new(bsp_dungeon::BspDungeonBuilder::new(new_depth)); }
        2 => { result = Box::new(bsp_interior::BspInteriorBuilder::new(new_depth)); }
        3 => { result = Box::new(cellular_automata::CellularAutomataBuilder::new(new_depth)); }
        4 => { result = Box::new(drunkard::DrunkardsWalkBuilder::open_area(new_depth)); }
        5 => { result = Box::new(drunkard::DrunkardsWalkBuilder::open_halls(new_depth)); }
        6 => { result = Box::new(drunkard::DrunkardsWalkBuilder::winding_passages(new_depth)); }
        7 => { result = Box::new(drunkard::DrunkardsWalkBuilder::fat_passages(new_depth)); }
        8 => { result = Box::new(drunkard::DrunkardsWalkBuilder::fearful_symmetry(new_depth)); }
        9 => { result = Box::new(maze::MazeBuilder::new(new_depth)); }
        10 => { result = Box::new(dla::DLABuilder::walk_inwards(new_depth)); }
        11 => { result = Box::new(dla::DLABuilder::walk_outwards(new_depth)); }
        12 => { result = Box::new(dla::DLABuilder::central_attractor(new_depth)); }
        13 => { result = Box::new(dla::DLABuilder::insectoid(new_depth)); }
        14 => { result = Box::new(voronoi::VoronoiBuilder::pythagoras(new_depth)); }
        15 => { result = Box::new(voronoi::VoronoiBuilder::manhattan(new_depth)); }
        16 => { result = Box::new(wfc::WaveFunctionCollapseBuilder::test_map(new_depth)); }
        _ => { result = Box::new(simple_map::SimpleMapBuilder::new(new_depth)); }
    }

    if rng.roll_dice(1, 3)==1 {
        result = Box::new(wfc::WaveFunctionCollapseBuilder::derived_map(new_depth, result));
    }

    result
}
