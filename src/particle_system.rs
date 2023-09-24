use super::{ ParticleLifetime, Position, Renderable, BTerm };
use bracket_lib::prelude::*;
use specs::prelude::*;
use crate::consts::visuals::{ DEFAULT_PARTICLE_LIFETIME, SHORT_PARTICLE_LIFETIME };

/// Runs each tick, deleting particles who are past their expiry.
// Should make an addition to this to also spawn delayed particles,
// running through a list and removing the frame_time_ms from the
// delay. When delay is <= 0, make a particle_builder.request for
// the particle.
pub fn particle_ticker(ecs: &mut World, ctx: &BTerm) {
    cull_dead_particles(ecs, ctx);
    create_delayed_particles(ecs, ctx);
}

fn cull_dead_particles(ecs: &mut World, ctx: &BTerm) {
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

pub fn check_queue(ecs: &World) -> bool {
    let particle_builder = ecs.read_resource::<ParticleBuilder>();
    if particle_builder.delayed_requests.is_empty() && particle_builder.requests.is_empty() {
        return true;
    }
    return false;
}

fn create_delayed_particles(ecs: &mut World, ctx: &BTerm) {
    let mut particle_builder = ecs.write_resource::<ParticleBuilder>();
    let mut handled_particles: Vec<ParticleRequest> = Vec::new();
    for delayed_particle in particle_builder.delayed_requests.iter_mut() {
        delayed_particle.delay -= ctx.frame_time_ms;
        if delayed_particle.delay < 0.0 {
            handled_particles.push(ParticleRequest {
                x: delayed_particle.particle.x,
                y: delayed_particle.particle.y,
                fg: delayed_particle.particle.fg,
                bg: delayed_particle.particle.bg,
                glyph: delayed_particle.particle.glyph,
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
            .insert(p, Renderable {
                sprite: None, // TODO: Particle sprite
                fg: handled.fg,
                bg: handled.bg,
                glyph: handled.glyph,
                render_order: 0,
            })
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
    bg: RGB,
    glyph: FontCharType,
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
        bg: RGB,
        glyph: FontCharType,
        lifetime: f32
    ) {
        self.requests.push(ParticleRequest { x, y, fg, bg, glyph, lifetime });
    }

    pub fn delay(
        &mut self,
        x: i32,
        y: i32,
        fg: RGB,
        bg: RGB,
        glyph: FontCharType,
        lifetime: f32,
        delay: f32
    ) {
        self.delayed_requests.push(DelayedParticleRequest {
            delay: delay,
            particle: ParticleRequest { x, y, fg, bg, glyph, lifetime },
        });
    }

    pub fn damage_taken(&mut self, x: i32, y: i32) {
        self.request(
            x,
            y,
            RGB::named(ORANGE),
            RGB::named(BLACK),
            to_cp437('‼'),
            DEFAULT_PARTICLE_LIFETIME
        );
    }

    pub fn attack_miss(&mut self, x: i32, y: i32) {
        self.request(
            x,
            y,
            RGB::named(CYAN),
            RGB::named(BLACK),
            to_cp437('‼'),
            DEFAULT_PARTICLE_LIFETIME
        );
    }

    pub fn kick(&mut self, x: i32, y: i32) {
        self.request(
            x,
            y,
            RGB::named(CHOCOLATE),
            RGB::named(BLACK),
            to_cp437('‼'),
            SHORT_PARTICLE_LIFETIME
        );
    }

    // Makes a particle request in the shape of an 'x'. Sort of.
    #[allow(dead_code)]
    pub fn request_star(
        &mut self,
        x: i32,
        y: i32,
        fg: RGB,
        bg: RGB,
        glyph: FontCharType,
        lifetime: f32,
        secondary_fg: RGB
    ) {
        let eighth_l = lifetime / 8.0;
        let quarter_l = eighth_l * 2.0;
        self.request(x, y, fg, bg, glyph, lifetime);
        self.delay(
            x + 1,
            y + 1,
            secondary_fg.lerp(bg, 0.8),
            bg,
            to_cp437('/'),
            quarter_l,
            eighth_l
        );
        self.delay(
            x + 1,
            y - 1,
            secondary_fg.lerp(bg, 0.6),
            bg,
            to_cp437('\\'),
            quarter_l,
            quarter_l
        );
        self.delay(
            x - 1,
            y - 1,
            secondary_fg.lerp(bg, 0.2),
            bg,
            to_cp437('/'),
            quarter_l,
            eighth_l * 3.0
        );
        self.delay(
            x - 1,
            y + 1,
            secondary_fg.lerp(bg, 0.4),
            bg,
            to_cp437('\\'),
            quarter_l,
            lifetime
        );
    }

    // Makes a rainbow particle request in the shape of an 'x'. Sort of.
    #[allow(dead_code)]
    pub fn request_rainbow_star(&mut self, x: i32, y: i32, glyph: FontCharType, lifetime: f32) {
        let bg = RGB::named(BLACK);
        let eighth_l = lifetime / 8.0;
        let quarter_l = eighth_l * 2.0;
        let half_l = quarter_l * 2.0;

        self.request(x, y, RGB::named(CYAN), bg, glyph, lifetime);
        self.delay(x + 1, y + 1, RGB::named(RED), bg, to_cp437('\\'), half_l, eighth_l);
        self.delay(x + 1, y - 1, RGB::named(ORANGE), bg, to_cp437('/'), half_l, quarter_l);
        self.delay(x - 1, y - 1, RGB::named(GREEN), bg, to_cp437('\\'), half_l, eighth_l * 3.0);
        self.delay(x - 1, y + 1, RGB::named(YELLOW), bg, to_cp437('/'), half_l, half_l);
    }

    // Makes a rainbow particle request. Sort of.
    #[allow(dead_code)]
    pub fn request_rainbow(&mut self, x: i32, y: i32, glyph: FontCharType, lifetime: f32) {
        let bg = RGB::named(BLACK);
        let eighth_l = lifetime / 8.0;

        self.request(x, y, RGB::named(RED), bg, glyph, eighth_l);
        self.delay(x, y, RGB::named(ORANGE), bg, glyph, eighth_l, eighth_l);
        self.delay(x, y, RGB::named(YELLOW), bg, glyph, eighth_l, eighth_l * 2.0);
        self.delay(x, y, RGB::named(GREEN), bg, glyph, eighth_l, eighth_l * 3.0);
        self.delay(x, y, RGB::named(BLUE), bg, glyph, eighth_l, eighth_l * 4.0);
        self.delay(x, y, RGB::named(INDIGO), bg, glyph, eighth_l, eighth_l * 5.0);
        self.delay(x, y, RGB::named(VIOLET), bg, glyph, eighth_l, eighth_l * 6.0);
    }

    /// Makes a particle request in the shape of a +.
    #[allow(dead_code)]
    pub fn request_plus(
        &mut self,
        x: i32,
        y: i32,
        fg: RGB,
        bg: RGB,
        glyph: FontCharType,
        lifetime: f32
    ) {
        self.request(x, y, fg, bg, glyph, lifetime * 2.0);
        self.request(x + 1, y, fg, bg, to_cp437('─'), lifetime);
        self.request(x - 1, y, fg, bg, to_cp437('─'), lifetime);
        self.request(x, y + 1, fg, bg, to_cp437('│'), lifetime);
        self.request(x, y - 1, fg, bg, to_cp437('│'), lifetime);
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
                .insert(p, Renderable {
                    sprite: None, // TODO: Particle sprite
                    fg: new_particle.fg,
                    bg: new_particle.bg,
                    glyph: new_particle.glyph,
                    render_order: 0,
                })
                .expect("Could not insert renderables");
            particles
                .insert(p, ParticleLifetime { lifetime_ms: new_particle.lifetime })
                .expect("Could not insert lifetime");
        }
        particle_builder.requests.clear();
    }
}
