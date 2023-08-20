use super::{
    triggers::{BLESSED, UNCURSED},
    EffectSpawner, EffectType,
};
use crate::HungerClock;
use specs::prelude::*;

pub fn restore_food(ecs: &mut World, effect: &EffectSpawner, target: Entity) {
    let buc = if let EffectType::RestoreNutrition { buc } = effect.effect_type { buc } else { UNCURSED };
    if let Some(hc) = ecs.write_storage::<HungerClock>().get_mut(target) {
        if buc == BLESSED || buc == UNCURSED {
            hc.duration += 400;
        } else {
            hc.duration += 200;
        }
    }
}
