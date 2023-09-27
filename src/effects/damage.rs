use super::{ add_effect, targeting, EffectSpawner, EffectType, Entity, Targets, World };
use crate::{
    gamelog,
    gamesystem::{ hp_per_level, mana_per_level },
    Attributes,
    Confusion,
    Destructible,
    GrantsXP,
    Map,
    Player,
    Pools,
    Name,
    Blind,
    HungerClock,
    HungerState,
    Bleeds,
    HasDamageModifiers,
};
use crate::gui::with_article;
use crate::consts::visuals::{ DEFAULT_PARTICLE_LIFETIME, LONG_PARTICLE_LIFETIME };
use crate::consts::messages::LEVELUP_PLAYER;
use crate::consts::events::*;
use crate::consts::messages::*;
use bracket_lib::prelude::*;
use specs::prelude::*;

pub fn inflict_damage(ecs: &mut World, damage: &EffectSpawner, target: Entity) {
    let mut pools = ecs.write_storage::<Pools>();
    if let Some(target_pool) = pools.get_mut(target) {
        if !target_pool.god {
            if let EffectType::Damage { amount, damage_type } = damage.effect_type {
                let mult = if
                    let Some(modifiers) = ecs.read_storage::<HasDamageModifiers>().get(target)
                {
                    modifiers.modifier(&damage_type).multiplier()
                } else {
                    1.0
                };
                target_pool.hit_points.current -= ((amount as f32) * mult) as i32;
                let bleeders = ecs.read_storage::<Bleeds>();
                if let Some(bleeds) = bleeders.get(target) {
                    add_effect(
                        None,
                        EffectType::Bloodstain { colour: bleeds.colour },
                        Targets::Entity { target }
                    );
                }
                add_effect(
                    None,
                    EffectType::Particle {
                        glyph: to_cp437('‼'),
                        fg: RGB::named(ORANGE),
                        bg: RGB::named(BLACK),
                        lifespan: DEFAULT_PARTICLE_LIFETIME,
                        delay: 0.0,
                    },
                    Targets::Entity { target }
                );
                if target_pool.hit_points.current < 1 {
                    super::DEAD_ENTITIES.lock().unwrap().push_back(target);
                    add_effect(damage.source, EffectType::EntityDeath, Targets::Entity { target });
                }
            }
        }
    } else if let Some(_destructible) = ecs.read_storage::<Destructible>().get(target) {
        add_effect(damage.source, EffectType::EntityDeath, Targets::Entity { target });
    }
}

