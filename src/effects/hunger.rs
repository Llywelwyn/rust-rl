use super::{
    triggers::{BLESSED, CURSED, UNCURSED},
    EffectSpawner, EffectType,
};
use crate::{HungerClock, HungerState};
use specs::prelude::*;

const SATIATED_DURATION: i32 = 200;

pub fn restore_food(ecs: &mut World, effect: &EffectSpawner, target: Entity) {
    let buc = if let EffectType::RestoreNutrition { buc } = effect.effect_type { buc } else { UNCURSED };
    if let Some(hc) = ecs.write_storage::<HungerClock>().get_mut(target) {
        if buc == BLESSED || buc == UNCURSED {
            hc.state = HungerState::Satiated;
            hc.duration = SATIATED_DURATION;
        } else {
            hc.duration = 0;
        }
    }
}
