use super::{ spawner, Map, Position, Rect, TileType };
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
use simple_map::SimpleMapBuilder;
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
use area_starting_points::{ AreaStartingPosition, XStart, YStart };
mod cull_unreachable;
use cull_unreachable::CullUnreachable;
mod distant_exit;
use distant_exit::DistantExit;
mod voronoi_spawning;
use common::*;
use specs::prelude::*;
use voronoi_spawning::VoronoiSpawning;
use super::config::CONFIG;
//use wfc::WaveFunctionCollapseBuilder;
mod room_exploder;
use room_exploder::RoomExploder;
mod room_corner_rounding;
use room_corner_rounding::RoomCornerRounder;
mod rooms_corridors_dogleg;
use rooms_corridors_dogleg::DoglegCorridors;
mod rooms_corridors_bsp;
use rooms_corridors_bsp::BspCorridors;
mod room_sorter;
use room_sorter::{ RoomSort, RoomSorter };
mod room_draw;
use room_draw::RoomDrawer;
mod rooms_corridors_nearest;
use rooms_corridors_nearest::NearestCorridors;
mod rooms_corridors_bresenham;
use rooms_corridors_bresenham::BresenhamCorridors;
mod rooms_corridors_spawner;
use rooms_corridors_spawner::CorridorSpawner;
mod door_placement;
use door_placement::DoorPlacement;
mod fill_edges;
use fill_edges::FillEdges;
mod town;
use town::town_builder;
mod forest;
use forest::forest_builder;

// Shared data to be passed around build chain
pub struct BuilderMap {
    pub spawn_list: Vec<(usize, String)>,
    pub map: Map,
    pub starting_position: Option<Position>,
    pub rooms: Option<Vec<Rect>>,
    pub corridors: Option<Vec<Vec<usize>>>,
    pub history: Vec<Map>,
    pub width: i32,
    pub height: i32,
    pub initial_player_level: i32,
}

