use super::{Entity, Targets, World};
use crate::{gamelog, Consumable};
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
    let logger = gamelog::Logger::new();
}
