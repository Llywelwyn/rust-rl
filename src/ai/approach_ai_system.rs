use crate::{ EntityMoved, Map, Position, TakingTurn, Telepath, Viewshed, WantsToApproach };
use rltk::prelude::*;
use specs::prelude::*;

pub struct ApproachAI {}

impl<'a> System<'a> for ApproachAI {
    #[allow(clippy::type_complexity)]
    type SystemData = (
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
            let path = a_star_search(
                map.xy_idx(pos.x, pos.y) as i32,
                map.xy_idx(approach.idx % map.width, approach.idx / map.width) as i32,
                &mut *map
            );
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
