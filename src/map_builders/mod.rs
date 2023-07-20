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

pub fn random_builder(new_depth: i32) -> Box<dyn MapBuilder> {
    /*let mut rng = rltk::RandomNumberGenerator::new();
    let builder = rng.roll_dice(1, 17);
    match builder {
        1 => Box::new(bsp_dungeon::BspDungeonBuilder::new(new_depth)),
        2 => Box::new(bsp_interior::BspInteriorBuilder::new(new_depth)),
        3 => Box::new(cellular_automata::CellularAutomataBuilder::new(new_depth)),
        4 => Box::new(drunkard::DrunkardsWalkBuilder::open_area(new_depth)),
        5 => Box::new(drunkard::DrunkardsWalkBuilder::open_halls(new_depth)),
        6 => Box::new(drunkard::DrunkardsWalkBuilder::winding_passages(new_depth)),
        6 => Box::new(drunkard::DrunkardsWalkBuilder::fat_passages(new_depth)),
        6 => Box::new(drunkard::DrunkardsWalkBuilder::fearful_symmetry(new_depth)),
        7 => Box::new(maze::MazeBuilder::new(new_depth)),
        8 => Box::new(dla::DLABuilder::walk_inwards(new_depth)),
        9 => Box::new(dla::DLABuilder::walk_outwards(new_depth)),
        10 => Box::new(dla::DLABuilder::central_attractor(new_depth)),
        11 => Box::new(dla::DLABuilder::insectoid(new_depth)),
        12 => Box::new(voronoi::VoronoiBuilder::pythagoras(new_depth)),
        12 => Box::new(voronoi::VoronoiBuilder::manhattan(new_depth)),
        12 => Box::new(voronoi::VoronoiBuilder::chebyshev(new_depth)),
        _ => Box::new(simple_map::SimpleMapBuilder::new(new_depth)),
    }*/
    Box::new(wfc::WaveFunctionCollapseBuilder::new(new_depth))
}
