use super::{
    gamelog, Attributes, GrantsXP, Item, Map, Name, ParticleBuilder, Player, Pools, Position, RunState, SufferDamage,
};
use crate::gamesystem::{mana_per_level, player_hp_per_level};
use specs::prelude::*;

pub struct DamageSystem {}

impl<'a> System<'a> for DamageSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        WriteStorage<'a, Pools>,
        WriteStorage<'a, SufferDamage>,
        ReadStorage<'a, Position>,
        WriteExpect<'a, Map>,
        Entities<'a>,
        ReadExpect<'a, Entity>,
        ReadStorage<'a, Attributes>,
        WriteExpect<'a, rltk::RandomNumberGenerator>,
        ReadStorage<'a, GrantsXP>,
        WriteExpect<'a, ParticleBuilder>,
        ReadExpect<'a, rltk::Point>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            mut stats,
            mut damage,
            positions,
            mut map,
            entities,
            player,
            attributes,
            mut rng,
            xp_granters,
            mut particle_builder,
            player_pos,
        ) = data;
        let mut xp_gain = 0;

        for (entity, mut stats, damage) in (&entities, &mut stats, &damage).join() {
            for dmg in damage.amount.iter() {
                stats.hit_points.current -= dmg.0;
                let pos = positions.get(entity);
                if let Some(pos) = pos {
                    let idx = map.xy_idx(pos.x, pos.y);
                    map.bloodstains.insert(idx);
                }

                if stats.hit_points.current < 1 && dmg.1 {
                    let gives_xp = xp_granters.get(entity);
                    if let Some(xp_value) = gives_xp {
                        xp_gain += xp_value.amount;
                    }
                }
            }
        }

        if xp_gain != 0 {
            let mut player_stats = stats.get_mut(*player).unwrap();
            let player_attributes = attributes.get(*player).unwrap();
            player_stats.xp += xp_gain;
            rltk::console::log(xp_gain);

            let mut next_level_requirement = -1;
            if player_stats.level < 10 {
                next_level_requirement = 20 * 2_i32.pow(player_stats.level as u32 - 1);
            } else if player_stats.level < 20 {
                next_level_requirement = 10000 * 2_i32.pow(player_stats.level as u32 - 10);
            } else if player_stats.level < 30 {
                next_level_requirement = 10000000 * (player_stats.level - 19);
            }

            if next_level_requirement != -1 && player_stats.xp >= next_level_requirement {
                // We've gone up a level!
                player_stats.level += 1;
                gamelog::Logger::new()
                    .append("Welcome to experience level")
                    .append(player_stats.level)
                    .append(".")
                    .log();
                for i in 0..5 {
                    if player_pos.y - i > 1 {
                        particle_builder.request(
                            player_pos.x,
                            player_pos.y - i,
                            rltk::RGB::named(rltk::GOLD),
                            rltk::RGB::named(rltk::BLACK),
                            rltk::to_cp437('*'),
                            200.0,
                        );
                    }
                }
                let hp_gained = player_hp_per_level(
                    &mut rng,
                    player_attributes.constitution.base + player_attributes.constitution.modifiers,
                );
                player_stats.hit_points.max += hp_gained;
                player_stats.hit_points.current += hp_gained;

                let mana_gained = mana_per_level(
                    &mut rng,
                    player_attributes.intelligence.base + player_attributes.intelligence.modifiers,
                );
                player_stats.mana.max += mana_gained;
                player_stats.mana.current += mana_gained;
            }
        }

        damage.clear();
    }
}

pub fn delete_the_dead(ecs: &mut World) {
    let mut dead: Vec<Entity> = Vec::new();
    // Using scope to make borrow checker happy
    {
        let combat_stats = ecs.read_storage::<Pools>();
        let players = ecs.read_storage::<Player>();
        let names = ecs.read_storage::<Name>();
        let items = ecs.read_storage::<Item>();
        let entities = ecs.entities();
        for (entity, stats) in (&entities, &combat_stats).join() {
            if stats.hit_points.current < 1 {
                let player = players.get(entity);
                match player {
                    None => {
                        let victim_name = names.get(entity);
                        if let Some(victim_name) = victim_name {
                            let item = items.get(entity);
                            if let Some(_item) = item {
                                gamelog::Logger::new()
                                    .append("The")
                                    .npc_name(&victim_name.name)
                                    .colour(rltk::WHITE)
                                    .append("is destroyed!")
                                    .log();
                            } else {
                                gamelog::Logger::new()
                                    .append("The")
                                    .npc_name(&victim_name.name)
                                    .colour(rltk::WHITE)
                                    .append("dies!")
                                    .log();
                            }
                        }
                        dead.push(entity)
                    }
                    Some(_) => {
                        let mut runstate = ecs.write_resource::<RunState>();
                        *runstate = RunState::GameOver;
                    }
                }
            }
        }
    }

    for victim in dead {
        gamelog::record_event("death_count", 1);
        ecs.delete_entity(victim).expect("Unable to delete.");
    }
}
