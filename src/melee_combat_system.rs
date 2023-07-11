use super::{gamelog, CombatStats, Name, ParticleBuilder, Position, SufferDamage, WantsToMelee};
use specs::prelude::*;

pub struct MeleeCombatSystem {}

impl<'a> System<'a> for MeleeCombatSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, WantsToMelee>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, CombatStats>,
        WriteStorage<'a, SufferDamage>,
        WriteExpect<'a, ParticleBuilder>,
        ReadStorage<'a, Position>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, mut wants_melee, names, combat_stats, mut inflict_damage, mut particle_builder, positions) =
            data;

        for (_entity, wants_melee, name, stats) in (&entities, &wants_melee, &names, &combat_stats).join() {
            if stats.hp <= 0 {
                break;
            }
            let target_stats = combat_stats.get(wants_melee.target).unwrap();
            if target_stats.hp <= 0 {
                break;
            }

            let target_name = names.get(wants_melee.target).unwrap();
            let pos = positions.get(wants_melee.target);
            if let Some(pos) = pos {
                particle_builder.request(
                    pos.x,
                    pos.y,
                    rltk::RGB::named(rltk::ORANGE),
                    rltk::RGB::named(rltk::BLACK),
                    rltk::to_cp437('‼'),
                    150.0,
                );
            }
            let damage = i32::max(0, stats.power - target_stats.defence);

            if damage == 0 {
                gamelog::Logger::new()
                    .append("The")
                    .npc_name(&name.name)
                    .colour(rltk::WHITE)
                    .append("attempts to strike")
                    .npc_name(&target_name.name)
                    .colour(rltk::WHITE)
                    .append("- but fails.")
                    .log();
            } else {
                gamelog::Logger::new() // <name> hits the <name>!
                    .append("The")
                    .npc_name(&name.name)
                    .colour(rltk::WHITE)
                    .append("hits the")
                    .npc_name_n(format!("{}", &target_name.name))
                    .period()
                    .log();
                SufferDamage::new_damage(&mut inflict_damage, wants_melee.target, damage);
            }
        }

        wants_melee.clear();
    }
}
