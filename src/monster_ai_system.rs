use super::{
    gamelog::GameLog, Confusion, Map, Monster, Name, ParticleBuilder, Position, RunState, Viewshed, WantsToMelee,
};
use rltk::Point;
use specs::prelude::*;

pub struct MonsterAI {}

impl<'a> System<'a> for MonsterAI {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        WriteExpect<'a, Map>,
        WriteExpect<'a, GameLog>,
        ReadExpect<'a, Point>,
        ReadExpect<'a, Entity>,
        ReadExpect<'a, RunState>,
        Entities<'a>,
        WriteStorage<'a, Viewshed>,
        ReadStorage<'a, Monster>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, WantsToMelee>,
        WriteStorage<'a, Confusion>,
        ReadStorage<'a, Name>,
        WriteExpect<'a, ParticleBuilder>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            mut map,
            mut gamelog,
            player_pos,
            player_entity,
            runstate,
            entities,
            mut viewshed,
            monster,
            mut position,
            mut wants_to_melee,
            mut confused,
            name,
            mut particle_builder,
        ) = data;

        if *runstate != RunState::MonsterTurn {
            return;
        }

        for (entity, mut viewshed, _monster, mut pos) in (&entities, &mut viewshed, &monster, &mut position).join() {
            let mut can_act = true;

            // Check confusion
            let is_confused = confused.get_mut(entity);
            if let Some(i_am_confused) = is_confused {
                i_am_confused.turns -= 1;
                let entity_name = name.get(entity).unwrap();
                let mut fg = rltk::RGB::named(rltk::MAGENTA);
                let mut glyph = rltk::to_cp437('?');
                if i_am_confused.turns < 1 {
                    confused.remove(entity);
                    gamelog.entries.push(format!("{} snaps out of its confusion!", entity_name.name));
                    fg = rltk::RGB::named(rltk::MEDIUMSLATEBLUE);
                    glyph = rltk::to_cp437('!');
                } else {
                    gamelog.entries.push(format!("{} is confused.", entity_name.name));
                }
                particle_builder.request(pos.x, pos.y, fg, rltk::RGB::named(rltk::BLACK), glyph, 200.0);
                can_act = false;
            }

            if can_act {
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
                    let path =
                        rltk::a_star_search(map.xy_idx(pos.x, pos.y), map.xy_idx(player_pos.x, player_pos.y), &*map);
                    if path.success && path.steps.len() > 1 {
                        let mut idx = map.xy_idx(pos.x, pos.y);
                        map.blocked[idx] = false;
                        pos.x = (path.steps[1] as i32) % map.width;
                        pos.y = (path.steps[1] as i32) / map.width;
                        idx = map.xy_idx(pos.x, pos.y);
                        map.blocked[idx] = true;
                        viewshed.dirty = true;
                    }
                }
            }
        }
    }
}
