use super::{gamelog, HungerClock, HungerState, RunState, SufferDamage};
use specs::prelude::*;

pub struct HungerSystem {}

impl<'a> System<'a> for HungerSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, HungerClock>,
        ReadExpect<'a, Entity>,
        ReadExpect<'a, RunState>,
        WriteStorage<'a, SufferDamage>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, mut hunger_clock, player_entity, runstate, mut inflict_damage) = data;

        for (entity, mut clock) in (&entities, &mut hunger_clock).join() {
            let mut proceed = false;

            match *runstate {
                RunState::PlayerTurn => {
                    if entity == *player_entity {
                        proceed = true;
                    }
                }
                RunState::MonsterTurn => {
                    if entity != *player_entity {
                        proceed = true;
                    }
                }
                _ => proceed = false,
            }

            if !proceed {
                return;
            }
            clock.duration -= 1;
            if clock.duration > 0 {
                return;
            }

            match clock.state {
                HungerState::Satiated => {
                    clock.state = HungerState::Normal;
                    clock.duration = 300;
                    if entity == *player_entity {
                        gamelog::Logger::new().append("You are no longer satiated.").log();
                    }
                }
                HungerState::Normal => {
                    clock.state = HungerState::Hungry;
                    clock.duration = 100;
                    if entity == *player_entity {
                        gamelog::Logger::new().colour(rltk::RED).append("You feel hungry.").log();
                    }
                }
                HungerState::Hungry => {
                    clock.state = HungerState::Weak;
                    clock.duration = 50;
                    if entity == *player_entity {
                        gamelog::Logger::new().colour(rltk::RED).append("You feel weak with hunger.").log();
                    }
                }
                HungerState::Weak => {
                    clock.state = HungerState::Fainting;
                    clock.duration = 50;
                    if entity == *player_entity {
                        gamelog::Logger::new().colour(rltk::RED).append("You feel hungry enough to faint.").log();
                    }
                }
                HungerState::Fainting => {
                    SufferDamage::new_damage(&mut inflict_damage, entity, 1, false);
                    if entity == *player_entity {
                        gamelog::Logger::new().colour(rltk::RED).append("You can't go on without food...").log();
                    }
                }
            }
        }
    }
}
