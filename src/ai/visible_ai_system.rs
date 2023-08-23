use crate::{
    raws::Reaction,
    Chasing,
    Faction,
    HasAncestry,
    Map,
    Mind,
    Position,
    TakingTurn,
    Telepath,
    Viewshed,
    WantsToApproach,
    WantsToFlee,
};
use rltk::prelude::*;
use specs::prelude::*;
use std::collections::HashSet;

pub struct VisibleAI {}

impl<'a> System<'a> for VisibleAI {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        ReadStorage<'a, TakingTurn>,
        ReadStorage<'a, Faction>,
        ReadStorage<'a, HasAncestry>,
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
            ancestries,
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

        for (entity, _turn, pos, viewshed) in (&entities, &turns, &positions, &viewsheds).join() {
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
                    evaluate(entity, idx, &ancestries, &factions, &mut reactions, None);
                    idxs.insert(idx);
                }
            }
            if let Some(is_telepath) = telepaths.get(entity) {
                for telepath_tile in is_telepath.telepath_tiles.iter() {
                    let idx = map.xy_idx(telepath_tile.x, telepath_tile.y);
                    // If we didn't already evaluate this idx (if it's not contained in the HashSet),
                    // and it's not the idx we're standing on, then evaluate here w/ minds taken into
                    // account.
                    if this_idx != idx && !idxs.contains(&idx) {
                        evaluate(entity, idx, &ancestries, &factions, &mut reactions, Some(&minds));
                    }
                }
            }
            reactions.sort_by(|(a, _, _), (b, _, _)| {
                let (a_x, a_y) = (a % (map.width as usize), a / (map.width as usize));
                let dist_a = DistanceAlg::PythagorasSquared.distance2d(Point::new(a_x, a_y), Point::new(pos.x, pos.y));
                let dist_a_estimate = dist_a as i32;
                let (b_x, b_y) = (b % (map.width as usize), b / (map.width as usize));
                let dist_b = DistanceAlg::PythagorasSquared.distance2d(Point::new(b_x, b_y), Point::new(pos.x, pos.y));
                let dist_b_estimate = dist_b as i32;
                return dist_b_estimate.cmp(&dist_a_estimate);
            });
            let mut found_flee = false;
            for reaction in reactions.iter() {
                match reaction.1 {
                    Reaction::Attack => {
                        if !found_flee {
                            wants_to_approach
                                .insert(entity, WantsToApproach { idx: reaction.0 as i32 })
                                .expect("Error inserting WantsToApproach");
                            chasing.insert(entity, Chasing { target: reaction.2 }).expect("Unable to insert Chasing");
                            continue;
                        }
                    }
                    Reaction::Flee => {
                        flee.push(reaction.0);
                        found_flee = true;
                    }
                    _ => {}
                }
            }
            if !flee.is_empty() {
                wants_to_flee.insert(entity, WantsToFlee { indices: flee }).expect("Unable to insert");
            }
        }
    }
}

fn evaluate(
    entity: Entity,
    idx: usize,
    ancestries: &ReadStorage<HasAncestry>,
    factions: &ReadStorage<Faction>,
    reactions: &mut Vec<(usize, Reaction, Entity)>,
    minds: Option<&ReadStorage<Mind>>
) {
    crate::spatial::for_each_tile_content(idx, |other_entity| {
        let mut check = true;
        if minds.is_some() {
            console::log("Minds got passed! Evaluating!");
            if minds.unwrap().get(other_entity).is_none() {
                console::log("No brain here. Skipping!");
                check = false;
            }
        }
        if check {
            reactions.push((
                idx,
                crate::raws::get_reactions(
                    entity,
                    other_entity,
                    &factions,
                    &ancestries,
                    &crate::raws::RAWS.lock().unwrap()
                ),
                other_entity,
            ));
        }
    });
}
