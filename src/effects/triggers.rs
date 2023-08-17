use super::{add_effect, spatial, EffectType, Entity, Targets, World};
use crate::{
    gamelog, gui::item_colour_ecs, gui::obfuscate_name_ecs, Confusion, Consumable, Cursed, InflictsDamage, MagicMapper,
    Prop, ProvidesHealing, ProvidesNutrition, RandomNumberGenerator, Renderable, RunState,
};
use rltk::prelude::*;
use specs::prelude::*;

pub fn item_trigger(source: Option<Entity>, item: Entity, target: &Targets, ecs: &mut World) {
    // Use the item via the generic system
    event_trigger(source, item, target, ecs);
    // If it's a consumable, delete it
    if ecs.read_storage::<Consumable>().get(item).is_some() {
        ecs.entities().delete(item).expect("Failed to delete item");
    }
}

pub const BLESSED: i32 = 2;
pub const UNCURSED: i32 = 1;
pub const CURSED: i32 = 0;

struct EventInfo {
    source: Option<Entity>,
    entity: Entity,
    target: Targets,
    buc: i32,
    log: bool,
}

// TODO: Currently, items can only be used by the player, and so this system is only built for that.
//       It does almost no sanity-checking to make sure the logs only appear if the effect is taking
//       place on the player -- once monsters can use an item, their item usage will make logs for
//       the player saying they were the one who used the item. This will need refactoring then.
fn event_trigger(source: Option<Entity>, entity: Entity, target: &Targets, ecs: &mut World) {
    let buc = if ecs.read_storage::<Cursed>().get(entity).is_some() { CURSED } else { UNCURSED };
    let mut event = EventInfo { source, entity, target: target.clone(), buc, log: false };
    let mut logger = gamelog::Logger::new();
    // PROVIDES NUTRITION
    logger = handle_restore_nutrition(ecs, &mut event, logger);
    // MAGIC MAPPER
    logger = handle_magic_mapper(ecs, &mut event, logger);
    // DOES HEALING
    logger = handle_healing(ecs, &mut event, logger);
    // DOES DAMAGE
    logger = handle_damage(ecs, &mut event, logger);
    // APPLIES CONFUSION
    logger = handle_confusion(ecs, &mut event, logger);
    if event.log {
        logger.log();
    }
}

fn handle_restore_nutrition(ecs: &mut World, event: &mut EventInfo, mut logger: gamelog::Logger) -> gamelog::Logger {
    if ecs.read_storage::<ProvidesNutrition>().get(event.entity).is_some() {
        add_effect(event.source, EffectType::RestoreNutrition { buc: event.buc }, event.target.clone());
        logger = logger
            .append("You eat the")
            .colour(item_colour_ecs(ecs, event.entity))
            .append_n(obfuscate_name_ecs(ecs, event.entity).0)
            .colour(WHITE)
            .period()
            .buc(event.buc, Some("Blech! Rotten."), Some("Delicious."));
        event.log = true;
    }
    return logger;
}

fn handle_magic_mapper(ecs: &mut World, event: &mut EventInfo, mut logger: gamelog::Logger) -> gamelog::Logger {
    if ecs.read_storage::<MagicMapper>().get(event.entity).is_some() {
        let mut runstate = ecs.fetch_mut::<RunState>();
        let cursed = if event.buc == CURSED { true } else { false };
        *runstate = RunState::MagicMapReveal { row: 0, cursed: cursed };
        logger = logger.append("You recall your surroundings!").buc(
            event.buc,
            Some("... but forget where you last were."),
            None,
        );
        event.log = true;
    }
    return logger;
}

fn handle_healing(ecs: &mut World, event: &mut EventInfo, mut logger: gamelog::Logger) -> gamelog::Logger {
    if let Some(healing_item) = ecs.read_storage::<ProvidesHealing>().get(event.entity) {
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();
        let roll = rng.roll_dice(healing_item.n_dice, healing_item.sides) + healing_item.modifier;
        add_effect(event.source, EffectType::Healing { amount: roll }, event.target.clone());
        logger = logger.append("You recover some vigour.").buc(event.buc, None, Some("You feel great!"));
        event.log = true;
    }
    return logger;
}

fn handle_damage(ecs: &mut World, event: &mut EventInfo, mut logger: gamelog::Logger) -> gamelog::Logger {
    if let Some(damage_item) = ecs.read_storage::<InflictsDamage>().get(event.entity) {
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();
        let roll = rng.roll_dice(damage_item.n_dice, damage_item.sides) + damage_item.modifier;
        add_effect(event.source, EffectType::Damage { amount: roll }, event.target.clone());
        for target in get_entity_targets(&event.target) {
            if ecs.read_storage::<Prop>().get(target).is_some() {
                continue;
            }
            let fg = if let Some(renderable) = ecs.read_storage::<Renderable>().get(target) {
                ((renderable.fg.r * 255.0) as u8, (renderable.fg.g * 255.0) as u8, (renderable.fg.b * 255.0) as u8)
            } else {
                WHITE
            };
            logger = logger
                .append("The")
                .colour(fg)
                .append(obfuscate_name_ecs(ecs, target).0)
                .colour(WHITE)
                .append("is hit!");
            event.log = true;
        }
    }
    return logger;
}

#[allow(unused_mut)]
fn handle_confusion(ecs: &mut World, event: &mut EventInfo, mut logger: gamelog::Logger) -> gamelog::Logger {
    if let Some(confusion) = ecs.read_storage::<Confusion>().get(event.entity) {
        add_effect(event.source, EffectType::Confusion { turns: confusion.turns }, event.target.clone());
    }
    return logger;
}

fn get_entity_targets(target: &Targets) -> Vec<Entity> {
    let mut entities: Vec<Entity> = Vec::new();
    match target {
        Targets::Entity { target } => entities.push(*target),
        Targets::EntityList { targets } => targets.iter().for_each(|target| entities.push(*target)),
        Targets::Tile { target } => {
            spatial::for_each_tile_content(*target, |entity| {
                entities.push(entity);
            });
        }
        Targets::TileList { targets } => {
            targets.iter().for_each(|target| {
                spatial::for_each_tile_content(*target, |entity| {
                    entities.push(entity);
                });
            });
        }
    }
    return entities;
}
