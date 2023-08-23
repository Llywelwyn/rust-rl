use crate::{
    gamelog,
    gui::Class,
    Attributes,
    Clock,
    HasClass,
    Player,
    Pools,
    Position,
    RandomNumberGenerator,
    TakingTurn,
};
use specs::prelude::*;

pub struct RegenSystem {}

const MONSTER_HP_REGEN_TURN: i32 = 20;
const MONSTER_HP_REGEN_PER_TICK: i32 = 1;

const WIZARD_MP_REGEN_MOD: i32 = 3;
const NONWIZARD_MP_REGEN_MOD: i32 = 4;
const MP_REGEN_BASE: i32 = 38;
const MP_REGEN_DIVISOR: i32 = 6;
const MIN_MP_REGEN_PER_TURN: i32 = 1;

impl<'a> System<'a> for RegenSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        ReadStorage<'a, Clock>,
        Entities<'a>,
        ReadStorage<'a, Position>,
        WriteStorage<'a, Pools>,
        ReadStorage<'a, TakingTurn>,
        ReadStorage<'a, Player>,
        ReadStorage<'a, HasClass>,
        ReadStorage<'a, Attributes>,
        WriteExpect<'a, RandomNumberGenerator>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (clock, entities, positions, mut pools, turns, player, classes, attributes, mut rng) = data;
        let mut clock_turn = false;
        for (_e, _c, _t) in (&entities, &clock, &turns).join() {
            clock_turn = true;
        }
        if !clock_turn {
            return;
        }
        // Monster HP regen
        let current_turn = gamelog::get_event_count("turns");
        if current_turn % MONSTER_HP_REGEN_TURN == 0 {
            for (_e, _p, pool, _player) in (&entities, &positions, &mut pools, !&player).join() {
                try_hp_regen_tick(pool, MONSTER_HP_REGEN_PER_TICK);
            }
        }
        // Player HP regen
        let level = gamelog::get_event_count("player_level");
        if current_turn % get_player_hp_regen_turn(level) == 0 {
            for (_e, _p, pool, _player) in (&entities, &positions, &mut pools, &player).join() {
                try_hp_regen_tick(pool, get_player_hp_regen_per_tick(level));
            }
        }
        // Both MP regen
        for (e, _p, pool) in (&entities, &positions, &mut pools).join() {
            let is_wizard = if let Some(class) = classes.get(e) { class.name == Class::Wizard } else { false };
            let numerator = if is_wizard { WIZARD_MP_REGEN_MOD } else { NONWIZARD_MP_REGEN_MOD };
            let multiplier: f32 = (numerator as f32) / (MP_REGEN_DIVISOR as f32);
            let mp_regen_tick = (((MP_REGEN_BASE - pool.level) as f32) * multiplier) as i32;
            if current_turn % mp_regen_tick == 0 {
                try_mana_regen_tick(pool, rng.roll_dice(1, get_mana_regen_per_tick(e, &attributes)));
            }
        }
    }
}

fn get_player_hp_regen_turn(level: i32) -> i32 {
    if level < 10 {
        return 42 / (level + 2) + 1;
    } else {
        return 3;
    }
}

fn get_player_hp_regen_per_tick(level: i32) -> i32 {
    if level < 10 {
        return 1;
    } else {
        return 2;
    }
}

fn try_hp_regen_tick(pool: &mut Pools, amount: i32) {
    pool.hit_points.current = i32::min(pool.hit_points.current + amount, pool.hit_points.max);
}

fn get_mana_regen_per_tick(e: Entity, attributes: &ReadStorage<Attributes>) -> i32 {
    let regen = if let Some(attributes) = attributes.get(e) {
        (attributes.intelligence.bonus + attributes.wisdom.bonus) / 2 + MIN_MP_REGEN_PER_TURN
    } else {
        MIN_MP_REGEN_PER_TURN
    };
    return i32::max(regen, 1);
}

fn try_mana_regen_tick(pool: &mut Pools, amount: i32) {
    pool.mana.current = i32::min(pool.mana.current + amount, pool.mana.max);
}
