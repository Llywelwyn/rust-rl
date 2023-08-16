use crate::{Map, Position};
use specs::prelude::*;

pub fn entity_position(ecs: &World, target: Entity) -> Option<usize> {
    if let Some(position) = ecs.read_storage::<Position>().get(target) {
        let map = ecs.fetch::<Map>();
        return Some(map.xy_idx(position.x, position.y));
    }
    return None;
}