impl BuilderMap {
    fn take_snapshot(&mut self) {
        if CONFIG.logging.show_mapgen {
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
    pub fn new<S: ToString>(
        overmap: bool,
        new_id: i32,
        width: i32,
        height: i32,
        difficulty: i32,
        name: S,
        initial_player_level: i32
    ) -> BuilderChain {
        BuilderChain {
            starter: None,
            builders: Vec::new(),
            build_data: BuilderMap {
                spawn_list: Vec::new(),
                map: Map::new(overmap, new_id, width, height, difficulty, name),
                starting_position: None,
                rooms: None,
                corridors: None,
                history: Vec::new(),
                width: width,
                height: height,
                initial_player_level: initial_player_level,
            },
        }
    }

    pub fn start_with(&mut self, starter: Box<dyn InitialMapBuilder>) {
        match self.starter {
            None => {
                self.starter = Some(starter);
            }
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
        let mut spawned_entities = Vec::new();
        for entity in self.build_data.spawn_list.iter() {
            spawned_entities.push(&entity.1);
            spawner::spawn_entity(ecs, &(&entity.0, &entity.1));
        }
        if CONFIG.logging.log_spawning {
            rltk::console::log(format!("DEBUGINFO: SPAWNED ENTITIES = {:?}", spawned_entities));
        }
    }
}

pub trait InitialMapBuilder {
    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data: &mut BuilderMap);
}

pub trait MetaMapBuilder {
    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data: &mut BuilderMap);
}

fn random_start_position(rng: &mut rltk::RandomNumberGenerator) -> (XStart, YStart) {
    let x;
    let xroll = rng.roll_dice(1, 3);
    match xroll {
        1 => {
            x = XStart::LEFT;
        }
        2 => {
            x = XStart::CENTRE;
        }
        _ => {
            x = XStart::RIGHT;
        }
    }

    let y;
    let yroll = rng.roll_dice(1, 3);
    match yroll {
        1 => {
            y = YStart::BOTTOM;
        }
        2 => {
            y = YStart::CENTRE;
        }
        _ => {
            y = YStart::TOP;
        }
    }

    (x, y)
}

fn random_room_builder(rng: &mut rltk::RandomNumberGenerator, builder: &mut BuilderChain) {
    let build_roll = rng.roll_dice(1, 3);
    // Start with a room builder.
    match build_roll {
        1 => builder.start_with(SimpleMapBuilder::new(None)),
        2 => builder.start_with(BspDungeonBuilder::new()),
        _ => builder.start_with(BspInteriorBuilder::new()),
    }

    // BspInterior makes its own doorways. If we're not using that one,
    // select a sorting method, a type of corridor, and modifiers.
    if build_roll != 3 {
        // Sort by one of the 5 available algorithms
        let sort_roll = rng.roll_dice(1, 5);
        match sort_roll {
            1 => builder.with(RoomSorter::new(RoomSort::LEFTMOST)),
            2 => builder.with(RoomSorter::new(RoomSort::RIGHTMOST)),
            3 => builder.with(RoomSorter::new(RoomSort::TOPMOST)),
            4 => builder.with(RoomSorter::new(RoomSort::BOTTOMMOST)),
            _ => builder.with(RoomSorter::new(RoomSort::CENTRAL)),
        }

        builder.with(RoomDrawer::new());

        let corridor_roll = rng.roll_dice(1, 2);
        match corridor_roll {
            1 => builder.with(DoglegCorridors::new()),
            _ => builder.with(BspCorridors::new()),
        }

        let corridor_roll = rng.roll_dice(1, 4);
        match corridor_roll {
            1 => builder.with(DoglegCorridors::new()),
            2 => builder.with(NearestCorridors::new()),
            3 => builder.with(BresenhamCorridors::new()),
            _ => builder.with(BspCorridors::new()),
        }

        let cspawn_roll = rng.roll_dice(1, 2);
        if cspawn_roll == 1 {
            builder.with(CorridorSpawner::new());
        }

        let modifier_roll = rng.roll_dice(1, 6);
        match modifier_roll {
            1 => builder.with(RoomExploder::new()),
            2 => builder.with(RoomCornerRounder::new()),
            _ => {}
        }
    }

    // Pick a starting position, in a room or elsewhere.
    let start_roll = rng.roll_dice(1, 2);
    match start_roll {
        1 => builder.with(RoomBasedStartingPosition::new()),
        _ => {
            let (start_x, start_y) = random_start_position(rng);
            builder.with(AreaStartingPosition::new(start_x, start_y));
        }
    }

    // Decide where to put the exit - in a room or far away, anywhere.
    let exit_roll = rng.roll_dice(1, 2);
    match exit_roll {
        1 => builder.with(RoomBasedStairs::new()),
        _ => builder.with(DistantExit::new()),
    }

    // Decide whether to spawn entities only in rooms, or with voronoi noise.
    let spawn_roll = rng.roll_dice(1, 2);
    match spawn_roll {
        1 => builder.with(RoomBasedSpawner::new()),
        _ => builder.with(VoronoiSpawning::new()),
    }
}

fn random_shape_builder(rng: &mut rltk::RandomNumberGenerator, builder: &mut BuilderChain) -> bool {
    // Pick an initial builder
    let builder_roll = rng.roll_dice(1, 16);
    let mut want_doors = true;
    match builder_roll {
        1 => builder.start_with(CellularAutomataBuilder::new()),
        2 => builder.start_with(DrunkardsWalkBuilder::open_area()),
        3 => builder.start_with(DrunkardsWalkBuilder::open_halls()),
        4 => builder.start_with(DrunkardsWalkBuilder::winding_passages()),
        5 => builder.start_with(DrunkardsWalkBuilder::fat_passages()),
        6 => builder.start_with(DrunkardsWalkBuilder::fearful_symmetry()),
        7 => {
            builder.start_with(MazeBuilder::new());
            want_doors = false;
        }
        8 => builder.start_with(DLABuilder::walk_inwards()),
        9 => builder.start_with(DLABuilder::walk_outwards()),
        10 => builder.start_with(DLABuilder::central_attractor()),
        11 => builder.start_with(DLABuilder::insectoid()),
        12 => builder.start_with(VoronoiBuilder::pythagoras()),
        13 => builder.start_with(VoronoiBuilder::manhattan()),
        _ => builder.start_with(PrefabBuilder::constant(prefab_builder::prefab_levels::WFC_POPULATED)),
    }

    // 'Select' the centre by placing a starting position, and cull everywhere unreachable.
    builder.with(AreaStartingPosition::new(XStart::CENTRE, YStart::CENTRE));
    builder.with(CullUnreachable::new());

    // Now set the start to a random spot in our remaining area.
    let (start_x, start_y) = random_start_position(rng);
    builder.with(AreaStartingPosition::new(start_x, start_y));

    // Place the exit and spawn mobs
    builder.with(VoronoiSpawning::new());
    builder.with(DistantExit::new());

    return want_doors;
}

fn overmap_builder() -> BuilderChain {
    let mut builder = BuilderChain::new(true, 1, 69, 41, 0, "the world", 1);
    builder.start_with(PrefabBuilder::overmap());
    return builder;
}

pub fn random_builder(
    new_id: i32,
    rng: &mut rltk::RandomNumberGenerator,
    width: i32,
    height: i32,
    difficulty: i32,
    initial_player_level: i32
) -> BuilderChain {
    rltk::console::log(format!("DEBUGINFO: Building random (ID:{}, DIFF:{})", new_id, difficulty));
    let mut builder = BuilderChain::new(false, new_id, width, height, difficulty, "the dungeon", initial_player_level);
    let type_roll = rng.roll_dice(1, 2);
    let mut want_doors = true;
    match type_roll {
        1 => random_room_builder(rng, &mut builder),
        _ => {
            want_doors = random_shape_builder(rng, &mut builder);
        }
    }

    /*
    WFC needs polishing up before it makes good maps. Right now it leaves too much unusable area,
    by making disconnected sections and having no methods to connect them.

    if rng.roll_dice(1, 1) == 1 {
        builder.with(WaveFunctionCollapseBuilder::new());

        // Now set the start to a random starting area
        let (start_x, start_y) = random_start_position(rng);
        builder.with(AreaStartingPosition::new(start_x, start_y));

        // Setup an exit and spawn mobs
        builder.with(VoronoiSpawning::new());
        builder.with(DistantExit::new());
    }
    */

    if want_doors {
        builder.with(DoorPlacement::new());
    }

    if rng.roll_dice(1, 20) == 1 {
        builder.with(PrefabBuilder::sectional(prefab_builder::prefab_sections::UNDERGROUND_FORT));
    }

    builder.with(PrefabBuilder::vaults());
    // Regardless of anything else, fill the edges back in with walls. We can't walk
    // there anyway, and we don't want an open line of sight into the unmapped void.
    builder.with(FillEdges::wall());

    builder
}

pub fn level_builder(
    new_id: i32,
    rng: &mut rltk::RandomNumberGenerator,
    width: i32,
    height: i32,
    initial_player_level: i32
) -> BuilderChain {
    // TODO: With difficulty and ID/depth decoupled, this can be used for branches later.
    let difficulty = new_id;
    match new_id {
        1 => overmap_builder(),
        2 => town_builder(new_id, rng, width, height, 0, initial_player_level),
        3 => forest_builder(new_id, rng, width, height, 1, initial_player_level),
        _ => random_builder(new_id, rng, width, height, difficulty, initial_player_level),
    }
}
