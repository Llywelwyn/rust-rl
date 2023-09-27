use specs::prelude::*;
use bracket_lib::prelude::*;
use super::{ EffectSpawner, EffectType };
use crate::components::Attributes;

const ATTRIBUTE_SOFTCAP: i32 = 20;
const ABUSE_CHANCE: i32 = 2; // 1 in this chance of abuse. 2 = 50%, 3 = 33%, etc.

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
        // Roll a d20. If we're trying to exercise a stat, we need to roll higher
        // than the stat's current value. If we're abusing a stat, flip a coin.
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();
        let success = if inc {
            rng.roll_dice(1, ATTRIBUTE_SOFTCAP) > has_attr.attr_from_index(attr).current()
        } else {
            rng.roll_dice(1, ABUSE_CHANCE) == 1
        };
        if success {
            has_attr.exercise(attr, inc);
        }
    }
}
