use super::{add_effect, targeting, EffectSpawner, EffectType, Entity, Targets, World};
use crate::{
    gamelog,
    gamesystem::{hp_per_level, mana_per_level},
    Attributes, Confusion, GrantsXP, Map, Player, Pools, DEFAULT_PARTICLE_LIFETIME, LONG_PARTICLE_LIFETIME,
};
use rltk::prelude::*;
use specs::prelude::*;

pub fn inflict_damage(ecs: &mut World, damage: &EffectSpawner, target: Entity) {
    let mut pools = ecs.write_storage::<Pools>();
    if let Some(target_pool) = pools.get_mut(target) {
        if !target_pool.god {
            if let EffectType::Damage { amount } = damage.effect_type {
                target_pool.hit_points.current -= amount;
                add_effect(None, EffectType::Bloodstain, Targets::Entity { target });
                add_effect(
                    None,
                    EffectType::Particle {
                        glyph: to_cp437('‼'),
                        fg: RGB::named(ORANGE),
                        bg: RGB::named(BLACK),
                        lifespan: DEFAULT_PARTICLE_LIFETIME,
                        delay: 0.0,
                    },
                    Targets::Entity { target },
                );
                if target_pool.hit_points.current < 1 {
                    add_effect(damage.source, EffectType::EntityDeath, Targets::Entity { target });
                }
            }
        }
    }
}

pub fn heal_damage(ecs: &mut World, heal: &EffectSpawner, target: Entity) {
    let mut pools = ecs.write_storage::<Pools>();
    if let Some(pool) = pools.get_mut(target) {
        if let EffectType::Healing { amount } = heal.effect_type {
            pool.hit_points.current = i32::min(pool.hit_points.max, pool.hit_points.current + amount);
            add_effect(
                None,
                EffectType::Particle {
                    glyph: to_cp437('♥'),
                    fg: RGB::named(BLUE),
                    bg: RGB::named(BLACK),
                    lifespan: DEFAULT_PARTICLE_LIFETIME,
                    delay: 0.0,
                },
                Targets::Entity { target },
            );
        }
    }
}

pub fn add_confusion(ecs: &mut World, effect: &EffectSpawner, target: Entity) {
    if let EffectType::Confusion { turns } = &effect.effect_type {
        ecs.write_storage::<Confusion>()
            .insert(target, Confusion { turns: *turns })
            .expect("Unable to insert Confusion");
    }
}

pub fn bloodstain(ecs: &mut World, target: usize) {
    let mut map = ecs.fetch_mut::<Map>();
    // If the current tile isn't bloody, bloody it.
    if !map.bloodstains.contains(&target) {
        map.bloodstains.insert(target);
        return;
    }
    let mut spread: i32 = target as i32;
    let mut attempts: i32 = 0;
    // Otherwise, roll to move one tile in any direction.
    // If this tile isn't bloody, bloody it. If not, loop.
    loop {
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();
        attempts += 1;
        spread = match rng.roll_dice(1, 8) {
            1 => spread + 1,
            2 => spread - 1,
            3 => spread + 1 + map.width,
            4 => spread - 1 + map.width,
            5 => spread + 1 - map.width,
            6 => spread - 1 - map.width,
            7 => spread + map.width,
            _ => spread - map.width,
        };
        // - If we're in bounds and the tile is unbloodied, bloody it and return.
        // - If we ever leave bounds, return.
        // - Roll a dice on each failed attempt, with an increasing change to return (soft-capping max spread)
        if spread > 0 && spread < (map.height * map.width) {
            if !map.bloodstains.contains(&(spread as usize)) {
                map.bloodstains.insert(spread as usize);
                return;
            }
            if rng.roll_dice(1, 10 - attempts) == 1 {
                return;
            }
        } else {
            return;
        }
    }
}

