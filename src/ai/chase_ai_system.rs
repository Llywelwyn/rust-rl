use crate::{Chasing, EntityMoved, Map, Position, TakingTurn, Telepath, Viewshed};
use rltk::prelude::*;
use specs::prelude::*;
use std::collections::HashMap;

// If the target is beyond this distance, they're no longer being detected,
// so stop following them. This is essentially a combined value of the sound
// the target might be making, noise, light, etc., anything they could do to
// be detected. As those constituent systems are developed, this value should
// be changed to being a result of some calculations between them.
const MAX_CHASE_DISTANCE: usize = 15;
pub struct ChaseAI {}

impl<'a> System<'a> for ChaseAI {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        WriteStorage<'a, TakingTurn>,
        WriteStorage<'a, Chasing>,
        WriteStorage<'a, Position>,
        WriteExpect<'a, Map>,
        WriteStorage<'a, Viewshed>,
        WriteStorage<'a, Telepath>,
        WriteStorage<'a, EntityMoved>,
        Entities<'a>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut turns, mut chasing, mut positions, mut map, mut viewsheds, mut telepaths, mut entity_moved, entities) =
            data;
        let mut targets: HashMap<Entity, (i32, i32)> = HashMap::new();
        let mut end_chase: Vec<Entity> = Vec::new();
        // For every chasing entity with a turn, look for a valid target position,
        // and if found, store that position in a temporary HashMap. This gets around
        // needing to read Position twice - that would cause borrowchecker issues.
        // If there's no valid target found, remove the chasing component.
        for (entity, _turn, chasing) in (&entities, &turns, &chasing).join() {
            if let Some(target_pos) = positions.get(chasing.target) {
                targets.insert(entity, (target_pos.x, target_pos.y));
            } else {
                end_chase.push(entity);
            }
        }
        for done in end_chase.iter() {
            chasing.remove(*done);
        }
        end_chase.clear();
        // Iterate over everyone who is *still* chasing, and path to the target
        // stored in the HashMap. If successful, follow the path. If not, remove
        // the chasing component.
        let mut turn_done: Vec<Entity> = Vec::new();
        for (entity, _turn, mut pos, _chase, mut viewshed) in
            (&entities, &turns, &mut positions, &chasing, &mut viewsheds).join()
        {
            turn_done.push(entity);
            let target_pos = targets[&entity];
            let path = a_star_search(
                map.xy_idx(pos.x, pos.y) as i32,
                map.xy_idx(target_pos.0, target_pos.1) as i32,
                &mut *map,
            );
            if path.success && path.steps.len() > 1 && path.steps.len() < MAX_CHASE_DISTANCE {
                let mut idx = map.xy_idx(pos.x, pos.y);
                map.blocked[idx] = false;
                pos.x = path.steps[1] as i32 % map.width;
                pos.y = path.steps[1] as i32 / map.width;
                entity_moved.insert(entity, EntityMoved {}).expect("Failed to insert EntityMoved");
                idx = map.xy_idx(pos.x, pos.y);
                map.blocked[idx] = true;
                viewshed.dirty = true;
                if let Some(is_telepath) = telepaths.get_mut(entity) {
                    is_telepath.dirty = true;
                }
                turn_done.push(entity);
            } else {
                end_chase.push(entity);
            }
        }
        for done in end_chase.iter() {
            chasing.remove(*done);
        }
        for done in turn_done.iter() {
            turns.remove(*done);
        }
    }
}
