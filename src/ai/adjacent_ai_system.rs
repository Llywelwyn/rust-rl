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
        for (entity, _turn, my_faction, pos) in (&entities, &turns, &factions, &positions).join() {
            if entity != *player {
                let mut reactions: Vec<(Entity, Reaction)> = Vec::new();
                let idx = map.xy_idx(pos.x, pos.y);
                let w = map.width;
                let h = map.height;
                // Add possible reactions to adjacents for each direction
                if pos.x > 0 {
                    evaluate(entity, idx - 1, &ancestries, &factions, &my_faction.name, &mut reactions);
                }
                if pos.x < w - 1 {
                    evaluate(entity, idx + 1, &ancestries, &factions, &my_faction.name, &mut reactions);
                }
                if pos.y > 0 {
                    evaluate(entity, idx - w as usize, &ancestries, &factions, &my_faction.name, &mut reactions);
                }
                if pos.y < h - 1 {
                    evaluate(entity, idx + w as usize, &ancestries, &factions, &my_faction.name, &mut reactions);
                }
                if pos.y > 0 && pos.x > 0 {
                    evaluate(entity, (idx - w as usize) - 1, &ancestries, &factions, &my_faction.name, &mut reactions);
                }
                if pos.y > 0 && pos.x < w - 1 {
                    evaluate(entity, (idx - w as usize) + 1, &ancestries, &factions, &my_faction.name, &mut reactions);
                }
                if pos.y < h - 1 && pos.x > 0 {
                    evaluate(entity, (idx + w as usize) - 1, &ancestries, &factions, &my_faction.name, &mut reactions);
                }
                if pos.y < h - 1 && pos.x < w - 1 {
                    evaluate(entity, (idx + w as usize) + 1, &ancestries, &factions, &my_faction.name, &mut reactions);
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
    this_faction: &str,
    reactions: &mut Vec<(Entity, Reaction)>,
) {
    crate::spatial::for_each_tile_content(idx, |other_entity| {
        let mut shared_ancestry = false;
        if let Some(this_ancestry) = ancestries.get(entity) {
            if let Some(other_ancestry) = ancestries.get(other_entity) {
                if this_ancestry.name == other_ancestry.name {
                    reactions.push((other_entity, Reaction::Ignore));
                    shared_ancestry = true;
                }
            }
        }
        if !shared_ancestry {
            if let Some(faction) = factions.get(other_entity) {
                reactions.push((
                    other_entity,
                    crate::raws::faction_reaction(this_faction, &faction.name, &crate::raws::RAWS.lock().unwrap()),
                ));
            }
        }
    });
}
