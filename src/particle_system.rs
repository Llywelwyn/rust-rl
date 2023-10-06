use super::{ ParticleLifetime, Position, Renderable, BTerm };
use bracket_lib::prelude::*;
use notan::prelude::*;
use specs::prelude::*;
use crate::consts::visuals::{ DEFAULT_PARTICLE_LIFETIME, SHORT_PARTICLE_LIFETIME };

/// Runs each tick, deleting particles who are past their expiry.
// Should make an addition to this to also spawn delayed particles,
// running through a list and removing the frame_time_ms from the
// delay. When delay is <= 0, make a particle_builder.request for
// the particle.
pub fn particle_ticker(ecs: &mut World, ctx: &App) {
    cull_dead_particles(ecs, ctx);
    create_delayed_particles(ecs, ctx);
}

fn cull_dead_particles(ecs: &mut World, ctx: &App) {
    let mut dead_particles: Vec<Entity> = Vec::new();
    {
        // Age out particles
        let mut particles = ecs.write_storage::<ParticleLifetime>();
        let entities = ecs.entities();
        for (entity, mut particle) in (&entities, &mut particles).join() {
            particle.lifetime_ms -= ctx.timer.delta_f32() * 1000.0;
            if particle.lifetime_ms < 0.0 {
                dead_particles.push(entity);
            }
        }
    }
    for dead in dead_particles.iter() {
        ecs.delete_entity(*dead).expect("Particle will not die");
    }
}

pub fn check_queue(ecs: &World) -> bool {
    let particle_builder = ecs.read_resource::<ParticleBuilder>();
    if particle_builder.delayed_requests.is_empty() && particle_builder.requests.is_empty() {
        return true;
    }
    return false;
}

fn create_delayed_particles(ecs: &mut World, ctx: &App) {
    let mut particle_builder = ecs.write_resource::<ParticleBuilder>();
    let mut handled_particles: Vec<ParticleRequest> = Vec::new();
    for delayed_particle in particle_builder.delayed_requests.iter_mut() {
        delayed_particle.delay -= ctx.timer.delta_f32() * 1000.0;
        if delayed_particle.delay < 0.0 {
            handled_particles.push(ParticleRequest {
                x: delayed_particle.particle.x,
                y: delayed_particle.particle.y,
                fg: delayed_particle.particle.fg,
                glyph: delayed_particle.particle.glyph,
                sprite: delayed_particle.particle.sprite.clone(),
                lifetime: delayed_particle.particle.lifetime,
            });
        }
    }
    if handled_particles.is_empty() {
        return;
    }
    // This is repeated code from the ticking system. It could probably be put into a function of its own,
    // but that'd mean having to work around borrow-checker issues, which I'm not convinced is actually
    // cleaner than just repeating the code twice - here and in the ParticleSpawnSystem.
    //
    // We're running separately from the system so we can have it called every single tick, without having
    // to do the same for every system. Later, this will probably all be refactored so every system can run
    // at a higher tickrate.
    let entities = ecs.entities();
    let mut positions = ecs.write_storage::<Position>();
    let mut renderables = ecs.write_storage::<Renderable>();
    let mut particles = ecs.write_storage::<ParticleLifetime>();
    for handled in handled_particles {
        let index = particle_builder.delayed_requests
            .iter()
            .position(|x| x.particle == handled)
            .unwrap();
        particle_builder.delayed_requests.remove(index);
        let p = entities.create();
        positions
            .insert(p, Position { x: handled.x, y: handled.y })
            .expect("Could not insert position");
        renderables
            .insert(p, Renderable::new(handled.glyph, handled.sprite, handled.fg, 0))
            .expect("Could not insert renderables");
        particles
            .insert(p, ParticleLifetime { lifetime_ms: handled.lifetime })
            .expect("Could not insert lifetime");
    }
}

#[derive(PartialEq)]
pub struct ParticleRequest {
    x: i32,
    y: i32,
    fg: RGB,
    glyph: FontCharType,
    sprite: String,
    lifetime: f32,
}

#[derive(PartialEq)]
pub struct DelayedParticleRequest {
    pub delay: f32,
    pub particle: ParticleRequest,
}

pub struct ParticleBuilder {
    requests: Vec<ParticleRequest>,
    delayed_requests: Vec<DelayedParticleRequest>,
}

impl ParticleBuilder {
    #[allow(clippy::new_without_default)]
    pub fn new() -> ParticleBuilder {
        ParticleBuilder { requests: Vec::new(), delayed_requests: Vec::new() }
    }

    /// Makes a single particle request.
    pub fn request(
        &mut self,
        x: i32,
        y: i32,
        fg: RGB,
        glyph: FontCharType,
        sprite: String,
        lifetime: f32
    ) {
        self.requests.push(ParticleRequest { x, y, fg, glyph, sprite, lifetime });
    }

    pub fn delay(
        &mut self,
        x: i32,
        y: i32,
        fg: RGB,
        glyph: FontCharType,
        sprite: String,
        lifetime: f32,
        delay: f32
    ) {
        self.delayed_requests.push(DelayedParticleRequest {
            delay: delay,
            particle: ParticleRequest { x, y, fg, glyph, sprite, lifetime },
        });
    }

    // MASSIVE TODO: Animate these, or make them random. PLACEHOLDER.
    pub fn damage_taken(&mut self, x: i32, y: i32) {
        self.request(x, y, RGB::named(RED), to_cp437('‼'), "slash1".to_string(), 75.0);
        self.delay(x, y, RGB::named(RED), to_cp437('‼'), "slash2".to_string(), 75.0, 75.0);
        self.delay(x, y, RGB::named(RED), to_cp437('‼'), "slash3".to_string(), 75.0, 150.0);
    }
    pub fn attack_miss(&mut self, x: i32, y: i32) {
        self.request(
            x,
            y,
            RGB::named(CYAN),
            to_cp437('‼'),
            "slash1".to_string(),
            DEFAULT_PARTICLE_LIFETIME
        );
    }
    pub fn kick(&mut self, x: i32, y: i32) {
        self.request(
            x,
            y,
            RGB::named(CHOCOLATE),
            to_cp437('‼'),
            "kick".to_string(),
            SHORT_PARTICLE_LIFETIME
        );
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
            positions
                .insert(p, Position { x: new_particle.x, y: new_particle.y })
                .expect("Could not insert position");
            renderables
                .insert(
                    p,
                    Renderable::new(
                        new_particle.glyph,
                        new_particle.sprite.clone(),
                        new_particle.fg,
                        0
                    )
                )
                .expect("Could not insert renderables");
            particles
                .insert(p, ParticleLifetime { lifetime_ms: new_particle.lifetime })
                .expect("Could not insert lifetime");
        }
        particle_builder.requests.clear();
    }
}
