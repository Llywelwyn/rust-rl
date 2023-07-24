use super::{ParticleLifetime, Position, Renderable, Rltk};
use rltk::RGB;
use specs::prelude::*;

pub const SHORT_PARTICLE_LIFETIME: f32 = 100.0;
// For things which will happen frequently - i.e. attacking.
pub const DEFAULT_PARTICLE_LIFETIME: f32 = 200.0;
// For exceptional things, like large AOEs, to make sure the
// player can actually see what's being impacted - i.e. fireball.
pub const LONG_PARTICLE_LIFETIME: f32 = 300.0;

/// Runs each tick, deleting particles who are past their expiry.
// Should make an addition to this to also spawn delayed particles,
// running through a list and removing the frame_time_ms from the
// delay. When delay is <= 0, make a particle_builder.request for
// the particle.
pub fn cull_dead_particles(ecs: &mut World, ctx: &Rltk) {
    let mut dead_particles: Vec<Entity> = Vec::new();
    {
        // Age out particles
        let mut particles = ecs.write_storage::<ParticleLifetime>();
        let entities = ecs.entities();
        for (entity, mut particle) in (&entities, &mut particles).join() {
            particle.lifetime_ms -= ctx.frame_time_ms;
            if particle.lifetime_ms < 0.0 {
                dead_particles.push(entity);
            }
        }
    }
    for dead in dead_particles.iter() {
        ecs.delete_entity(*dead).expect("Particle will not die");
    }
}

struct ParticleRequest {
    x: i32,
    y: i32,
    fg: RGB,
    bg: RGB,
    glyph: rltk::FontCharType,
    lifetime: f32,
}

pub struct ParticleBuilder {
    requests: Vec<ParticleRequest>,
}

impl ParticleBuilder {
    #[allow(clippy::new_without_default)]
    pub fn new() -> ParticleBuilder {
        ParticleBuilder { requests: Vec::new() }
    }

    /// Makes a single particle request.
    pub fn request(&mut self, x: i32, y: i32, fg: RGB, bg: RGB, glyph: rltk::FontCharType, lifetime: f32) {
        self.requests.push(ParticleRequest { x, y, fg, bg, glyph, lifetime });
    }

    pub fn damage_taken(&mut self, x: i32, y: i32) {
        self.request(
            x,
            y,
            rltk::RGB::named(rltk::ORANGE),
            rltk::RGB::named(rltk::BLACK),
            rltk::to_cp437('‼'),
            DEFAULT_PARTICLE_LIFETIME,
        );
    }

    pub fn trap_triggered(&mut self, x: i32, y: i32) {
        self.request(
            x,
            y,
            rltk::RGB::named(rltk::RED),
            rltk::RGB::named(rltk::RED),
            rltk::to_cp437('‼'),
            DEFAULT_PARTICLE_LIFETIME,
        );
    }

    pub fn heal(&mut self, x: i32, y: i32) {
        self.request(
            x,
            y,
            rltk::RGB::named(rltk::GREEN),
            rltk::RGB::named(rltk::BLACK),
            rltk::to_cp437('♥'),
            DEFAULT_PARTICLE_LIFETIME,
        );
    }

    pub fn kick(&mut self, x: i32, y: i32) {
        self.request(
            x,
            y,
            rltk::RGB::named(rltk::CHOCOLATE),
            rltk::RGB::named(rltk::BLACK),
            rltk::to_cp437('‼'),
            SHORT_PARTICLE_LIFETIME,
        );
    }

    // Makes a particle request in the shape of an 'x'. Sort of.
    pub fn request_star(&mut self, x: i32, y: i32, fg: RGB, bg: RGB, glyph: rltk::FontCharType, lifetime: f32) {
        self.request(x, y, fg, bg, glyph, lifetime * 2.0);
        self.request(x + 1, y + 1, fg, bg, rltk::to_cp437('/'), lifetime);
        self.request(x + 1, y - 1, fg, bg, rltk::to_cp437('\\'), lifetime);
        self.request(x - 1, y + 1, fg, bg, rltk::to_cp437('\\'), lifetime);
        self.request(x - 1, y - 1, fg, bg, rltk::to_cp437('/'), lifetime);
    }

    /// Makes a particle request in the shape of a +.
    #[allow(dead_code)]
    pub fn request_plus(&mut self, x: i32, y: i32, fg: RGB, bg: RGB, glyph: rltk::FontCharType, lifetime: f32) {
        self.request(x, y, fg, bg, glyph, lifetime * 2.0);
        self.request(x + 1, y, fg, bg, rltk::to_cp437('─'), lifetime);
        self.request(x - 1, y, fg, bg, rltk::to_cp437('─'), lifetime);
        self.request(x, y + 1, fg, bg, rltk::to_cp437('│'), lifetime);
        self.request(x, y - 1, fg, bg, rltk::to_cp437('│'), lifetime);
    }
}

pub struct ParticleSpawnSystem {}

impl<'a> System<'a> for ParticleSpawnSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, Renderable>,
        WriteStorage<'a, ParticleLifetime>,
        WriteExpect<'a, ParticleBuilder>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, mut positions, mut renderables, mut particles, mut particle_builder) = data;

        for new_particle in particle_builder.requests.iter() {
            let p = entities.create();
            positions.insert(p, Position { x: new_particle.x, y: new_particle.y }).expect("Could not insert position");
            renderables
                .insert(
                    p,
                    Renderable { fg: new_particle.fg, bg: new_particle.bg, glyph: new_particle.glyph, render_order: 0 },
                )
                .expect("Could not insert renderables");
            particles
                .insert(p, ParticleLifetime { lifetime_ms: new_particle.lifetime })
                .expect("Could not insert lifetime");
        }
        particle_builder.requests.clear();
    }
}
