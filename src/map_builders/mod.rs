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
use prefab_builder::PrefabBuilder;
mod room_based_spawner;
mod wfc;
use room_based_spawner::*;
mod room_based_stairs;
use room_based_stairs::*;
mod room_based_starting_position;
use room_based_starting_position::*;
mod area_starting_points;
use area_starting_points::{AreaStartingPosition, XStart, YStart};
mod cull_unreachable;
use cull_unreachable::CullUnreachable;
mod distant_exit;
use distant_exit::DistantExit;
mod voronoi_spawning;
use common::*;
use specs::prelude::*;
use voronoi_spawning::VoronoiSpawning;
use wfc::WaveFunctionCollapseBuilder;

// Shared data to be passed around build chain
pub struct BuilderMap {
    pub spawn_list: Vec<(usize, String)>,
    pub map: Map,
    pub starting_position: Option<Position>,
    pub rooms: Option<Vec<Rect>>,
    pub history: Vec<Map>,
}

impl BuilderMap {
    fn take_snapshot(&mut self) {
        if SHOW_MAPGEN {
            let mut snapshot = self.map.clone();
            for v in snapshot.revealed_tiles.iter_mut() {
                *v = true;
            }
            self.history.push(snapshot);
        }
    }
}

pub struct BuilderChain {
    starter: Option<Box<dyn InitialMapBuilder>>,
    builders: Vec<Box<dyn MetaMapBuilder>>,
    pub build_data: BuilderMap,
}

impl BuilderChain {
    pub fn new(new_depth: i32) -> BuilderChain {
        BuilderChain {
            starter: None,
            builders: Vec::new(),
            build_data: BuilderMap {
                spawn_list: Vec::new(),
                map: Map::new(new_depth),
                starting_position: None,
                rooms: None,
                history: Vec::new(),
            },
        }
    }

    pub fn start_with(&mut self, starter: Box<dyn InitialMapBuilder>) {
        match self.starter {
            None => self.starter = Some(starter),
            Some(_) => panic!("You can only have one starting builder."),
        };
    }

    pub fn with(&mut self, metabuilder: Box<dyn MetaMapBuilder>) {
        self.builders.push(metabuilder);
    }

    pub fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator) {
        match &mut self.starter {
            None => panic!("Cannot run a map builder chain without a starting build system"),
            Some(starter) => {
                // Build the starting map
                starter.build_map(rng, &mut self.build_data);
            }
        }

        // Build additional layers in turn
        for metabuilder in self.builders.iter_mut() {
            metabuilder.build_map(rng, &mut self.build_data);
        }
    }

    pub fn spawn_entities(&mut self, ecs: &mut World) {
        for entity in self.build_data.spawn_list.iter() {
            spawner::spawn_entity(ecs, &(&entity.0, &entity.1));
        }
    }
}

pub trait InitialMapBuilder {
    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data: &mut BuilderMap);
}

pub trait MetaMapBuilder {
    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data: &mut BuilderMap);
}

fn random_initial_builder(rng: &mut rltk::RandomNumberGenerator) -> (Box<dyn InitialMapBuilder>, bool) {
    let builder = rng.roll_dice(1, 17);
    let result: (Box<dyn InitialMapBuilder>, bool);
    match builder {
        1 => result = (BspDungeonBuilder::new(), true),
        2 => result = (BspInteriorBuilder::new(), true),
        3 => result = (CellularAutomataBuilder::new(), false),
        4 => result = (DrunkardsWalkBuilder::open_area(), false),
        5 => result = (DrunkardsWalkBuilder::open_halls(), false),
        6 => result = (DrunkardsWalkBuilder::winding_passages(), false),
        7 => result = (DrunkardsWalkBuilder::fat_passages(), false),
        8 => result = (DrunkardsWalkBuilder::fearful_symmetry(), false),
        9 => result = (MazeBuilder::new(), false),
        10 => result = (DLABuilder::walk_inwards(), false),
        11 => result = (DLABuilder::walk_outwards(), false),
        12 => result = (DLABuilder::central_attractor(), false),
        13 => result = (DLABuilder::insectoid(), false),
        14 => result = (VoronoiBuilder::pythagoras(), false),
        15 => result = (VoronoiBuilder::manhattan(), false),
        16 => result = (PrefabBuilder::constant(prefab_builder::prefab_levels::WFC_POPULATED), false),
        _ => result = (simple_map::SimpleMapBuilder::new(), true),
    }
    result
}

pub fn random_builder(new_depth: i32, rng: &mut rltk::RandomNumberGenerator) -> BuilderChain {
    let mut builder = BuilderChain::new(new_depth);
    builder.start_with(simple_map::SimpleMapBuilder::new());
    builder.with(DLABuilder::heavy_erosion());
    builder.with(AreaStartingPosition::new(XStart::CENTRE, YStart::CENTRE));
    builder.with(CullUnreachable::new());
    builder.with(VoronoiSpawning::new());
    builder.with(DistantExit::new());
    builder
}
