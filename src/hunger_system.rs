use super::{
    effects::{add_effect, EffectType, Targets},
    gamelog, Clock, HungerClock, HungerState, TakingTurn, LOG_TICKS,
};
use rltk::prelude::*;
use specs::prelude::*;

/// HungerSystem is in charge of ticking down the hunger clock for entities with a hunger clock,
/// every time the turn clock ticks.
pub struct HungerSystem {}

const MAX_SATIATION: i32 = 2000;
const HUNGER_BREAKPOINTS: [(i32, HungerState); 5] = [
    (1000, HungerState::Satiated),
    (600, HungerState::Normal),
    (400, HungerState::Hungry),
    (200, HungerState::Weak),
    (0, HungerState::Fainting),
];
const BASE_CLOCK_DECREMENT_PER_TURN: i32 = 4;

pub fn get_hunger_state(duration: i32) -> HungerState {
    for (threshold, state) in HUNGER_BREAKPOINTS.iter() {
        if duration > *threshold {
            return *state;
        }
    }
    return HungerState::Starving;
}

pub fn get_hunger_colour(state: HungerState) -> (u8, u8, u8) {
    match state {
        HungerState::Satiated => GREEN,
        HungerState::Normal => WHITE,
        HungerState::Hungry => BROWN1,
        HungerState::Weak => ORANGE,
        HungerState::Fainting => RED3,
        HungerState::Starving => RED,
    }
}

impl<'a> System<'a> for HungerSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, HungerClock>,
        ReadExpect<'a, Entity>,
        ReadStorage<'a, Clock>,
        ReadStorage<'a, TakingTurn>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, mut hunger_clock, player_entity, turn_clock, turns) = data;

        // If the turn clock isn't taking a turn this tick, don't bother ticking hunger.
        let mut ticked = false;
        for (_e, _c, _t) in (&entities, &turn_clock, &turns).join() {
            ticked = true;
            break;
        }
        if !ticked {
            return;
        }
        // Otherwise, tick down the hunger clock for all entities with one.
        for (entity, mut hunger_clock) in (&entities, &mut hunger_clock).join() {
            if hunger_clock.duration >= MAX_SATIATION {
                hunger_clock.duration = MAX_SATIATION;
            } else {
                hunger_clock.duration -= BASE_CLOCK_DECREMENT_PER_TURN;
            }
            let initial_state = hunger_clock.state;
            hunger_clock.state = get_hunger_state(hunger_clock.duration);
            if hunger_clock.state == HungerState::Starving {
                add_effect(None, EffectType::Damage { amount: 1 }, Targets::Entity { target: entity });
            }
            if LOG_TICKS && entity == *player_entity {
                rltk::console::log(format!(
                    "HUNGER SYSTEM: Ticked for player entity. [clock: {}]",
                    hunger_clock.duration
                ));
            }
            if hunger_clock.state == initial_state {
                continue;
            }
            if entity != *player_entity {
                continue;
            }
            // Things which only happen to the player.
            match hunger_clock.state {
                HungerState::Satiated => gamelog::Logger::new()
                    .append("You feel")
                    .colour(get_hunger_colour(hunger_clock.state))
                    .append_n("satiated")
                    .colour(WHITE)
                    .period()
                    .log(),
                HungerState::Normal => {}
                HungerState::Hungry => gamelog::Logger::new()
                    .append("You feel")
                    .colour(get_hunger_colour(hunger_clock.state))
                    .append_n("hungry")
                    .colour(WHITE)
                    .period()
                    .log(),
                HungerState::Weak => gamelog::Logger::new()
                    .append("You feel")
                    .colour(get_hunger_colour(hunger_clock.state))
                    .append_n("weak with hunger")
                    .colour(WHITE)
                    .period()
                    .log(),
                HungerState::Fainting => gamelog::Logger::new()
                    .append("You feel")
                    .colour(get_hunger_colour(hunger_clock.state))
                    .append_n("hungry enough to faint")
                    .colour(WHITE)
                    .period()
                    .log(),
                _ => gamelog::Logger::new()
                    .colour(get_hunger_colour(hunger_clock.state))
                    .append_n("You can't go on without food!")
                    .log(),
            }
        }
    }
}
