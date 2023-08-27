use super::{ BuilderMap, MetaMapBuilder, TileType };
use crate::tile_walkable;
use rltk::RandomNumberGenerator;

pub struct FillEdges {
    fill_with: TileType,
    only_walkable: bool,
}

impl MetaMapBuilder for FillEdges {
    #[allow(dead_code)]
    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data: &mut BuilderMap) {
        self.fill_edges(rng, build_data);
    }
}

impl FillEdges {
    #[allow(dead_code)]
    pub fn wall() -> Box<FillEdges> {
        return Box::new(FillEdges { fill_with: TileType::Wall, only_walkable: false });
    }
    pub fn overmap_transition(id: i32) -> Box<FillEdges> {
        return Box::new(FillEdges { fill_with: TileType::ToOvermap(id), only_walkable: true });
    }

    fn fill_edges(&mut self, _rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        // Get map edges as possible points to fill
        let mut possible_idxs: Vec<usize> = Vec::new();
        for x in 0..build_data.map.width {
            let mut idx = build_data.map.xy_idx(x, 0);
            possible_idxs.push(idx);
            idx = build_data.map.xy_idx(x, build_data.map.height - 1);
            possible_idxs.push(idx);
        }
        for y in 0..build_data.map.height {
            let mut idx = build_data.map.xy_idx(0, y);
            possible_idxs.push(idx);
            idx = build_data.map.xy_idx(build_data.map.width - 1, y);
            possible_idxs.push(idx);
        }
        // For every possible point, first check if we only want to fill walkable tiles (and if its walkable if so)
        for idx in possible_idxs {
            if !self.only_walkable || tile_walkable(build_data.map.tiles[idx]) {
                build_data.map.tiles[idx] = self.fill_with;
            }
        }
    }
}
