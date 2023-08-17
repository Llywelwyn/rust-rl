use crate::{
    effects::{add_effect, EffectType, Targets},
    gamelog,
    gui::renderable_colour,
    Clock, Confusion, Name, Renderable, TakingTurn,
};
use rltk::prelude::*;
use specs::prelude::*;

pub struct TurnStatusSystem {}

impl<'a> System<'a> for TurnStatusSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        WriteStorage<'a, TakingTurn>,
        ReadStorage<'a, Clock>,
        WriteStorage<'a, Confusion>,
        Entities<'a>,
        ReadStorage<'a, Name>,
        ReadExpect<'a, Entity>,
        ReadStorage<'a, Renderable>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut turns, clock, mut confusion, entities, names, player_entity, renderables) = data;
        let mut clock_tick = false;
        for (_e, _c, _t) in (&entities, &clock, &turns).join() {
            clock_tick = true;
        }
        if !clock_tick {
            return;
        }
        let mut logger = gamelog::Logger::new();
        let mut log = false;
        let mut not_my_turn: Vec<Entity> = Vec::new();
        let mut not_confused: Vec<Entity> = Vec::new();
        for (entity, _turn, confused, name) in (&entities, &mut turns, &mut confusion, &names).join() {
            log = true;
            confused.turns -= 1;
            if confused.turns < 1 {
                not_confused.push(entity);
                if entity == *player_entity {
                    logger = logger
                        .colour(renderable_colour(&renderables, entity))
                        .append(&name.name)
                        .colour(WHITE)
                        .append("snap out of it.");
                } else {
                    logger = logger
                        .append("The")
                        .colour(renderable_colour(&renderables, entity))
                        .append(&name.name)
                        .colour(WHITE)
                        .append("snaps out of it.");
                }
                add_effect(
                    None,
                    EffectType::Particle {
                        glyph: to_cp437('!'),
                        fg: RGB::named(LIGHT_BLUE),
                        bg: RGB::named(BLACK),
                        lifespan: 200.0,
                        delay: 0.0,
                    },
                    Targets::Entity { target: entity },
                );
            } else {
                not_my_turn.push(entity);
                if entity == *player_entity {
                    logger = logger
                        .colour(renderable_colour(&renderables, entity))
                        .append(&name.name)
                        .colour(WHITE)
                        .append("are confused!");
                } else {
                    logger = logger
                        .append("The")
                        .colour(renderable_colour(&renderables, entity))
                        .append(&name.name)
                        .colour(WHITE)
                        .append("is confused!");
                }
                add_effect(
                    None,
                    EffectType::Particle {
                        glyph: to_cp437('?'),
                        fg: RGB::named(MAGENTA),
                        bg: RGB::named(BLACK),
                        lifespan: 200.0,
                        delay: 0.0,
                    },
                    Targets::Entity { target: entity },
                );
            }
        }
        if log {
            logger.log();
        }
        for e in not_my_turn {
            turns.remove(e);
        }
        for e in not_confused {
            confusion.remove(e);
        }
    }
}
