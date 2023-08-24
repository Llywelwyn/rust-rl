use crate::config::entity::*;
use crate::{ Burden, BurdenLevel, Clock, Energy, Name, Position, RunState, TakingTurn };
use rltk::prelude::*;
use specs::prelude::*;
use crate::config::CONFIG;

pub struct EnergySystem {}

const TURN_COST: i32 = NORMAL_SPEED * TURN_COST_MULTIPLIER;

impl<'a> System<'a> for EnergySystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
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
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
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
                turns.insert(entity, TakingTurn {}).expect("Unable to insert turn for turn counter.");
                energy.current -= TURN_COST;
                crate::gamelog::record_event("turns", 1);
                // Handle spawning mobs each turn
                if CONFIG.logging.log_ticks {
                    console::log(format!("===== TURN {} =====", crate::gamelog::get_event_count("turns")));
                }
            }
        }
        // EVERYTHING ELSE
        for (entity, energy, pos) in (&entities, &mut energies, &positions).join() {
            let burden_modifier = if let Some(burden) = burdens.get(entity) {
                match burden.level {
                    BurdenLevel::Burdened => 0.75,
                    BurdenLevel::Strained => 0.5,
                    BurdenLevel::Overloaded => 0.25,
                }
            } else {
                1.0
            };
            // Every entity has a POTENTIAL equal to their speed.
            let mut energy_potential: i32 = ((energy.speed as f32) * burden_modifier) as i32;
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
                let mut my_turn = true;
                energy.current -= TURN_COST;
                if entity == *player {
                    *runstate = RunState::AwaitingInput;
                } else {
                    let distance = rltk::DistanceAlg::Pythagoras.distance2d(*player_pos, Point::new(pos.x, pos.y));
                    if distance > 20.0 {
                        my_turn = false;
                    }
                }
                if my_turn {
                    turns.insert(entity, TakingTurn {}).expect("Unable to insert turn.");
                    if CONFIG.logging.log_ticks {
                        let name = if let Some(name) = names.get(entity) { &name.name } else { "Unknown entity" };
                        console::log(
                            format!("ENERGY SYSTEM: {} granted a turn. [leftover energy: {}].", name, energy.current)
                        );
                    }
                }
            }
        }
    }
}
