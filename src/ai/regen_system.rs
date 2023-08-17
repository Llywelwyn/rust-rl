use crate::{gamelog, Clock, Player, Pools, Position, TakingTurn};
use specs::prelude::*;

pub struct RegenSystem {}

const MONSTER_HP_REGEN_TURN: i32 = 20;
const MONSTER_HP_REGEN_PER_TICK: i32 = 1;

impl<'a> System<'a> for RegenSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        ReadStorage<'a, Clock>,
        Entities<'a>,
        ReadStorage<'a, Position>,
        WriteStorage<'a, Pools>,
        ReadStorage<'a, TakingTurn>,
        ReadStorage<'a, Player>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (clock, entities, positions, mut pools, turns, player) = data;
        let mut clock_turn = false;
        for (_e, _c, _t) in (&entities, &clock, &turns).join() {
            clock_turn = true;
        }
        if !clock_turn {
            return;
        }
        let current_turn = gamelog::get_event_count("turns");
        if current_turn % MONSTER_HP_REGEN_TURN == 0 {
            for (_e, _p, pool, _player) in (&entities, &positions, &mut pools, !&player).join() {
                try_hp_regen_tick(pool, MONSTER_HP_REGEN_PER_TICK);
            }
        }
        let level = gamelog::get_event_count("player_level");
        if current_turn % get_player_hp_regen_turn(level) == 0 {
            for (_e, _p, pool, _player) in (&entities, &positions, &mut pools, &player).join() {
                try_hp_regen_tick(pool, get_player_hp_regen_per_tick(level));
            }
        }
    }
}

fn get_player_hp_regen_turn(level: i32) -> i32 {
    if level < 10 {
        return (42 / (level + 2)) + 1;
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