pub fn heal_damage(ecs: &mut World, heal: &EffectSpawner, target: Entity) {
    let mut pools = ecs.write_storage::<Pools>();
    if let Some(pool) = pools.get_mut(target) {
        if let EffectType::Healing { amount, increment_max } = &heal.effect_type {
            let before = pool.hit_points.current;
            pool.hit_points.current = i32::min(
                pool.hit_points.max,
                pool.hit_points.current + amount
            );
            if pool.hit_points.current - before < *amount && *increment_max {
                // If the heal was not fully effective, and healing source was noncursed, increase max HP by 1.
                pool.hit_points.max += 1;
                pool.hit_points.current += 1;
            }
            add_effect(
                None,
                EffectType::Particle {
                    glyph: to_cp437('♥'),
                    fg: RGB::named(BLUE),
                    bg: RGB::named(BLACK),
                    lifespan: DEFAULT_PARTICLE_LIFETIME,
                    delay: 0.0,
                },
                Targets::Entity { target }
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

pub fn bloodstain(ecs: &mut World, target: usize, colour: RGB) {
    let mut map = ecs.fetch_mut::<Map>();
    // If the current tile isn't bloody, bloody it.
    if !map.bloodstains.contains_key(&target) {
        map.bloodstains.insert(target, colour);
        return;
    }
    if map.bloodstains.get(&target).unwrap() == &colour {
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
            if spread > 0 && spread < map.height * map.width {
                if !map.bloodstains.contains_key(&(spread as usize)) {
                    map.bloodstains.insert(spread as usize, colour);
                    return;
                }
                // If bloodied with the same colour, return
                if map.bloodstains.get(&(spread as usize)).unwrap() == &colour {
                    if rng.roll_dice(1, 10 - attempts) == 1 {
                        return;
                    }
                    // If bloodied but a *different* colour, lerp this blood and current blood.
                } else {
                    let new_col = map.bloodstains
                        .get(&(spread as usize))
                        .unwrap()
                        .lerp(colour, 0.5);
                    map.bloodstains.insert(spread as usize, new_col);
                }
            } else {
                return;
            }
        }
    } else {
        let curr_blood = map.bloodstains.get(&target).unwrap();
        let new_colour = curr_blood.lerp(colour, 0.5);
        map.bloodstains.insert(target, new_colour);
        return;
    }
}

/// Takes a level, and returns the total XP required to reach the next level.
fn get_next_level_requirement(level: i32) -> i32 {
    if level == 0 {
        return 5;
    } else if level < 10 {
        return 20 * (2_i32).pow((level as u32) - 1);
    } else if level < 20 {
        return 10000 * (2_i32).pow((level as u32) - 10);
    } else if level < 30 {
        return 10000000 * (level - 19);
    }
    return -1;
}

fn get_death_message(ecs: &World, source: Entity) -> String {
    let player = ecs.fetch::<Entity>();
    let mut result: String = format!("{} ", PLAYER_DIED);
    // If we killed ourselves,
    if source == *player {
        result.push_str(format!("{}", PLAYER_DIED_SUICIDE).as_str());
    } else if let Some(name) = ecs.read_storage::<Name>().get(source) {
        result.push_str(
            format!("{} {}", PLAYER_DIED_NAMED_ATTACKER, with_article(&name.name)).as_str()
        );
    } else {
        result.push_str(format!("{}", PLAYER_DIED_UNKNOWN).as_str());
    }
    // Status effects
    {
        let mut addendums: Vec<&str> = Vec::new();
        if let Some(_confused) = ecs.read_storage::<Confusion>().get(*player) {
            addendums.push(STATUS_CONFUSED_STRING);
        }
        if let Some(_blind) = ecs.read_storage::<Blind>().get(*player) {
            addendums.push(STATUS_BLIND_STRING);
        }
        if !addendums.is_empty() {
            result.push_str(" whilst");
            for (i, addendum) in addendums.iter().enumerate() {
                if i == 0 {
                    result.push_str(format!("{}{}", PLAYER_DIED_ADDENDUM_FIRST, addendum).as_str());
                } else if i == addendums.len() {
                    result.push_str(format!("{}{}", PLAYER_DIED_ADDENDUM_LAST, addendum).as_str());
                } else {
                    result.push_str(format!("{}{}", PLAYER_DIED_ADDENDUM_MID, addendum).as_str());
                }
            }
        }
    }
    return result;
}

/// Handles EntityDeath effects.
pub fn entity_death(ecs: &mut World, effect: &EffectSpawner, target: Entity) {
    let mut xp_gain = 0;
    let mut pools = ecs.write_storage::<Pools>();
    let attributes = ecs.read_storage::<Attributes>();
    let names = ecs.read_storage::<Name>();
    let player = ecs.fetch::<Entity>();
    // If the target has a position, remove it from the SpatialMap.
    if let Some(pos) = targeting::entity_position(ecs, target) {
        crate::spatial::remove_entity(target, pos as usize);
    }
    // If the target was killed by a source, cont.
    if let Some(source) = effect.source {
        // If the target was the player, game over, and record source of death.
        if target == *player {
            gamelog::record_event(EVENT::PlayerDied(get_death_message(ecs, source)));
            return;
        } else {
            // If the player was the source, record the kill.
            if let Some(tar_name) = names.get(target) {
                gamelog::record_event(EVENT::Killed(tar_name.name.clone()));
            }
        }
        // Calc XP value of target.
        if let Some(xp_value) = ecs.read_storage::<GrantsXP>().get(target) {
            xp_gain += xp_value.amount;
        }
        // If there was XP, run through XP-gain and level-up.
        if xp_gain != 0 {
            if let None = pools.get(source) {
                return;
            }
            let mut source_pools = pools.get_mut(source).unwrap();
            let source_attributes = attributes.get(source).unwrap();
            source_pools.xp += xp_gain;
            let next_level_requirement = get_next_level_requirement(source_pools.level);
            if next_level_requirement != -1 && source_pools.xp >= next_level_requirement {
                source_pools.level += 1;
                // If it was the PLAYER that levelled up:
                if ecs.read_storage::<Player>().get(source).is_some() {
                    gamelog::record_event(EVENT::Level(1));
                    gamelog::Logger
                        ::new()
                        .append(LEVELUP_PLAYER)
                        .append_n(source_pools.level)
                        .append("!")
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
                                    delay: (i as f32) * 100.0,
                                },
                                Targets::Tile { target: map.xy_idx(player_pos.x, player_pos.y - i) }
                            );
                            if i > 2 {
                                add_effect(
                                    None,
                                    EffectType::Particle {
                                        glyph: to_cp437('░'),
                                        fg: RGB::named(GOLD),
                                        bg: RGB::named(BLACK),
                                        lifespan: LONG_PARTICLE_LIFETIME,
                                        delay: (i as f32) * 100.0,
                                    },
                                    Targets::Tile {
                                        target: map.xy_idx(
                                            player_pos.x + (i - 2),
                                            player_pos.y - i
                                        ),
                                    }
                                );
                                add_effect(
                                    None,
                                    EffectType::Particle {
                                        glyph: to_cp437('░'),
                                        fg: RGB::named(GOLD),
                                        bg: RGB::named(BLACK),
                                        lifespan: LONG_PARTICLE_LIFETIME,
                                        delay: (i as f32) * 100.0,
                                    },
                                    Targets::Tile {
                                        target: map.xy_idx(
                                            player_pos.x - (i - 2),
                                            player_pos.y - i
                                        ),
                                    }
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
                    source_attributes.constitution.base + source_attributes.constitution.bonuses
                );
                let mana_gained = mana_per_level(
                    &mut rng,
                    source_attributes.intelligence.base + source_attributes.intelligence.bonuses
                );
                source_pools.hit_points.max += hp_gained;
                source_pools.hit_points.current += hp_gained;
                // Roll for MANA gain this level
                source_pools.mana.max += mana_gained;
                source_pools.mana.current += mana_gained;
            }
        }
    } else {
        if target == *player {
            if let Some(hc) = ecs.read_storage::<HungerClock>().get(target) {
                if hc.state == HungerState::Starving {
                    gamelog::record_event(EVENT::PlayerDied("You starved to death!".to_string()));
                }
            } else {
                gamelog::record_event(
                    EVENT::PlayerDied("You died from unknown causes!".to_string())
                );
            }
        }
    }
}
