use super::{spatial, BlocksTile, Map, Position};
use specs::prelude::*;

pub struct MapIndexingSystem {}

impl<'a> System<'a> for MapIndexingSystem {
    type SystemData = (WriteExpect<'a, Map>, ReadStorage<'a, Position>, ReadStorage<'a, BlocksTile>, Entities<'a>);

    fn run(&mut self, data: Self::SystemData) {
        let (mut map, position, blockers, entities) = data;

        spatial::clear();
        spatial::populate_blocked_from_map(&*map);
        for (entity, position) in (&entities, &position).join() {
            let idx = map.xy_idx(position.x, position.y);
            spatial::index_entity(entity, idx, blockers.get(entity).is_some());
        }
    }
}
