use super::BUC;
use crate::spatial;
use rltk::prelude::*;
use specs::prelude::*;
use std::collections::VecDeque;
use std::sync::Mutex;

mod damage;
mod hunger;
mod particles;
mod targeting;
mod triggers;

pub use targeting::aoe_tiles;

lazy_static! {
    pub static ref EFFECT_QUEUE: Mutex<VecDeque<EffectSpawner>> = Mutex::new(VecDeque::new());
}

lazy_static! {
    pub static ref DEAD_ENTITIES: Mutex<VecDeque<Entity>> = Mutex::new(VecDeque::new());
}

pub enum EffectType {
    Damage { amount: i32 },
    Healing { amount: i32 },
    Confusion { turns: i32 },
    Bloodstain,
    Particle { glyph: FontCharType, fg: RGB, bg: RGB, lifespan: f32, delay: f32 },
    EntityDeath,
    ItemUse { item: Entity },
    RestoreNutrition { buc: BUC },
    TriggerFire { trigger: Entity },
}

#[derive(Clone)]
pub enum Targets {
    Entity { target: Entity },
    EntityList { targets: Vec<Entity> },
    Tile { target: usize },
    TileList { targets: Vec<usize> },
}

pub struct EffectSpawner {
    pub source: Option<Entity>,
    pub effect_type: EffectType,
    pub target: Targets,
}

/// Adds an effect to the effects queue
pub fn add_effect(source: Option<Entity>, effect_type: EffectType, target: Targets) {
    let mut lock = EFFECT_QUEUE.lock().unwrap();
    lock.push_back(EffectSpawner { source, effect_type, target });
}

/// Iterates through the effects queue, applying each effect to their target.
pub fn run_effects_queue(ecs: &mut World) {
    // First removes any effect in the EFFECT_QUEUE with a dead entity as its source.
    loop {
        let dead_entity: Option<Entity> = DEAD_ENTITIES.lock().unwrap().pop_front();
        if let Some(dead_entity) = dead_entity {
            EFFECT_QUEUE.lock().unwrap().retain(|x| x.source != Some(dead_entity));
        } else {
            break;
        }
    }
    // Then runs every effect that remains in the queue.
    loop {
        let effect: Option<EffectSpawner> = EFFECT_QUEUE.lock().unwrap().pop_front();
        if let Some(effect) = effect {
            target_applicator(ecs, &effect);
        } else {
            break;
        }
    }
}

/// Applies an effect to the correct target(s).
fn target_applicator(ecs: &mut World, effect: &EffectSpawner) {
    // Item use is handled differently - it creates other effects with itself
    // as the source, passing all effects attached to the item into the queue.
    if let EffectType::ItemUse { item } = effect.effect_type {
        triggers::item_trigger(effect.source, item, &effect.target, ecs);
        return;
    } else if let EffectType::TriggerFire { trigger } = effect.effect_type {
        triggers::trigger(effect.source, trigger, &effect.target, ecs);
        return;
    }
    // Otherwise, just match the effect and enact it directly.
    match &effect.target {
        Targets::Tile { target } => affect_tile(ecs, effect, *target),
        Targets::TileList { targets } => targets.iter().for_each(|target| affect_tile(ecs, effect, *target)),
        Targets::Entity { target } => affect_entity(ecs, effect, *target),
        Targets::EntityList { targets } => targets.iter().for_each(|target| affect_entity(ecs, effect, *target)),
    }
}

/// Runs an effect on a given tile index
fn affect_tile(ecs: &mut World, effect: &EffectSpawner, target: usize) {
    if tile_effect_hits_entities(&effect.effect_type) {
        spatial::for_each_tile_content(target, |entity| {
            affect_entity(ecs, effect, entity);
        });
    }

    match &effect.effect_type {
        EffectType::Bloodstain => damage::bloodstain(ecs, target),
        EffectType::Particle { .. } => particles::particle_to_tile(ecs, target as i32, &effect),
        _ => {}
    }
    // Run the effect
}

/// Checks if a given effect affects entities or not.
fn tile_effect_hits_entities(effect: &EffectType) -> bool {
    match effect {
        EffectType::Damage { .. } => true,
        EffectType::Healing { .. } => true,
        EffectType::RestoreNutrition { .. } => true,
        EffectType::Confusion { .. } => true,
        _ => false,
    }
}

/// Runs an effect on a given entity
fn affect_entity(ecs: &mut World, effect: &EffectSpawner, target: Entity) {
    match &effect.effect_type {
        EffectType::Damage { .. } => damage::inflict_damage(ecs, effect, target),
        EffectType::Healing { .. } => damage::heal_damage(ecs, effect, target),
        EffectType::Confusion { .. } => damage::add_confusion(ecs, effect, target),
        EffectType::Bloodstain { .. } => {
            if let Some(pos) = targeting::entity_position(ecs, target) {
                damage::bloodstain(ecs, pos)
            }
        }
        EffectType::Particle { .. } => {
            if let Some(pos) = targeting::entity_position(ecs, target) {
                particles::particle_to_tile(ecs, pos as i32, &effect)
            }
        }
        EffectType::EntityDeath => damage::entity_death(ecs, effect, target),
        EffectType::RestoreNutrition { .. } => hunger::restore_food(ecs, effect, target),
        _ => {}
    }
}
