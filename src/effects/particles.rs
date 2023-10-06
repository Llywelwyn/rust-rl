use super::{ add_effect, targeting, EffectSpawner, EffectType, Targets };
use crate::{ Map, ParticleBuilder, SpawnParticleBurst, SpawnParticleLine, SpawnParticleSimple };
use bracket_lib::prelude::*;
use specs::prelude::*;

pub fn particle_to_tile(ecs: &mut World, target: i32, effect: &EffectSpawner) {
    if let EffectType::Particle { glyph, sprite, fg, lifespan, delay } = &effect.effect_type {
        let map = ecs.fetch::<Map>();
        let mut particle_builder = ecs.fetch_mut::<ParticleBuilder>();
        if delay <= &0.0 {
            particle_builder.request(
                target % map.width,
                target / map.width,
                *fg,
                *glyph,
                sprite.clone(),
                *lifespan
            );
        } else {
            particle_builder.delay(
                target % map.width,
                target / map.width,
                *fg,
                *glyph,
                sprite.clone(),
                *lifespan,
                *delay
            );
        }
    }
}

pub fn handle_simple_particles(ecs: &World, entity: Entity, target: &Targets) {
    if let Some(part) = ecs.read_storage::<SpawnParticleSimple>().get(entity) {
        add_effect(
            None,
            EffectType::Particle {
                glyph: part.glyph,
                sprite: part.sprite.clone(),
                fg: part.colour,
                lifespan: part.lifetime_ms,
                delay: 0.0,
            },
            target.clone()
        );
    }
}

pub fn handle_burst_particles(ecs: &World, entity: Entity, target: &Targets) {
    if let Some(part) = ecs.read_storage::<SpawnParticleBurst>().get(entity) {
        if let Some(start_pos) = targeting::find_item_position(ecs, entity) {
            let end_pos: i32 = get_centre(ecs, target);
            spawn_line_particles(
                ecs,
                start_pos,
                end_pos,
                &(SpawnParticleLine {
                    glyph: part.head_glyph,
                    sprite: part.head_sprite.clone(),
                    tail_glyph: part.tail_glyph,
                    tail_sprite: part.tail_sprite.clone(),
                    colour: part.colour,
                    trail_colour: part.trail_colour,
                    lifetime_ms: part.trail_lifetime_ms, // 75.0 is good here.
                    trail_lifetime_ms: part.trail_lifetime_ms,
                })
            );
            let map = ecs.fetch::<Map>();
            let line = line2d(
                LineAlg::Bresenham,
                Point::new(start_pos % map.width, start_pos / map.width),
                Point::new(end_pos % map.width, end_pos / map.width)
            );
            let burst_delay = (line.len() as f32) * part.trail_lifetime_ms;
            for i in 0..10 {
                add_effect(
                    None,
                    EffectType::Particle {
                        glyph: part.glyph,
                        sprite: part.sprite.clone(),
                        fg: part.colour.lerp(part.lerp, (i as f32) * 0.1),
                        lifespan: part.lifetime_ms / 10.0, // ~50-80 is good here.
                        delay: burst_delay + ((i as f32) * part.lifetime_ms) / 10.0, // above + burst_delay
                    },
                    target.clone()
                );
            }
        }
    }
}

fn get_centre(ecs: &World, target: &Targets) -> i32 {
    match target {
        Targets::Tile { target } => {
            return *target as i32;
        }
        Targets::TileList { targets } => {
            let map = ecs.fetch::<Map>();
            let (mut count, mut sum_x, mut sum_y) = (0, 0, 0);
            for target in targets {
                sum_x += (*target as i32) % map.width;
                sum_y += (*target as i32) / map.width;
                count += 1;
            }
            let (mean_x, mean_y) = (sum_x / count, sum_y / count);
            let centre = map.xy_idx(mean_x, mean_y);
            return centre as i32;
        }
        Targets::Entity { target } => {
            return targeting::entity_position(ecs, *target).unwrap() as i32;
        }
        Targets::EntityList { targets } => {
            let map = ecs.fetch::<Map>();
            let (mut count, mut sum_x, mut sum_y) = (0, 0, 0);
            for target in targets {
                if let Some(pos) = targeting::entity_position(ecs, *target) {
                    sum_x += (pos as i32) % map.width;
                    sum_y += (pos as i32) / map.width;
                    count += 1;
                }
            }
            let (mean_x, mean_y) = (sum_x / count, sum_y / count);
            let centre = map.xy_idx(mean_x, mean_y);
            return centre as i32;
        }
    }
}

pub fn handle_line_particles(ecs: &World, entity: Entity, target: &Targets) {
    if let Some(part) = ecs.read_storage::<SpawnParticleLine>().get(entity) {
        if let Some(start_pos) = targeting::find_item_position(ecs, entity) {
            match target {
                Targets::Tile { target } =>
                    spawn_line_particles(ecs, start_pos, *target as i32, part),
                Targets::TileList { targets } => {
                    targets
                        .iter()
                        .for_each(|target|
                            spawn_line_particles(ecs, start_pos, *target as i32, part)
                        )
                }
                Targets::Entity { target } => {
                    if let Some(end_pos) = targeting::entity_position(ecs, *target) {
                        spawn_line_particles(ecs, start_pos, end_pos as i32, part);
                    }
                }
                Targets::EntityList { targets } =>
                    targets.iter().for_each(|target| {
                        if let Some(end_pos) = targeting::entity_position(ecs, *target) {
                            spawn_line_particles(ecs, start_pos, end_pos as i32, part);
                        }
                    }),
            }
        }
    }
}

fn spawn_line_particles(ecs: &World, start: i32, end: i32, part: &SpawnParticleLine) {
    let map = ecs.fetch::<Map>();
    let start_pt = Point::new(start % map.width, start / map.width);
    let end_pt = Point::new(end % map.width, end / map.width);
    let line = line2d(LineAlg::Bresenham, start_pt, end_pt);
    for (i, pt) in line.iter().enumerate() {
        add_effect(
            None,
            EffectType::Particle {
                glyph: part.glyph,
                sprite: part.sprite.clone(),
                fg: part.colour,
                lifespan: part.lifetime_ms,
                delay: (i as f32) * part.lifetime_ms,
            },
            Targets::Tile { target: map.xy_idx(pt.x, pt.y) }
        );
        if i > 0 {
            add_effect(
                None,
                EffectType::Particle {
                    glyph: part.tail_glyph,
                    sprite: part.tail_sprite.clone(),
                    fg: part.trail_colour,
                    lifespan: part.trail_lifetime_ms,
                    delay: (i as f32) * part.lifetime_ms,
                },
                Targets::Tile { target: map.xy_idx(line[i - 1].x, line[i - 1].y) }
            );
        }
    }
}
