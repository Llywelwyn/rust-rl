use super::{add_effect, EffectType, Entity, Targets, World};
use crate::{gamelog, gui::item_colour_ecs, gui::obfuscate_name, Consumable, ProvidesNutrition};
use specs::prelude::*;

pub fn item_trigger(source: Option<Entity>, item: Entity, target: &Targets, ecs: &mut World) {
    // Use the item via the generic system
    event_trigger(source, item, target, ecs);
    // If it's a consumable, delete it
    if ecs.read_storage::<Consumable>().get(item).is_some() {
        ecs.entities().delete(item).expect("Failed to delete item");
    }
}

fn event_trigger(source: Option<Entity>, entity: Entity, target: &Targets, ecs: &mut World) {
    let mut logger = gamelog::Logger::new();
    // Providing nutrition
    if ecs.read_storage::<ProvidesNutrition>().get(entity).is_some() {
        add_effect(source, EffectType::RestoreNutrition, target.clone());
        logger = logger
            .append("You eat the")
            .append_n(obfuscate_name(ecs, entity).0)
            .colour(item_colour_ecs(ecs, entity))
            .period();
    }
    logger.log();
}
