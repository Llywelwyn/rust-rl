use super::{EntityMoved, Map, Monster, Position, TakingTurn, Viewshed, WantsToMelee};
use rltk::Point;
use specs::prelude::*;

pub struct MonsterAI {}

impl<'a> System<'a> for MonsterAI {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        WriteExpect<'a, Map>,
        ReadExpect<'a, Point>,
        ReadExpect<'a, Entity>,
        Entities<'a>,
        WriteStorage<'a, Viewshed>,
        ReadStorage<'a, Monster>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, WantsToMelee>,
        WriteStorage<'a, EntityMoved>,
        ReadStorage<'a, TakingTurn>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            mut map,
            player_pos,
            player_entity,
            entities,
            mut viewshed,
            monster,
            mut position,
            mut wants_to_melee,
            mut entity_moved,
            turns,
        ) = data;

        for (entity, mut viewshed, _monster, mut pos, _turn) in
            (&entities, &mut viewshed, &monster, &mut position, &turns).join()
        {
            let distance = rltk::DistanceAlg::Pythagoras.distance2d(Point::new(pos.x, pos.y), *player_pos);
            if distance < 1.5 {
                wants_to_melee
                    .insert(entity, WantsToMelee { target: *player_entity })
                    .expect("Unable to insert attack.");
            } else if viewshed.visible_tiles.contains(&*player_pos) {
                // If the player is visible, but the path is obstructed, this will currently search
                // the entire map (i.e. Will do a huge ASTAR to find an alternate route), and the
                // mob will follow that path until it leaves vision, then lose sight of the player
                // and stop.
                let path = rltk::a_star_search(map.xy_idx(pos.x, pos.y), map.xy_idx(player_pos.x, player_pos.y), &*map);
                if path.success && path.steps.len() > 1 {
                    let mut idx = map.xy_idx(pos.x, pos.y);
                    map.blocked[idx] = false;
                    pos.x = (path.steps[1] as i32) % map.width;
                    pos.y = (path.steps[1] as i32) / map.width;
                    idx = map.xy_idx(pos.x, pos.y);
                    map.blocked[idx] = true;
                    viewshed.dirty = true;
                    entity_moved.insert(entity, EntityMoved {}).expect("Unable to insert marker");
                }
            }
        }
    }
}
