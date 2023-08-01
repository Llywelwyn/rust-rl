use super::{
    gamelog, Attributes, Equipped, GrantsXP, InBackpack, Item, LootTable, Map, Name, ParticleBuilder, Player, Pools,
    Position, RunState, SufferDamage,
};
use crate::gamesystem::{mana_per_level, player_hp_per_level};
use rltk::prelude::*;
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
                gamelog::record_event("player_level", 1);
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
                // Roll for HP gain this level
                let hp_gained = player_hp_per_level(
                    &mut rng,
                    player_attributes.constitution.base + player_attributes.constitution.modifiers,
                );
                player_stats.hit_points.max += hp_gained;
                player_stats.hit_points.current += hp_gained;
                // Roll for MANA gain this level
                let mana_gained = mana_per_level(
                    &mut rng,
                    player_attributes.intelligence.base + player_attributes.intelligence.modifiers,
                );
                player_stats.mana.max += mana_gained;
                player_stats.mana.current += mana_gained;
            }
        }
        // Clear the queue
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
                    // The player died, go to GameOver.
                    Some(_) => {
                        let mut runstate = ecs.write_resource::<RunState>();
                        *runstate = RunState::GameOver;
                    }
                }
            }
        }
    }
    let (items_to_delete, loot_to_spawn) = handle_dead_entity_items(ecs, &dead);
    for loot in loot_to_spawn {
        crate::raws::spawn_named_entity(
            &crate::raws::RAWS.lock().unwrap(),
            ecs,
            &loot.0,
            crate::raws::SpawnType::AtPosition { x: loot.1.x, y: loot.1.y },
            0,
        );
    }
    for item in items_to_delete {
        ecs.delete_entity(item).expect("Unable to delete item.");
    }
    // For everything that died, increment the event log, and delete.
    for victim in dead {
        gamelog::record_event("death_count", 1);
        ecs.delete_entity(victim).expect("Unable to delete.");
    }
}

fn handle_dead_entity_items(ecs: &mut World, dead: &Vec<Entity>) -> (Vec<Entity>, Vec<(String, Position)>) {
    let mut to_drop: Vec<(Entity, Position)> = Vec::new();
    let mut to_spawn: Vec<(String, Position)> = Vec::new();
    let entities = ecs.entities();
    let mut equipped = ecs.write_storage::<Equipped>();
    let mut carried = ecs.write_storage::<InBackpack>();
    let mut positions = ecs.write_storage::<Position>();
    let loot_tables = ecs.read_storage::<LootTable>();
    let mut rng = ecs.write_resource::<RandomNumberGenerator>();
    // Make list of every item in every dead thing's inv/equip
    for victim in dead.iter() {
        let pos = positions.get(*victim);
        for (entity, equipped) in (&entities, &equipped).join() {
            if equipped.owner == *victim {
                // Push equipped item entities and positions
                if let Some(pos) = pos {
                    to_drop.push((entity, pos.clone()));
                }
            }
        }
        for (entity, backpack) in (&entities, &carried).join() {
            if backpack.owner == *victim {
                // Push backpack item entities and positions
                if let Some(pos) = pos {
                    to_drop.push((entity, pos.clone()));
                }
            }
        }
        if let Some(table) = loot_tables.get(*victim) {
            let roll: f32 = rng.rand();
            if roll < table.chance {
                let potential_drop =
                    crate::raws::roll_on_loot_table(&crate::raws::RAWS.lock().unwrap(), &mut rng, &table.table);
                if let Some(id) = potential_drop {
                    if let Some(pos) = pos {
                        to_spawn.push((id, pos.clone()));
                    }
                }
            }
        }
    }
    const DROP_ONE_IN_THIS_MANY_TIMES: i32 = 6;
    let mut to_return: Vec<Entity> = Vec::new();
    for drop in to_drop.iter() {
        if rng.roll_dice(1, DROP_ONE_IN_THIS_MANY_TIMES) == 1 {
            equipped.remove(drop.0);
            carried.remove(drop.0);
            positions.insert(drop.0, drop.1.clone()).expect("Unable to insert Position{}.");
        } else {
            to_return.push(drop.0);
        }
    }
    return (to_return, to_spawn);
}
