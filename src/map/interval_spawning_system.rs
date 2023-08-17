use crate::{gamelog, raws, spawner, Clock, Map, RandomNumberGenerator, TakingTurn, LOG_SPAWNING};
use specs::prelude::*;

const TRY_SPAWN_CHANCE: i32 = 70;

pub fn try_spawn_interval(ecs: &mut World) {
    let mut try_spawn = false;
    // Scope for borrow checker (ECS)
    {
        let map = ecs.fetch::<Map>();
        // Difficulty 0 maps shouldn't have respawning hostile mobs.
        if map.difficulty == 0 {
            return;
        }
        let clock = ecs.read_storage::<Clock>();
        let turns = ecs.read_storage::<TakingTurn>();
        let mut rng = ecs.write_resource::<rltk::RandomNumberGenerator>();
        for (_c, _t) in (&clock, &turns).join() {
            if rng.roll_dice(1, TRY_SPAWN_CHANCE) == 1 {
                try_spawn = true;
            }
        }
    }
    if try_spawn {
        if LOG_SPAWNING {
            rltk::console::log("SPAWNINFO: Trying spawn.");
        }
        spawn_random_mob_in_free_nonvisible_tile(ecs);
    }
}

fn spawn_random_mob_in_free_nonvisible_tile(ecs: &mut World) {
    let map = ecs.fetch::<Map>();
    let mut available_tiles = populate_unblocked_nonvisible(&map);
    let player_level = gamelog::get_event_count("player_level");
    rltk::console::log(player_level);
    let difficulty = (map.difficulty + player_level) / 2;
    if available_tiles.len() == 0 {
        if LOG_SPAWNING {
            rltk::console::log("SPAWNINFO: No free tiles; not spawning anything..");
        }
        return;
    }
    let mut spawn_locations: Vec<(i32, i32)> = Vec::new();
    let mut rng = ecs.write_resource::<RandomNumberGenerator>();
    let key = spawner::mob_table(difficulty).roll(&mut rng);
    let spawn_type = raws::get_mob_spawn_type(&raws::RAWS.lock().unwrap(), &key);
    let roll = raws::get_mob_spawn_amount(&mut rng, &spawn_type, player_level);
    for _i in 0..roll {
        let idx = get_random_idx_from_tiles(&mut rng, &mut available_tiles);
        spawn_locations.push((idx as i32 % map.width, idx as i32 / map.width));
    }
    // Dropping resources for borrow-checker.
    std::mem::drop(map);
    std::mem::drop(rng);
    // For every idx in the spawn list, spawn mob.
    for idx in spawn_locations {
        if LOG_SPAWNING {
            rltk::console::log(format!("SPAWNINFO: Spawning {} at {}, {}.", key, idx.0, idx.1));
        }
        raws::spawn_named_entity(
            &raws::RAWS.lock().unwrap(),
            ecs,
            &key,
            raws::SpawnType::AtPosition { x: idx.0, y: idx.1 },
            difficulty,
        );
    }
}

/// Returns a Vec<usize> of every tile that is not blocked, and is not currently in the player's view.
fn populate_unblocked_nonvisible(map: &Map) -> Vec<usize> {
    let mut tiles: Vec<usize> = Vec::new();
    for (i, _tile) in map.tiles.iter().enumerate() {
        if !crate::spatial::is_blocked(i) && !map.visible_tiles[i] {
            tiles.push(i);
        }
    }
    return tiles;
}

/// Picks a random index from a vector of indexes, and removes it from the vector.
fn get_random_idx_from_tiles(rng: &mut rltk::RandomNumberGenerator, area: &mut Vec<usize>) -> usize {
    let idx = if area.len() == 1 { 0usize } else { (rng.roll_dice(1, area.len() as i32) - 1) as usize };
    area.remove(idx);
    return area[idx];
}
