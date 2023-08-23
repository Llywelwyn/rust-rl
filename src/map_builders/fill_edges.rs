use super::{ BuilderMap, MetaMapBuilder, TileType };
use rltk::RandomNumberGenerator;

pub struct FillEdges {
    fill_with: TileType,
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
        return Box::new(FillEdges { fill_with: TileType::Wall });
    }

    fn fill_edges(&mut self, _rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        for x in 0..build_data.map.width {
            let mut idx = build_data.map.xy_idx(x, 0);
            build_data.map.tiles[idx] = self.fill_with;
            idx = build_data.map.xy_idx(x, build_data.map.height - 1);
            build_data.map.tiles[idx] = self.fill_with;
        }
        for y in 0..build_data.map.height {
            let mut idx = build_data.map.xy_idx(0, y);
            build_data.map.tiles[idx] = self.fill_with;
            idx = build_data.map.xy_idx(build_data.map.width - 1, y);
            build_data.map.tiles[idx] = self.fill_with;
        }
    }
}
