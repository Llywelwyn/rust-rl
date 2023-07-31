use crate::{gamelog, Confusion, Name, ParticleBuilder, Position, RunState, TakingTurn};
use rltk::prelude::*;
use specs::prelude::*;

pub struct TurnStatusSystem {}

impl<'a> System<'a> for TurnStatusSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        WriteStorage<'a, TakingTurn>,
        WriteStorage<'a, Confusion>,
        Entities<'a>,
        ReadExpect<'a, RunState>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, Position>,
        WriteExpect<'a, ParticleBuilder>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut turns, mut confusion, entities, runstate, names, positions, mut particle_builder) = data;
        if *runstate != RunState::Ticking {
            return;
        }
        let mut remove_turn: Vec<Entity> = Vec::new();
        let mut remove_confusion: Vec<Entity> = Vec::new();
        for (entity, _turn, confused, name, pos) in (&entities, &mut turns, &mut confusion, &names, &positions).join() {
            confused.turns -= 1;
            if confused.turns < 1 {
                remove_confusion.push(entity);
                gamelog::Logger::new().npc_name(&name.name).colour(WHITE).append("snaps out of it.").log();
                particle_builder.request(pos.x, pos.y, RGB::named(LIGHT_BLUE), RGB::named(BLACK), to_cp437('!'), 200.0);
            } else {
                remove_turn.push(entity);
                gamelog::Logger::new().npc_name(&name.name).colour(WHITE).append("is confused.").log();
                particle_builder.request(pos.x, pos.y, RGB::named(MAGENTA), RGB::named(BLACK), to_cp437('?'), 200.0);
            }
        }
        for e in remove_turn {
            turns.remove(e);
        }
        for e in remove_confusion {
            confusion.remove(e);
        }
    }
}
