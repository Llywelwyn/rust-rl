use super::{
    gamelog, CombatStats, DefenceBonus, Equipped, HungerClock, HungerState, MeleePowerBonus, Name, ParticleBuilder,
    Position, SufferDamage, WantsToMelee,
};
use specs::prelude::*;

pub struct MeleeCombatSystem {}

impl<'a> System<'a> for MeleeCombatSystem {
    type SystemData = (
        Entities<'a>,
        ReadExpect<'a, Entity>,
        WriteStorage<'a, WantsToMelee>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, CombatStats>,
        WriteStorage<'a, SufferDamage>,
        WriteExpect<'a, ParticleBuilder>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, Equipped>,
        ReadStorage<'a, DefenceBonus>,
        ReadStorage<'a, MeleePowerBonus>,
        ReadStorage<'a, HungerClock>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            entities,
            player_entity,
            mut wants_melee,
            names,
            combat_stats,
            mut inflict_damage,
            mut particle_builder,
            positions,
            equipped,
            defence_bonuses,
            melee_power_bonuses,
            hunger_clock,
        ) = data;

        for (entity, wants_melee, name, stats) in (&entities, &wants_melee, &names, &combat_stats).join() {
            if stats.hp <= 0 {
                break;
            }
            let target_stats = combat_stats.get(wants_melee.target).unwrap();
            if target_stats.hp <= 0 {
                break;
            }

            let target_name = names.get(wants_melee.target).unwrap();

            let mut offensive_bonus = 0;
            for (_item_entity, power_bonus, equipped_by) in (&entities, &melee_power_bonuses, &equipped).join() {
                if equipped_by.owner == entity {
                    offensive_bonus += power_bonus.amount;
                }
            }
            let mut defensive_bonus = 0;
            for (_item_entity, defence_bonus, equipped_by) in (&entities, &defence_bonuses, &equipped).join() {
                if equipped_by.owner == wants_melee.target {
                    defensive_bonus += defence_bonus.amount;
                }
            }
            let hc = hunger_clock.get(entity);
            if let Some(hc) = hc {
                match hc.state {
                    HungerState::Satiated => {
                        offensive_bonus += 1;
                    }
                    HungerState::Weak => {
                        offensive_bonus -= 1;
                    }
                    HungerState::Fainting => {
                        offensive_bonus -= 1;
                        defensive_bonus -= 1;
                    }
                    _ => {}
                }
            }
            let damage = i32::max(0, (stats.power + offensive_bonus) - (target_stats.defence + defensive_bonus));

            if damage == 0 {
                if entity == *player_entity {
                    gamelog::Logger::new() // You miss.
                        .append("You miss.")
                        .log();
                } else if wants_melee.target == *player_entity {
                    gamelog::Logger::new() // <name> misses!
                        .append("The")
                        .npc_name(&name.name)
                        .colour(rltk::WHITE)
                        .append("misses!")
                        .log();
                } else {
                    gamelog::Logger::new() // <name> misses the <target>.
                        .append("The")
                        .npc_name(&name.name)
                        .colour(rltk::WHITE)
                        .append("misses the")
                        .npc_name_n(&target_name.name)
                        .period()
                        .log();
                }
            } else {
                if entity == *player_entity {
                    gamelog::Logger::new() // You hit the <name>.
                        .append("You hit the")
                        .npc_name_n(&target_name.name)
                        .period()
                        .log();
                } else if wants_melee.target == *player_entity {
                    gamelog::Logger::new() // <name> hits you!
                        .append("The")
                        .npc_name(&name.name)
                        .colour(rltk::WHITE)
                        .append("hits you!")
                        .log();
                } else {
                    gamelog::Logger::new() // <name> misses the <target>.
                        .append("The")
                        .npc_name(&name.name)
                        .colour(rltk::WHITE)
                        .append("hits the")
                        .npc_name_n(&target_name.name)
                        .period()
                        .log();
                }
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
                SufferDamage::new_damage(&mut inflict_damage, wants_melee.target, damage);
            }
        }

        wants_melee.clear();
    }
}
