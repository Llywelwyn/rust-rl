use crate::{Clock, Energy, Name, Position, RunState, TakingTurn, LOG_TICKS};
use rltk::prelude::*;
use specs::prelude::*;

pub struct EnergySystem {}

pub const NORMAL_SPEED: i32 = 12;
const TURN_COST: i32 = NORMAL_SPEED * 4;

impl<'a> System<'a> for EnergySystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        ReadStorage<'a, Clock>,
        WriteStorage<'a, Energy>,
        ReadStorage<'a, Position>,
        WriteStorage<'a, TakingTurn>,
        Entities<'a>,
        WriteExpect<'a, RandomNumberGenerator>,
        WriteExpect<'a, RunState>,
        ReadExpect<'a, Entity>,
        ReadStorage<'a, Name>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (clock, mut energies, positions, mut turns, entities, mut rng, mut runstate, player, names) = data;
        // If not ticking, do nothing.
        if *runstate != RunState::Ticking {
            return;
        }
        for (_entity, _clock, energy) in (&entities, &clock, &mut energies).join() {
            energy.current += NORMAL_SPEED;
            if energy.current >= TURN_COST {
                energy.current -= TURN_COST;
                crate::gamelog::record_event("turns", 1);
                if LOG_TICKS {
                    console::log(format!("===== TURN {} =====", crate::gamelog::get_event_count("turns")));
                }
            }
        }
        // Clear TakingTurn{} from every entity.
        turns.clear();
        for (entity, energy, _pos) in (&entities, &mut energies, &positions).join() {
            // Every entity has a POTENTIAL equal to their speed.
            let mut energy_potential: i32 = energy.speed;
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
                turns.insert(entity, TakingTurn {}).expect("Unable to insert turn.");
                energy.current -= TURN_COST;
                if LOG_TICKS {
                    let name = if let Some(name) = names.get(entity) { &name.name } else { "Unknown entity" };
                    console::log(format!(
                        "ENERGY SYSTEM: {} granted a turn. [leftover energy: {}].",
                        name, energy.current
                    ));
                }
                if entity == *player {
                    *runstate = RunState::AwaitingInput;
                }
            }
        }
    }
}