pub fn entity_death(ecs: &mut World, effect: &EffectSpawner, target: Entity) {
    let mut xp_gain = 0;
    let mut pools = ecs.write_storage::<Pools>();
    let attributes = ecs.read_storage::<Attributes>();
    // If the target has a position, remove it from the SpatialMap.
    if let Some(pos) = targeting::entity_position(ecs, target) {
        crate::spatial::remove_entity(target, pos as usize);
    }
    // If the target was killed by a source, cont.
    if let Some(source) = effect.source {
        // Calc XP value of target.
        if let Some(xp_value) = ecs.read_storage::<GrantsXP>().get(target) {
            xp_gain += xp_value.amount;
        }
        // If there was XP, run through XP-gain and level-up.
        if xp_gain != 0 {
            let mut source_pools = pools.get_mut(source).unwrap();
            let source_attributes = attributes.get(source).unwrap();
            source_pools.xp += xp_gain;
            let mut next_level_requirement = -1;
            if source_pools.level == 0 {
                next_level_requirement = 5
            } else if source_pools.level < 10 {
                next_level_requirement = 20 * 2_i32.pow(source_pools.level as u32 - 1);
            } else if source_pools.level < 20 {
                next_level_requirement = 10000 * 2_i32.pow(source_pools.level as u32 - 10);
            } else if source_pools.level < 30 {
                next_level_requirement = 10000000 * (source_pools.level - 19);
            }
            if next_level_requirement != -1 && source_pools.xp >= next_level_requirement {
                source_pools.level += 1;
                // If it was the PLAYER that levelled up:
                if ecs.read_storage::<Player>().get(source).is_some() {
                    gamelog::record_event("player_level", 1);
                    gamelog::Logger::new()
                        .append("Welcome to experience level")
                        .append(source_pools.level)
                        .append(".")
                        .log();
                    let player_pos = ecs.fetch::<Point>();
                    let map = ecs.fetch_mut::<Map>();
                    for i in 0..5 {
                        if player_pos.y - i > 1 {
                            add_effect(
                                None,
                                EffectType::Particle {
                                    glyph: to_cp437('░'),
                                    fg: RGB::named(GOLD),
                                    bg: RGB::named(BLACK),
                                    lifespan: LONG_PARTICLE_LIFETIME,
                                    delay: i as f32 * 100.0,
                                },
                                Targets::Tile { target: map.xy_idx(player_pos.x, player_pos.y - i) },
                            );
                            if i > 2 {
                                add_effect(
                                    None,
                                    EffectType::Particle {
                                        glyph: to_cp437('░'),
                                        fg: RGB::named(GOLD),
                                        bg: RGB::named(BLACK),
                                        lifespan: LONG_PARTICLE_LIFETIME,
                                        delay: i as f32 * 100.0,
                                    },
                                    Targets::Tile { target: map.xy_idx(player_pos.x + (i - 2), player_pos.y - i) },
                                );
                                add_effect(
                                    None,
                                    EffectType::Particle {
                                        glyph: to_cp437('░'),
                                        fg: RGB::named(GOLD),
                                        bg: RGB::named(BLACK),
                                        lifespan: LONG_PARTICLE_LIFETIME,
                                        delay: i as f32 * 100.0,
                                    },
                                    Targets::Tile { target: map.xy_idx(player_pos.x - (i - 2), player_pos.y - i) },
                                );
                            }
                        }
                    }
                } else {
                    console::log("DEBUGINFO: Something other than the player levelled up.");
                    // TODO: Growing up, NPC-specific level-up cases.
                }
                let mut rng = ecs.write_resource::<RandomNumberGenerator>();
                let hp_gained = hp_per_level(
                    &mut rng,
                    source_attributes.constitution.base + source_attributes.constitution.modifiers,
                );
                let mana_gained = mana_per_level(
                    &mut rng,
                    source_attributes.intelligence.base + source_attributes.intelligence.modifiers,
                );
                source_pools.hit_points.max += hp_gained;
                source_pools.hit_points.current += hp_gained;
                // Roll for MANA gain this level
                source_pools.mana.max += mana_gained;
                source_pools.mana.current += mana_gained;
            }
        }
    }
}
