use crate::{Map, Position};
use specs::prelude::*;

pub fn entity_position(ecs: &World, target: Entity) -> Option<usize> {
    if let Some(position) = ecs.read_storage::<Position>().get(target) {
        let map = ecs.fetch::<Map>();
        return Some(map.xy_idx(position.x, position.y));
    }
    return None;
}

pub fn aoe_tiles(map: &Map, target: rltk::Point, radius: i32) -> Vec<usize> {
    let mut blast_tiles = rltk::field_of_view(target, radius, &*map);
    blast_tiles.retain(|p| p.x > 0 && p.x < map.width - 1 && p.y > 0 && p.y < map.height - 1);
    let mut result = Vec::new();
    for t in blast_tiles.iter() {
        result.push(map.xy_idx(t.x, t.y));
    }
    result
}
