use crate::{Equipped, InBackpack, Map, Position};
use rltk::prelude::*;
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

pub fn find_item_position(ecs: &World, target: Entity) -> Option<i32> {
    let positions = ecs.read_storage::<Position>();
    let map = ecs.fetch::<Map>();
    // Does it have a position?
    if let Some(pos) = positions.get(target) {
        return Some(map.xy_idx(pos.x, pos.y) as i32);
    }
    // If not, is it carried?
    if let Some(carried) = ecs.read_storage::<InBackpack>().get(target) {
        if let Some(pos) = positions.get(carried.owner) {
            return Some(map.xy_idx(pos.x, pos.y) as i32);
        }
    }
    // Is it equipped?
    if let Some(carried) = ecs.read_storage::<Equipped>().get(target) {
        if let Some(pos) = positions.get(carried.owner) {
            return Some(map.xy_idx(pos.x, pos.y) as i32);
        }
    }
    // Out of luck: give up
    console::log("DEBUGINFO: Failed to find item position");
    None
}
