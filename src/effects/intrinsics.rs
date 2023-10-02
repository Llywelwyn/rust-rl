use super::{ EffectSpawner, EffectType };
use specs::prelude::*;

pub fn add_intrinsic(ecs: &mut World, effect: &EffectSpawner, target: Entity) {
    let intrinsic = if let EffectType::AddIntrinsic { intrinsic } = &effect.effect_type {
        intrinsic
    } else {
        unreachable!("add_intrinsic() called with the wrong EffectType")
    };
    add_intr!(ecs, target, *intrinsic);
}
