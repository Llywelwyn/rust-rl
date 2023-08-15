use crate::{
    raws::Reaction, Chasing, Faction, Map, Mind, Position, TakingTurn, Telepath, Viewshed, WantsToApproach, WantsToFlee,
};
use specs::prelude::*;
use std::collections::HashSet;

pub struct VisibleAI {}

impl<'a> System<'a> for VisibleAI {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        ReadStorage<'a, TakingTurn>,
        ReadStorage<'a, Faction>,
        ReadStorage<'a, Position>,
        ReadExpect<'a, Map>,
        WriteStorage<'a, WantsToApproach>,
        WriteStorage<'a, WantsToFlee>,
        Entities<'a>,
        ReadExpect<'a, Entity>,
        ReadStorage<'a, Viewshed>,
        ReadStorage<'a, Telepath>,
        ReadStorage<'a, Mind>,
        WriteStorage<'a, Chasing>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            turns,
            factions,
            positions,
            map,
            mut wants_to_approach,
            mut wants_to_flee,
            entities,
            player,
            viewsheds,
            telepaths,
            minds,
            mut chasing,
        ) = data;

        for (entity, _turn, faction, pos, viewshed) in (&entities, &turns, &factions, &positions, &viewsheds).join() {
            if entity == *player {
                continue;
            }
            let this_idx = map.xy_idx(pos.x, pos.y);
            let mut reactions: Vec<(usize, Reaction, Entity)> = Vec::new();
            let mut flee: Vec<usize> = Vec::new();
            let mut idxs: HashSet<usize> = HashSet::new();
            for visible_tile in viewshed.visible_tiles.iter() {
                let idx = map.xy_idx(visible_tile.x, visible_tile.y);
                if this_idx != idx {
                    evaluate(idx, &map, &factions, &faction.name, &mut reactions, None);
                    idxs.insert(idx);
                }
            }
            if let Some(is_telepath) = telepaths.get(entity) {
                for telepath_tile in is_telepath.telepath_tiles.iter() {
                    let idx = map.xy_idx(telepath_tile.x, telepath_tile.y);
                    // If we didn't already evaluate this idx (if it's not contained in the HashSet),
                    // and it's not the idx we're standing on, then evaluate here w/ minds taken into
                    // account.
                    if this_idx != idx && idxs.contains(&idx) {
                        evaluate(idx, &map, &factions, &faction.name, &mut reactions, Some(&minds));
                    }
                }
            }
            let mut done = false;
            for reaction in reactions.iter() {
                match reaction.1 {
                    Reaction::Attack => {
                        wants_to_approach
                            .insert(entity, WantsToApproach { idx: reaction.0 as i32 })
                            .expect("Error inserting WantsToApproach");
                        chasing.insert(entity, Chasing { target: reaction.2 }).expect("Unable to insert Chasing");
                        done = true;
                    }
                    Reaction::Flee => {
                        flee.push(reaction.0);
                    }
                    _ => {}
                }
            }
            if !done && !flee.is_empty() {
                wants_to_flee.insert(entity, WantsToFlee { indices: flee }).expect("Unable to insert");
            }
        }
    }
}

fn evaluate(
    idx: usize,
    map: &Map,
    factions: &ReadStorage<Faction>,
    this_faction: &str,
    reactions: &mut Vec<(usize, Reaction, Entity)>,
    minds: Option<&ReadStorage<Mind>>,
) {
    for other_entity in map.tile_content[idx].iter() {
        // If minds are passed, we assume we're using telepathy here,
        // so if the other entity is mindless, we skip it.
        if minds.is_some() {
            if minds.unwrap().get(*other_entity).is_none() {
                continue;
            }
        }
        if let Some(faction) = factions.get(*other_entity) {
            reactions.push((
                idx,
                crate::raws::faction_reaction(this_faction, &faction.name, &crate::raws::RAWS.lock().unwrap()),
                *other_entity,
            ));
        }
    }
}
