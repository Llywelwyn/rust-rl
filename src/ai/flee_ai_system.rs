use crate::{EntityMoved, Map, Position, TakingTurn, Telepath, Viewshed, WantsToFlee};
use rltk::prelude::*;
use specs::prelude::*;

pub struct FleeAI {}

impl<'a> System<'a> for FleeAI {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        WriteStorage<'a, TakingTurn>,
        WriteStorage<'a, WantsToFlee>,
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
            mut wants_to_flee,
            mut positions,
            mut map,
            mut viewsheds,
            mut telepaths,
            mut entity_moved,
            entities,
        ) = data;
        let mut turn_done: Vec<Entity> = Vec::new();
        for (entity, _turn, mut pos, fleeing, mut viewshed) in
            (&entities, &turns, &mut positions, &wants_to_flee, &mut viewsheds).join()
        {
            turn_done.push(entity);
            let my_idx = map.xy_idx(pos.x, pos.y);
            map.populate_blocked();
            let flee_map = DijkstraMap::new(map.width as usize, map.height as usize, &fleeing.indices, &*map, 100.0);
            let flee_target = DijkstraMap::find_highest_exit(&flee_map, my_idx, &*map);
            if let Some(flee_target) = flee_target {
                if !map.blocked[flee_target as usize] {
                    map.blocked[my_idx] = false;
                    map.blocked[flee_target as usize] = true;
                    viewshed.dirty = true;
                    if let Some(is_telepath) = telepaths.get_mut(entity) {
                        is_telepath.dirty = true;
                    }
                    pos.x = flee_target as i32 % map.width;
                    pos.y = flee_target as i32 / map.width;
                    entity_moved.insert(entity, EntityMoved {}).expect("Unable to insert EntityMoved");
                }
            }
        }
        wants_to_flee.clear();
        for done in turn_done.iter() {
            turns.remove(*done);
        }
    }
}
