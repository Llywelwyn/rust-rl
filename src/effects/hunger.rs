use super::{EffectSpawner, EffectType};
use crate::{HungerClock, BUC};
use specs::prelude::*;

pub fn restore_food(ecs: &mut World, effect: &EffectSpawner, target: Entity) {
    if let EffectType::RestoreNutrition { buc } = &effect.effect_type {
        if let Some(hc) = ecs.write_storage::<HungerClock>().get_mut(target) {
            if buc == &BUC::Blessed {
                hc.duration += 600;
            } else if buc == &BUC::Uncursed {
                hc.duration += 400;
            } else if buc == &BUC::Cursed {
                hc.duration += 200;
            }
        }
    }
}
