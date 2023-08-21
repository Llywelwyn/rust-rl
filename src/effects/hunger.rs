use super::{EffectSpawner, EffectType};
use crate::{HungerClock, BUC};
use specs::prelude::*;

pub fn modify_nutrition(ecs: &mut World, effect: &EffectSpawner, target: Entity) {
    if let EffectType::ModifyNutrition { amount } = &effect.effect_type {
        if let Some(hc) = ecs.write_storage::<HungerClock>().get_mut(target) {
            hc.duration += amount;
        }
    }
}
