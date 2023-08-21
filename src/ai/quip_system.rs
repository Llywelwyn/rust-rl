use crate::{gamelog, gui::renderable_colour, Name, Quips, Renderable, TakingTurn, Viewshed};
use rltk::prelude::*;
use specs::prelude::*;

pub struct QuipSystem {}

impl<'a> System<'a> for QuipSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, Quips>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, Renderable>,
        ReadStorage<'a, TakingTurn>,
        ReadExpect<'a, Point>,
        ReadStorage<'a, Viewshed>,
        WriteExpect<'a, RandomNumberGenerator>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, mut quips, names, renderables, turns, player_pos, viewsheds, mut rng) = data;
        for (entity, quip, name, viewshed, _turn) in (&entities, &mut quips, &names, &viewsheds, &turns).join() {
            if !quip.available.is_empty() && viewshed.visible_tiles.contains(&player_pos) && rng.roll_dice(1, 6) == 1 {
                let quip_index = if quip.available.len() == 1 {
                    0
                } else {
                    (rng.roll_dice(1, quip.available.len() as i32) - 1) as usize
                };
                gamelog::Logger::new()
                    .append("The")
                    .colour(renderable_colour(&renderables, entity))
                    .append(&name.name)
                    .colour(WHITE)
                    .append_n("says \"")
                    .append_n(&quip.available[quip_index])
                    .append("\"")
                    .log();
                quip.available.remove(quip_index);
            }
        }
    }
}
