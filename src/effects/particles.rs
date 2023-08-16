use super::{EffectSpawner, EffectType};
use crate::{Map, ParticleBuilder};
use specs::prelude::*;

pub fn particle_to_tile(ecs: &mut World, target: i32, effect: &EffectSpawner) {
    if let EffectType::Particle { glyph, fg, bg, lifespan, delay } = effect.effect_type {
        let map = ecs.fetch::<Map>();
        let mut particle_builder = ecs.fetch_mut::<ParticleBuilder>();
        if delay <= 0.0 {
            particle_builder.request(target % map.width, target / map.width, fg, bg, glyph, lifespan);
        } else {
            particle_builder.delay(target % map.width, target / map.width, fg, bg, glyph, lifespan, delay);
        }
    }
}
