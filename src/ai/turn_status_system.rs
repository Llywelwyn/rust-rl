use crate::{
    effects::{ add_effect, EffectType, Targets },
    gamelog,
    gui::renderable_colour,
    Clock,
    Confusion,
    Name,
    Renderable,
    TakingTurn,
    Item,
    Prop,
};
use rltk::prelude::*;
use specs::prelude::*;
use crate::data::events::*;

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
        ReadStorage<'a, Item>,
        ReadStorage<'a, Prop>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            mut turns,
            clock,
            mut confusion,
            entities,
            names,
            player_entity,
            renderables,
            items,
            props,
        ) = data;
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
        for (entity, confused, name, _i, _p) in (
            &entities,
            &mut confusion,
            &names,
            !&items,
            !&props,
        ).join() {
            confused.turns -= 1;
            if confused.turns < 1 {
                not_confused.push(entity);
                if entity == *player_entity {
                    logger = logger
                        .colour(renderable_colour(&renderables, entity))
                        .append("You")
                        .colour(WHITE)
                        .append("snap out of it.");
                    log = true;
                } else {
                    logger = logger
                        .append("The")
                        .colour(renderable_colour(&renderables, entity))
                        .append(&name.name)
                        .colour(WHITE)
                        .append("snaps out of it.");
                    log = true;
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
                    Targets::Entity { target: entity }
                );
            } else {
                not_my_turn.push(entity);
                if entity == *player_entity {
                    logger = logger
                        .colour(renderable_colour(&renderables, entity))
                        .append("You")
                        .colour(WHITE)
                        .append("are confused!");
                    log = true;
                    gamelog::record_event(EVENT::PLAYER_CONFUSED(1));
                } else {
                    logger = logger
                        .append("The")
                        .colour(renderable_colour(&renderables, entity))
                        .append(&name.name)
                        .colour(WHITE)
                        .append("is confused!");
                    log = true;
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
                    Targets::Entity { target: entity }
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
