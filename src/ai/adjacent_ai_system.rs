use crate::{raws::Reaction, Faction, HasAncestry, Map, Position, TakingTurn, WantsToMelee};
use specs::prelude::*;

pub struct AdjacentAI {}

impl<'a> System<'a> for AdjacentAI {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        WriteStorage<'a, TakingTurn>,
        ReadStorage<'a, Faction>,
        ReadStorage<'a, HasAncestry>,
        ReadStorage<'a, Position>,
        ReadExpect<'a, Map>,
        WriteStorage<'a, WantsToMelee>,
        Entities<'a>,
        ReadExpect<'a, Entity>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut turns, factions, ancestries, positions, map, mut want_melee, entities, player) = data;

        let mut turn_done: Vec<Entity> = Vec::new();
        for (entity, _turn, pos) in (&entities, &turns, &positions).join() {
            if entity != *player {
                let mut reactions: Vec<(Entity, Reaction)> = Vec::new();
                let idx = map.xy_idx(pos.x, pos.y);
                let w = map.width;
                let h = map.height;
                // Add possible reactions to adjacents for each direction
                if pos.x > 0 {
                    evaluate(entity, idx - 1, &ancestries, &factions, &mut reactions);
                }
                if pos.x < w - 1 {
                    evaluate(entity, idx + 1, &ancestries, &factions, &mut reactions);
                }
                if pos.y > 0 {
                    evaluate(entity, idx - w as usize, &ancestries, &factions, &mut reactions);
                }
                if pos.y < h - 1 {
                    evaluate(entity, idx + w as usize, &ancestries, &factions, &mut reactions);
                }
                if pos.y > 0 && pos.x > 0 {
                    evaluate(entity, (idx - w as usize) - 1, &ancestries, &factions, &mut reactions);
                }
                if pos.y > 0 && pos.x < w - 1 {
                    evaluate(entity, (idx - w as usize) + 1, &ancestries, &factions, &mut reactions);
                }
                if pos.y < h - 1 && pos.x > 0 {
                    evaluate(entity, (idx + w as usize) - 1, &ancestries, &factions, &mut reactions);
                }
                if pos.y < h - 1 && pos.x < w - 1 {
                    evaluate(entity, (idx + w as usize) + 1, &ancestries, &factions, &mut reactions);
                }

                let mut done = false;
                for reaction in reactions.iter() {
                    if let Reaction::Attack = reaction.1 {
                        want_melee.insert(entity, WantsToMelee { target: reaction.0 }).expect("Error inserting melee");
                        done = true;
                    }
                }

                if done {
                    turn_done.push(entity);
                }
            }
        }

        // Remove turn marker for those that are done
        for done in turn_done.iter() {
            turns.remove(*done);
        }
    }
}

/// Evaluates all possible reactions between this faction and all entities on a given tile idx.
fn evaluate(
    entity: Entity,
    idx: usize,
    ancestries: &ReadStorage<HasAncestry>,
    factions: &ReadStorage<Faction>,
    reactions: &mut Vec<(Entity, Reaction)>,
) {
    crate::spatial::for_each_tile_content(idx, |other_entity| {
        let result = crate::raws::get_reactions(
            entity,
            other_entity,
            &factions,
            &ancestries,
            &crate::raws::RAWS.lock().unwrap(),
        );
        reactions.push((other_entity, result));
    });
}
