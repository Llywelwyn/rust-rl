use super::{gamelog, HungerClock, HungerState, SufferDamage, LOG_TICKS};
use specs::prelude::*;

pub struct HungerSystem {}

impl<'a> System<'a> for HungerSystem {
    #[allow(clippy::type_complexity)]
    type SystemData =
        (Entities<'a>, WriteStorage<'a, HungerClock>, ReadExpect<'a, Entity>, WriteStorage<'a, SufferDamage>);

    fn run(&mut self, data: Self::SystemData) {
        let (entities, mut hunger_clock, player_entity, mut inflict_damage) = data;

        for (entity, mut clock) in (&entities, &mut hunger_clock).join() {
            if LOG_TICKS && entity == *player_entity {
                rltk::console::log(format!("HUNGER SYSTEM: Ticked for player entity. [clock: {}]", clock.duration));
            }
            clock.duration -= 1;
            if clock.duration > 0 {
                return;
            }

            match clock.state {
                HungerState::Satiated => {
                    clock.state = HungerState::Normal;
                    clock.duration = 1200;
                    if entity == *player_entity {
                        gamelog::Logger::new().append("You are no longer satiated.").log();
                    }
                }
                HungerState::Normal => {
                    clock.state = HungerState::Hungry;
                    clock.duration = 400;
                    if entity == *player_entity {
                        gamelog::Logger::new().colour(rltk::RED).append("You feel hungry.").log();
                    }
                }
                HungerState::Hungry => {
                    clock.state = HungerState::Weak;
                    clock.duration = 200;
                    if entity == *player_entity {
                        gamelog::Logger::new().colour(rltk::RED).append("You feel weak with hunger.").log();
                    }
                }
                HungerState::Weak => {
                    clock.state = HungerState::Fainting;
                    clock.duration = 200;
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
