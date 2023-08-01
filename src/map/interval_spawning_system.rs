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
    let available_tiles = populate_unblocked_nonvisible_tiles(&map);
    let difficulty = (map.difficulty + gamelog::get_event_count("player_level")) / 2;
    if available_tiles.len() == 0 {
        if LOG_SPAWNING {
            rltk::console::log("SPAWNINFO: No free tiles; not spawning anything..");
        }
        return;
    }
    let mut rng = ecs.write_resource::<RandomNumberGenerator>();
    let idx = get_random_idx_from_possible_tiles(&mut rng, available_tiles);
    let key = spawner::mob_table(difficulty).roll(&mut rng);
    let x = idx as i32 % map.width;
    let y = idx as i32 / map.width;
    std::mem::drop(map);
    std::mem::drop(rng);
    if LOG_SPAWNING {
        rltk::console::log(format!("SPAWNINFO: Spawning {} at {}, {}.", key, x, y));
    }
    raws::spawn_named_entity(&raws::RAWS.lock().unwrap(), ecs, &key, raws::SpawnType::AtPosition { x, y }, difficulty);
}

fn populate_unblocked_nonvisible_tiles(map: &Map) -> Vec<usize> {
    let mut tiles: Vec<usize> = Vec::new();
    for (i, _tile) in map.tiles.iter().enumerate() {
        if !map.blocked[i] && !map.visible_tiles[i] {
            tiles.push(i);
        }
    }
    return tiles;
}

fn get_random_idx_from_possible_tiles(rng: &mut rltk::RandomNumberGenerator, area: Vec<usize>) -> usize {
    let idx = if area.len() == 1 { 0usize } else { (rng.roll_dice(1, area.len() as i32) - 1) as usize };
    return area[idx];
}
