use super::{
    AreaStartingPosition,
    BuilderChain,
    BuilderMap,
    CellularAutomataBuilder,
    CullUnreachable,
    MetaMapBuilder,
    TileType,
    VoronoiSpawning,
    XStart,
    YStart,
    Foliage,
};
use rltk::prelude::*;
use crate::data::names::*;

pub fn forest_builder(
    new_id: i32,
    _rng: &mut rltk::RandomNumberGenerator,
    width: i32,
    height: i32,
    difficulty: i32,
    initial_player_level: i32
) -> BuilderChain {
    let mut chain = BuilderChain::new(
        false,
        new_id,
        width,
        height,
        difficulty,
        NAME_FOREST_BUILDER,
        initial_player_level
    );
    chain.start_with(CellularAutomataBuilder::floor(TileType::Grass));
    // Change ~30% of the floor to some sort of foliage.
    chain.with(AreaStartingPosition::new(XStart::CENTRE, YStart::CENTRE));
    chain.with(CullUnreachable::new());
    chain.with(AreaStartingPosition::new(XStart::LEFT, YStart::CENTRE));
    // Setup an exit and spawn mobs
    chain.with(VoronoiSpawning::new());
    chain.with(RoadExit::new());
    chain.with(Foliage::percent(TileType::Grass, 30));
    return chain;
}

pub struct RoadExit {}

impl MetaMapBuilder for RoadExit {
    fn build_map(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        self.build(rng, build_data);
    }
}

impl RoadExit {
    #[allow(dead_code)]
    pub fn new() -> Box<RoadExit> {
        return Box::new(RoadExit {});
    }

    fn find_exit(&self, build_data: &mut BuilderMap, seed_x: i32, seed_y: i32) -> (i32, i32) {
        let mut available_floors: Vec<(usize, f32)> = Vec::new();
        for (idx, tiletype) in build_data.map.tiles.iter().enumerate() {
            if crate::map::tile_walkable(*tiletype) {
                available_floors.push((
                    idx,
                    DistanceAlg::PythagorasSquared.distance2d(
                        Point::new((idx as i32) % build_data.map.width, (idx as i32) / build_data.map.width),
                        Point::new(seed_x, seed_y)
                    ),
                ));
            }
        }
        if available_floors.is_empty() {
            panic!("No valid floors to start on.");
        }
        available_floors.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        let end_x = (available_floors[0].0 as i32) % build_data.map.width;
        let end_y = (available_floors[0].0 as i32) / build_data.map.width;
        return (end_x, end_y);
    }

    fn paint_road(&self, build_data: &mut BuilderMap, x: i32, y: i32) {
        if x < 1 || x > build_data.map.width - 2 || y < 1 || y > build_data.map.width - 2 {
            return;
        }
        let idx = build_data.map.xy_idx(x, y);
        if build_data.map.tiles[idx] != TileType::DownStair {
            build_data.map.tiles[idx] = TileType::Road;
        }
    }

    fn build(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        let starting_pos = build_data.starting_position.as_ref().unwrap().clone();
        let start_idx = build_data.map.xy_idx(starting_pos.x, starting_pos.y);
        let (end_x, end_y) = self.find_exit(build_data, build_data.map.width - 2, build_data.height / 2);
        let end_idx = build_data.map.xy_idx(end_x, end_y);
        build_data.map.populate_blocked();

        let path = a_star_search(start_idx, end_idx, &mut build_data.map);
        for idx in path.steps.iter() {
            let x = (*idx as i32) % build_data.map.width;
            let y = (*idx as i32) / build_data.map.width;
            self.paint_road(build_data, x, y);
            self.paint_road(build_data, x - 1, y);
            self.paint_road(build_data, x + 1, y);
            self.paint_road(build_data, x, y - 1);
            self.paint_road(build_data, x, y + 1);
        }
        build_data.take_snapshot();

        let exit_dir = rng.roll_dice(1, 2);
        let (seed_x, seed_y, stream_start_x, stream_start_y) = if exit_dir == 1 {
            (build_data.map.width - 1, 1, 0, build_data.height - 1)
        } else {
            (build_data.map.width - 1, build_data.height - 1, 1, build_data.height - 1)
        };
        let (stairs_x, stairs_y) = self.find_exit(build_data, seed_x, seed_y);
        let stairs_idx = build_data.map.xy_idx(stairs_x, stairs_y);
        let (stream_x, stream_y) = self.find_exit(build_data, stream_start_x, stream_start_y);
        let stream_idx = build_data.map.xy_idx(stream_x, stream_y) as usize;
        let stream = a_star_search(stairs_idx, stream_idx, &mut build_data.map);
        for tile in stream.steps.iter() {
            // Maybe only turn grass to water here, and turn the road into a bridge.
            // i.e. if build_data.map.tiles[*tile as usize] == TileType::Grass
            build_data.map.tiles[*tile as usize] = TileType::ShallowWater;
        }
        build_data.map.tiles[stairs_idx] = TileType::DownStair;
        build_data.take_snapshot();
    }
}
