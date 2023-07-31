use crate::{gamelog, Name, Quips, TakingTurn, Viewshed};
use rltk::prelude::*;
use specs::prelude::*;

pub struct QuipSystem {}

impl<'a> System<'a> for QuipSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        WriteStorage<'a, Quips>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, TakingTurn>,
        ReadExpect<'a, Point>,
        ReadStorage<'a, Viewshed>,
        WriteExpect<'a, RandomNumberGenerator>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut quips, names, turns, player_pos, viewsheds, mut rng) = data;
        for (quip, name, viewshed, _turn) in (&mut quips, &names, &viewsheds, &turns).join() {
            if !quip.available.is_empty() && viewshed.visible_tiles.contains(&player_pos) && rng.roll_dice(1, 6) == 1 {
                let quip_index = if quip.available.len() == 1 {
                    0
                } else {
                    (rng.roll_dice(1, quip.available.len() as i32) - 1) as usize
                };
                gamelog::Logger::new()
                    .append("The")
                    .npc_name(&name.name)
                    .append_n("says \"")
                    .append_n(&quip.available[quip_index])
                    .append("\"")
                    .log();
                quip.available.remove(quip_index);
            }
        }
    }
}
