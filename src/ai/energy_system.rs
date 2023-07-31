use crate::{Energy, Position, RunState, TakingTurn};
use rltk::prelude::*;
use specs::prelude::*;

pub struct EnergySystem {}

const NORMAL_SPEED: i32 = 12;
const TURN_COST: i32 = NORMAL_SPEED * 4;

impl<'a> System<'a> for EnergySystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        WriteStorage<'a, Energy>,
        ReadStorage<'a, Position>,
        WriteStorage<'a, TakingTurn>,
        Entities<'a>,
        WriteExpect<'a, RandomNumberGenerator>,
        WriteExpect<'a, RunState>,
        ReadExpect<'a, Entity>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut energies, positions, mut turns, entities, mut rng, mut runstate, player) = data;

        if *runstate != RunState::MonsterTurn {
            return;
        }

        turns.clear();

        for (entity, energy, _pos) in (&entities, &mut energies, &positions).join() {
            let mut energy_potential: i32 = energy.speed;
            while energy_potential >= NORMAL_SPEED {
                energy_potential -= NORMAL_SPEED;
                energy.current += NORMAL_SPEED;
            }
            if energy_potential > 0 {
                if rng.roll_dice(1, NORMAL_SPEED) <= energy_potential {
                    energy.current += NORMAL_SPEED;
                }
            }
            if energy.current >= TURN_COST {
                turns.insert(entity, TakingTurn {}).expect("Unable to insert turn.");
                energy.current -= TURN_COST;
                if entity == *player {
                    *runstate = RunState::AwaitingInput;
                }
            }
        }
    }
}
