use specs::prelude::*;
use super::{ EffectSpawner, EffectType };
use crate::components::Attributes;

pub(crate) fn exercise(ecs: &mut World, effect: &EffectSpawner, target: Entity) {
    // Unwrap vars from the effect
    let (attr, inc) = if let EffectType::Exercise { attribute, increment } = effect.effect_type {
        (attribute, increment)
    } else {
        return;
    };

    // Get target attributes
    let mut attributes = ecs.write_storage::<Attributes>();
    if let Some(has_attr) = attributes.get_mut(target) {
        has_attr.exercise(attr, inc);
    }
}
