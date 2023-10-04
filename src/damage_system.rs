use super::{
    gamelog,
    gui::renderable_colour,
    Equipped,
    InBackpack,
    Item,
    LootTable,
    Name,
    Player,
    Pools,
    Position,
    Renderable,
    RunState,
};
use bracket_lib::prelude::*;
use specs::prelude::*;
use crate::consts::events;

pub fn delete_the_dead(ecs: &mut World) {
    let mut dead: Vec<Entity> = Vec::new();
    // Using scope to make borrow checker happy
    {
        let combat_stats = ecs.read_storage::<Pools>();
        let players = ecs.read_storage::<Player>();
        let names = ecs.read_storage::<Name>();
        let items = ecs.read_storage::<Item>();
        let entities = ecs.entities();
        let renderables = ecs.read_storage::<Renderable>();
        for (entity, stats) in (&entities, &combat_stats).join() {
            if stats.hit_points.current < 1 {
                let player = players.get(entity);
                match player {
                    None => {
                        let victim_name = names.get(entity);
                        if let Some(victim_name) = victim_name {
                            let item = items.get(entity);
                            if let Some(_item) = item {
                                gamelog::Logger
                                    ::new()
                                    .append("The")
                                    .colour(renderable_colour(&renderables, entity))
                                    .append(&victim_name.name)
                                    .colour(WHITE)
                                    .append("is destroyed!")
                                    .log();
                            } else {
                                gamelog::Logger
                                    ::new()
                                    .append("The")
                                    .colour(renderable_colour(&renderables, entity))
                                    .append(&victim_name.name)
                                    .colour(WHITE)
                                    .append("dies!")
                                    .log();
                            }
                        }
                        dead.push(entity);
                    }
                    // The player died, go to GameOver.
                    Some(_) => {
                        let mut runstate = ecs.write_resource::<RunState>();
                        *runstate = RunState::GameOver;
                    }
                }
            }
        }
    }
    let (items_to_delete, loot_to_spawn) = handle_dead_entity_items(ecs, &dead);
    for loot in loot_to_spawn {
        crate::raws::spawn_named_entity(
            &crate::raws::RAWS.lock().unwrap(),
            ecs,
            &loot.0,
            None,
            crate::raws::SpawnType::AtPosition { x: loot.1.x, y: loot.1.y },
            0
        );
    }
    for item in items_to_delete {
        ecs.delete_entity(item).expect("Unable to delete item.");
    }
    // For everything that died, increment the event log, and delete.
    for victim in dead {
        gamelog::record_event(events::EVENT::Turn(1));
        // TODO: Delete stuff from inventory? This should be handled elsewhere.
        ecs.delete_entity(victim).expect("Unable to delete.");
    }
}

fn handle_dead_entity_items(
    ecs: &mut World,
    dead: &Vec<Entity>
) -> (Vec<Entity>, Vec<(String, Position)>) {
    let mut to_drop: Vec<(Entity, Position)> = Vec::new();
    let mut to_spawn: Vec<(String, Position)> = Vec::new();
    let entities = ecs.entities();
    let mut equipped = ecs.write_storage::<Equipped>();
    let mut carried = ecs.write_storage::<InBackpack>();
    let mut positions = ecs.write_storage::<Position>();
    let loot_tables = ecs.read_storage::<LootTable>();
    let mut rng = ecs.write_resource::<RandomNumberGenerator>();
    // Make list of every item in every dead thing's inv/equip
    for victim in dead.iter() {
        let pos = positions.get(*victim);
        for (entity, equipped) in (&entities, &equipped).join() {
            if equipped.owner == *victim {
                // Push equipped item entities and positions
                if let Some(pos) = pos {
                    to_drop.push((entity, pos.clone()));
                }
            }
        }
        for (entity, backpack) in (&entities, &carried).join() {
            if backpack.owner == *victim {
                // Push backpack item entities and positions
                if let Some(pos) = pos {
                    to_drop.push((entity, pos.clone()));
                }
            }
        }
        if let Some(table) = loot_tables.get(*victim) {
            let roll: f32 = rng.rand();
            if roll < table.chance {
                let potential_drop = crate::raws::roll_on_loot_table(
                    &crate::raws::RAWS.lock().unwrap(),
                    &mut rng,
                    &table.table
                );
                if let Some(id) = potential_drop {
                    if let Some(pos) = pos {
                        to_spawn.push((id, pos.clone()));
                    }
                }
            }
        }
    }
    const DROP_ONE_IN_THIS_MANY_TIMES: i32 = 6;
    let mut to_return: Vec<Entity> = Vec::new();
    for drop in to_drop.iter() {
        if rng.roll_dice(1, DROP_ONE_IN_THIS_MANY_TIMES) == 1 {
            equipped.remove(drop.0);
            carried.remove(drop.0);
            positions.insert(drop.0, drop.1.clone()).expect("Unable to insert Position{}.");
        } else {
            to_return.push(drop.0);
        }
    }
    return (to_return, to_spawn);
}
