use crate::{raws::Reaction, Faction, Map, Position, TakingTurn, WantsToMelee};
use specs::prelude::*;

pub struct AdjacentAI {}

impl<'a> System<'a> for AdjacentAI {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        WriteStorage<'a, TakingTurn>,
        ReadStorage<'a, Faction>,
        ReadStorage<'a, Position>,
        ReadExpect<'a, Map>,
        WriteStorage<'a, WantsToMelee>,
        Entities<'a>,
        ReadExpect<'a, Entity>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut turns, factions, positions, map, mut wants_to_melee, entities, player) = data;

        let mut turn_done: Vec<Entity> = Vec::new();
        for (entity, _turn, faction, pos) in (&entities, &turns, &factions, &positions).join() {
            if entity == *player {
                continue;
            }
            let mut reactions: Vec<(Entity, Reaction)> = Vec::new();
            let idx = map.xy_idx(pos.x, pos.y);
            let (w, h) = (map.width, map.height);
            // Evaluate adjacent squares, add possible reactions
            if pos.x > 0 {
                evaluate(idx - 1, &map, &factions, &faction.name, &mut reactions);
            }
            if pos.x < w - 1 {
                evaluate(idx + 1, &map, &factions, &faction.name, &mut reactions);
            }
            if pos.y > 0 {
                evaluate(idx - w as usize, &map, &factions, &faction.name, &mut reactions);
            }
            if pos.y < h - 1 {
                evaluate(idx + w as usize, &map, &factions, &faction.name, &mut reactions);
            }
            if pos.y > 0 && pos.x > 0 {
                evaluate((idx - w as usize) - 1, &map, &factions, &faction.name, &mut reactions);
            }
            if pos.y > 0 && pos.x < w - 1 {
                evaluate((idx - w as usize) + 1, &map, &factions, &faction.name, &mut reactions);
            }
            if pos.y < h - 1 && pos.x > 0 {
                evaluate((idx + w as usize) - 1, &map, &factions, &faction.name, &mut reactions);
            }
            if pos.y < h - 1 && pos.x < w - 1 {
                evaluate((idx + w as usize) + 1, &map, &factions, &faction.name, &mut reactions);
            }
            let mut done = false;
            for reaction in reactions.iter() {
                if let Reaction::Attack = reaction.1 {
                    wants_to_melee
                        .insert(entity, WantsToMelee { target: reaction.0 })
                        .expect("Error inserting WantsToMelee");
                    done = true;
                }
            }
            if done {
                turn_done.push(entity);
            }
        }
        // Remove turn from entities that are done
        for done in turn_done.iter() {
            turns.remove(*done);
        }
    }
}

/// Evaluates all possible reactions between this faction and all entities on a given tile idx.
fn evaluate(
    idx: usize,
    map: &Map,
    factions: &ReadStorage<Faction>,
    this_faction: &str,
    reactions: &mut Vec<(Entity, Reaction)>,
) {
    for other_entity in map.tile_content[idx].iter() {
        if let Some(faction) = factions.get(*other_entity) {
            reactions.push((
                *other_entity,
                crate::raws::faction_reaction(this_faction, &faction.name, &crate::raws::RAWS.lock().unwrap()),
            ));
        }
    }
}
