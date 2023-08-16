use crate::{spatial, BlocksTile, Map, Pools, Position};
use specs::prelude::*;

pub struct MapIndexingSystem {}

impl<'a> System<'a> for MapIndexingSystem {
    type SystemData = (
        ReadExpect<'a, Map>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, BlocksTile>,
        Entities<'a>,
        ReadStorage<'a, Pools>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (map, position, blockers, entities, pools) = data;

        spatial::clear();
        spatial::populate_blocked_from_map(&*map);
        for (entity, position) in (&entities, &position).join() {
            let mut alive = true;
            if let Some(pools) = pools.get(entity) {
                if pools.hit_points.current < 1 {
                    alive = false;
                }
            }
            if alive {
                let idx = map.xy_idx(position.x, position.y);
                spatial::index_entity(entity, idx, blockers.get(entity).is_some());
            }
        }
    }
}
