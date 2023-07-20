use super::{spawner, Map, Position, Rect, TileType, SHOW_MAPGEN};
mod bsp_dungeon;
use bsp_dungeon::BspDungeonBuilder;
mod bsp_interior;
use bsp_interior::BspInteriorBuilder;
mod cellular_automata;
use cellular_automata::CellularAutomataBuilder;
mod common;
mod dla;
use dla::DLABuilder;
mod drunkard;
use drunkard::DrunkardsWalkBuilder;
mod maze;
use maze::MazeBuilder;
mod simple_map;
mod voronoi;
use voronoi::VoronoiBuilder;
mod prefab_builder;
mod wfc;
use common::*;
use rltk::RandomNumberGenerator;
use specs::prelude::*;
use wfc::WaveFunctionCollapseBuilder;

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
    /*let mut rng = rltk::RandomNumberGenerator::new();
    let builder = rng.roll_dice(1, 16);
    let mut result : Box<dyn MapBuilder>;
    match builder {
        1 => { result = Box::new(BspDungeonBuilder::new(new_depth)); }
        2 => { result = Box::new(BspInteriorBuilder::new(new_depth)); }
        3 => { result = Box::new(CellularAutomataBuilder::new(new_depth)); }
        4 => { result = Box::new(DrunkardsWalkBuilder::open_area(new_depth)); }
        5 => { result = Box::new(DrunkardsWalkBuilder::open_halls(new_depth)); }
        6 => { result = Box::new(DrunkardsWalkBuilder::winding_passages(new_depth)); }
        7 => { result = Box::new(DrunkardsWalkBuilder::fat_passages(new_depth)); }
        8 => { result = Box::new(DrunkardsWalkBuilder::fearful_symmetry(new_depth)); }
        9 => { result = Box::new(MazeBuilder::new(new_depth)); }
        10 => { result = Box::new(DLABuilder::walk_inwards(new_depth)); }
        11 => { result = Box::new(DLABuilder::walk_outwards(new_depth)); }
        12 => { result = Box::new(DLABuilder::central_attractor(new_depth)); }
        13 => { result = Box::new(DLABuilder::insectoid(new_depth)); }
        14 => { result = Box::new(VoronoiBuilder::pythagoras(new_depth)); }
        15 => { result = Box::new(VoronoiBuilder::manhattan(new_depth)); }
        _ => { result = Box::new(simple_map::SimpleMapBuilder::new(new_depth)); }
    }

    if rng.roll_dice(1, 3)==1 {
        result = Box::new(wfc::WaveFunctionCollapseBuilder::derived_map(new_depth, result));
    }

    result*/
    Box::new(prefab_builder::PrefabBuilder::new(new_depth))
    
}
