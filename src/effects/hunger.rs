use super::EffectSpawner;
use crate::{HungerClock, HungerState};
use specs::prelude::*;

const SATIATED_DURATION: i32 = 200;

pub fn restore_food(ecs: &mut World, _damage: &EffectSpawner, target: Entity) {
    if let Some(hc) = ecs.write_storage::<HungerClock>().get_mut(target) {
        hc.state = HungerState::Satiated;
        hc.duration = SATIATED_DURATION;
    }
}
