use crate::data::entity::*;
use crate::{
    Burden,
    BurdenLevel,
    Clock,
    Energy,
    Name,
    Position,
    RunState,
    Map,
    TakingTurn,
    Confusion,
    Intrinsics,
};
use bracket_lib::prelude::*;
use specs::prelude::*;
use crate::config::CONFIG;
use crate::data::events::*;

pub struct EnergySystem {}

const TURN_COST: i32 = NORMAL_SPEED * TURN_COST_MULTIPLIER;

impl<'a> System<'a> for EnergySystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        ReadExpect<'a, Map>,
        ReadStorage<'a, Clock>,
        WriteStorage<'a, Energy>,
        ReadStorage<'a, Burden>,
        ReadStorage<'a, Position>,
        WriteStorage<'a, TakingTurn>,
        Entities<'a>,
        WriteExpect<'a, RandomNumberGenerator>,
        WriteExpect<'a, RunState>,
        ReadExpect<'a, Entity>,
        ReadStorage<'a, Name>,
        ReadExpect<'a, Point>,
        ReadStorage<'a, Confusion>,
        ReadStorage<'a, Intrinsics>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            map,
            clock,
            mut energies,
            burdens,
            positions,
            mut turns,
            entities,
            mut rng,
            mut runstate,
            player,
            names,
            player_pos,
            confusion,
            intrinsics,
        ) = data;
        // If not ticking, do nothing.
        if *runstate != RunState::Ticking {
            return;
        }
        // Clear TakingTurn{} from every entity.
        turns.clear();
        // TURN COUNTER
        for (entity, _clock, energy) in (&entities, &clock, &mut energies).join() {
            energy.current += NORMAL_SPEED;
            if energy.current >= TURN_COST {
                turns
                    .insert(entity, TakingTurn {})
                    .expect("Unable to insert turn for turn counter.");
                energy.current -= TURN_COST;
                crate::gamelog::record_event(EVENT::Turn(1));
                // Handle spawning mobs each turn
                if CONFIG.logging.log_ticks {
                    console::log(
                        format!(
                            "===== TURN {} =====",
                            crate::gamelog::get_event_count(EVENT::COUNT_TURN)
                        )
                    );
                }
            }
        }
        // EVERYTHING ELSE
        for (entity, energy, pos, _c) in (
            &entities,
            &mut energies,
            &positions,
            !&confusion,
        ).join() {
            let burden_modifier = get_burden_modifier(&burdens, entity);
            let overmap_mod = get_overmap_modifier(&map);
            let intrinsic_speed = get_intrinsic_speed(&intrinsics, entity);
            // Every entity has a POTENTIAL equal to their speed.
            let mut energy_potential: i32 = ((energy.speed as f32) *
                burden_modifier *
                overmap_mod *
                intrinsic_speed) as i32;
            // Increment current energy by NORMAL_SPEED for every
            // whole number of NORMAL_SPEEDS in their POTENTIAL.
            while energy_potential >= NORMAL_SPEED {
                energy_potential -= NORMAL_SPEED;
                energy.current += NORMAL_SPEED;
            }
            // Roll a NORMAL_SPEED-sided die. If less than their
            // remaining POTENTIAL, increment current energy by
            // NORMAL_SPEED.
            // i.e. An entity with a speed of 3/4ths NORMAL_SPEED
            //      will gain NORMAL_SPEED energy in 75% of ticks.
            if energy_potential > 0 {
                if rng.roll_dice(1, NORMAL_SPEED) <= energy_potential {
                    energy.current += NORMAL_SPEED;
                }
            }
            // TURN_COST is equal to 4 * NORMAL_SPEED. If the current entity
            // has enough energy, they take a turn and decrement their energy
            // by TURN_COST. If the current entity is the player, await input.
            if energy.current >= TURN_COST {
                energy.current -= TURN_COST;
                if entity == *player {
                    *runstate = RunState::AwaitingInput;
                } else if cull_turn_by_distance(&player_pos, pos) {
                    continue;
                }
                turns.insert(entity, TakingTurn {}).expect("Unable to insert turn.");
                if CONFIG.logging.log_ticks {
                    let name = if let Some(name) = names.get(entity) {
                        &name.name
                    } else {
                        "Unknown entity"
                    };
                    console::log(
                        format!(
                            "ENERGY SYSTEM: {} granted a turn. [leftover energy: {}].",
                            name,
                            energy.current
                        )
                    );
                }
            }
        }
    }
}

fn get_burden_modifier(burdens: &ReadStorage<Burden>, entity: Entity) -> f32 {
    return if let Some(burden) = burdens.get(entity) {
        match burden.level {
            BurdenLevel::Burdened => SPEED_MOD_BURDENED,
            BurdenLevel::Strained => SPEED_MOD_STRAINED,
            BurdenLevel::Overloaded => SPEED_MOD_OVERLOADED,
        }
    } else {
        1.0
    };
}

fn get_overmap_modifier(map: &ReadExpect<Map>) -> f32 {
    return if map.overmap { SPEED_MOD_OVERMAP_TRAVEL } else { 1.0 };
}

fn cull_turn_by_distance(player_pos: &Point, pos: &Position) -> bool {
    let distance = DistanceAlg::Pythagoras.distance2d(*player_pos, Point::new(pos.x, pos.y));
    if distance > 20.0 {
        return true;
    }
    return false;
}

fn get_intrinsic_speed(intrinsics: &ReadStorage<Intrinsics>, entity: Entity) -> f32 {
    if let Some(intrinsics) = intrinsics.get(entity) {
        if intrinsics.list.contains(&crate::Intrinsic::Speed) {
            return 4.0 / 3.0;
        }
    }
    return 1.0;
}
