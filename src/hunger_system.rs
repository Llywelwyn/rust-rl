use super::{
    effects::{add_effect, EffectType, Targets},
    gamelog, HungerClock, HungerState, LOG_TICKS,
};
use specs::prelude::*;

pub struct HungerSystem {}

impl<'a> System<'a> for HungerSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (Entities<'a>, WriteStorage<'a, HungerClock>, ReadExpect<'a, Entity>);

    fn run(&mut self, data: Self::SystemData) {
        let (entities, mut hunger_clock, player_entity) = data;

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
                        gamelog::Logger::new().colour(rltk::BROWN1).append("You feel hungry.").log();
                    }
                }
                HungerState::Hungry => {
                    clock.state = HungerState::Weak;
                    clock.duration = 200;
                    if entity == *player_entity {
                        gamelog::Logger::new().colour(rltk::ORANGE).append("You feel weak with hunger.").log();
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
                    add_effect(None, EffectType::Damage { amount: 1 }, Targets::Entity { target: entity });
                    if entity == *player_entity {
                        gamelog::Logger::new().colour(rltk::RED).append("You can't go on without food...").log();
                    }
                }
            }
        }
    }
}
