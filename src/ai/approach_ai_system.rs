use crate::{ EntityMoved, Map, Position, TakingTurn, Telepath, Viewshed, WantsToApproach };
use rltk::prelude::*;
use specs::prelude::*;

pub struct ApproachAI {}

impl<'a> System<'a> for ApproachAI {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        WriteExpect<'a, RandomNumberGenerator>,
        WriteStorage<'a, TakingTurn>,
        WriteStorage<'a, WantsToApproach>,
        WriteStorage<'a, Position>,
        WriteExpect<'a, Map>,
        WriteStorage<'a, Viewshed>,
        WriteStorage<'a, Telepath>,
        WriteStorage<'a, EntityMoved>,
        Entities<'a>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            mut rng,
            mut turns,
            mut wants_to_approach,
            mut positions,
            mut map,
            mut viewsheds,
            mut telepaths,
            mut entity_moved,
            entities,
        ) = data;
        let mut turn_done: Vec<Entity> = Vec::new();
        for (entity, mut pos, approach, mut viewshed, _turn) in (
            &entities,
            &mut positions,
            &wants_to_approach,
            &mut viewsheds,
            &turns,
        ).join() {
            turn_done.push(entity);
            let target_idxs = if let Some(paths) = get_adjacent_unblocked(&map, approach.idx as usize) {
                paths
            } else {
                continue;
            };
            let mut path: Option<NavigationPath> = None;
            let idx = map.xy_idx(pos.x, pos.y);
            for tar_idx in target_idxs {
                let potential_path = rltk::a_star_search(idx, tar_idx, &mut *map);
                if potential_path.success && potential_path.steps.len() > 1 {
                    if path.is_none() || potential_path.steps.len() < path.as_ref().unwrap().steps.len() {
                        path = Some(potential_path);
                    }
                }
            }
            let path = if path.is_some() {
                path.unwrap()
            } else {
                continue;
            };
            if path.success && path.steps.len() > 1 {
                let idx = map.xy_idx(pos.x, pos.y);
                pos.x = (path.steps[1] as i32) % map.width;
                pos.y = (path.steps[1] as i32) / map.width;
                entity_moved.insert(entity, EntityMoved {}).expect("Unable to insert EntityMoved");
                let new_idx = map.xy_idx(pos.x, pos.y);
                crate::spatial::move_entity(entity, idx, new_idx);
                viewshed.dirty = true;
                if let Some(telepath) = telepaths.get_mut(entity) {
                    telepath.dirty = true;
                }
            }
        }
        wants_to_approach.clear();
        for done in turn_done.iter() {
            turns.remove(*done);
        }
    }
}

/// Try to get an unblocked index within one tile of a given idx, or None.
pub fn get_adjacent_unblocked(map: &WriteExpect<Map>, idx: usize) -> Option<Vec<usize>> {
    let mut adjacent = Vec::new();
    let x = (idx as i32) % map.width;
    let y = (idx as i32) / map.width;
    for i in -1..2 {
        for j in -1..2 {
            if i == 0 && j == 0 {
                continue;
            }
            let new_idx = (x + i + (y + j) * map.width) as usize;
            if !crate::spatial::is_blocked(new_idx) {
                adjacent.push(new_idx);
            }
        }
    }
    if adjacent.is_empty() {
        return None;
    }
    return Some(adjacent);
}
