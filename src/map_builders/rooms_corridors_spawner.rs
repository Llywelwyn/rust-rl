use super::{spawner, BuilderMap, MetaMapBuilder};
use rltk::RandomNumberGenerator;

pub struct CorridorSpawner {}

impl MetaMapBuilder for CorridorSpawner {
    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data: &mut BuilderMap) {
        self.build(rng, build_data);
    }
}

impl CorridorSpawner {
    #[allow(dead_code)]
    pub fn new() -> Box<CorridorSpawner> {
        return Box::new(CorridorSpawner {});
    }

    fn build(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        if let Some(corridors) = &build_data.corridors {
            for corridor in corridors.iter() {
                let depth = build_data.map.depth;
                spawner::spawn_region(&build_data.map, rng, &corridor, depth, &mut build_data.spawn_list);
            }
        } else {
            panic!("CorridorSpawner only works after corridors have been created");
        }
    }
}
