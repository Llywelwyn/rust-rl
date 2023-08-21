use super::{
    effects::{add_effect, EffectType, Targets},
    gamelog,
    gui::obfuscate_name_ecs,
    gui::renderable_colour,
    raws::Reaction,
    Attributes, BlocksTile, BlocksVisibility, Door, EntityMoved, Faction, HasAncestry, Hidden, HungerClock,
    HungerState, Item, Map, Name, ParticleBuilder, Player, Pools, Position, Renderable, RunState, State, Telepath,
    TileType, Viewshed, WantsToMelee, WantsToPickupItem,
};
use rltk::prelude::*;
use rltk::{Point, RandomNumberGenerator, Rltk, VirtualKeyCode};
use specs::prelude::*;
use std::cmp::{max, min};

pub fn try_door(i: i32, j: i32, ecs: &mut World) -> RunState {
    let mut positions = ecs.write_storage::<Position>();
    let mut players = ecs.write_storage::<Player>();
    let mut viewsheds = ecs.write_storage::<Viewshed>();
    let attributes = ecs.read_storage::<Attributes>();
    let map = ecs.fetch::<Map>();

    let entities = ecs.entities();
    let mut doors = ecs.write_storage::<Door>();
    let mut blocks_visibility = ecs.write_storage::<BlocksVisibility>();
    let mut blocks_movement = ecs.write_storage::<BlocksTile>();
    let mut renderables = ecs.write_storage::<Renderable>();
    let names = ecs.read_storage::<Name>();
    let mut rng = ecs.write_resource::<RandomNumberGenerator>();

    let mut result = RunState::AwaitingInput;
    let mut door_pos: Option<Point> = None;

    for (_entity, _player, pos, attributes) in (&entities, &mut players, &mut positions, &attributes).join() {
        let delta_x = i;
        let delta_y = j;

        if !(pos.x + delta_x < 0
            || pos.x + delta_x > map.width - 1
            || pos.y + delta_y < 0
            || pos.y + delta_y > map.height - 1)
        {
            let destination_idx = map.xy_idx(pos.x + delta_x, pos.y + delta_y);

            if !crate::spatial::has_tile_content(destination_idx) {
                gamelog::Logger::new().append("You see no door there.").log();
            }
            let mut multiple_tile_content = false;
            if crate::spatial::length(destination_idx) > 1 {
                multiple_tile_content = true;
            }
            crate::spatial::for_each_tile_content(destination_idx, |potential_target| {
                let door = doors.get_mut(potential_target);
                if let Some(door) = door {
                    if door.open == true {
                        if multiple_tile_content {
                            if let Some(name) = names.get(potential_target) {
                                gamelog::Logger::new().append("The").item_name(&name.name).append("is blocked.").log();
                            }
                        } else if rng.roll_dice(1, 6) + attributes.strength.bonus < 2 {
                            if let Some(name) = names.get(potential_target) {
                                gamelog::Logger::new().append("The").item_name(&name.name).append("resists!").log();
                            }
                        } else {
                            door.open = false;
                            blocks_visibility
                                .insert(potential_target, BlocksVisibility {})
                                .expect("Unable to insert BlocksVisibility.");
                            blocks_movement
                                .insert(potential_target, BlocksTile {})
                                .expect("Unable to insert BlocksTile.");
                            let render_data = renderables.get_mut(potential_target).unwrap();
                            if let Some(name) = names.get(potential_target) {
                                gamelog::Logger::new().append("You close the").item_name_n(&name.name).period().log();
                            }
                            render_data.glyph = rltk::to_cp437('+'); // Nethack open door, maybe just use '/' instead.
                            door_pos = Some(Point::new(pos.x + delta_x, pos.y + delta_y));
                        }
                        result = RunState::Ticking;
                    } else {
                        gamelog::Logger::new().append("It's already closed.").log();
                    }
                }
            });
        }
    }

    // If a door was interacted with, update every viewshed that could
    // see that door.
    for viewshed in (&mut viewsheds).join() {
        if let Some(door_pos) = door_pos {
            if viewshed.visible_tiles.contains(&door_pos) {
                viewshed.dirty = true;
            }
        }
    }

    return result;
}

pub fn open(i: i32, j: i32, ecs: &mut World) -> RunState {
    let mut positions = ecs.write_storage::<Position>();
    let mut players = ecs.write_storage::<Player>();
    let mut viewsheds = ecs.write_storage::<Viewshed>();
    let attributes = ecs.read_storage::<Attributes>();
    let map = ecs.fetch::<Map>();

    let entities = ecs.entities();
    let mut doors = ecs.write_storage::<Door>();
    let mut blocks_visibility = ecs.write_storage::<BlocksVisibility>();
    let mut blocks_movement = ecs.write_storage::<BlocksTile>();
    let mut renderables = ecs.write_storage::<Renderable>();
    let names = ecs.read_storage::<Name>();
    let mut rng = ecs.write_resource::<RandomNumberGenerator>();

    let mut result = RunState::AwaitingInput;
    let mut door_pos: Option<Point> = None;

    for (_entity, _player, pos, attributes) in (&entities, &mut players, &mut positions, &attributes).join() {
        let delta_x = i;
        let delta_y = j;

        if !(pos.x + delta_x < 0
            || pos.x + delta_x > map.width - 1
            || pos.y + delta_y < 0
            || pos.y + delta_y > map.height - 1)
        {
            let destination_idx = map.xy_idx(pos.x + delta_x, pos.y + delta_y);

            if !crate::spatial::has_tile_content(destination_idx) {
                gamelog::Logger::new().append("You see no door there.").log();
            }
            crate::spatial::for_each_tile_content(destination_idx, |potential_target| {
                let door = doors.get_mut(potential_target);
                if let Some(door) = door {
                    if door.open == false {
                        if rng.roll_dice(1, 6) + attributes.strength.bonus < 2 {
                            if let Some(name) = names.get(potential_target) {
                                gamelog::Logger::new().append("The").item_name(&name.name).append("resists!").log();
                            }
                        } else {
                            door.open = true;
                            blocks_visibility.remove(potential_target);
                            blocks_movement.remove(potential_target);
                            let render_data = renderables.get_mut(potential_target).unwrap();
                            if let Some(name) = names.get(potential_target) {
                                gamelog::Logger::new().append("You open the").item_name_n(&name.name).period().log();
                            }
                            render_data.glyph = rltk::to_cp437('▓'); // Nethack open door, maybe just use '/' instead.
                            door_pos = Some(Point::new(pos.x + delta_x, pos.y + delta_y));
                        }
                        result = RunState::Ticking;
                    } else {
                        gamelog::Logger::new().append("It's already open.").log();
                    }
                }
            });
        }
    }

    // If a door was interacted with, update every viewshed that could
    // see that door.
    for viewshed in (&mut viewsheds).join() {
        if let Some(door_pos) = door_pos {
            if viewshed.visible_tiles.contains(&door_pos) {
                viewshed.dirty = true;
            }
        }
    }

    return result;
}

pub fn kick(i: i32, j: i32, ecs: &mut World) -> RunState {
    let mut something_was_destroyed: Option<Entity> = None;
    let mut destroyed_pos: Option<Point> = None;
    {
        let mut positions = ecs.write_storage::<Position>();
        let mut players = ecs.write_storage::<Player>();
        let mut viewsheds = ecs.write_storage::<Viewshed>();
        let attributes = ecs.read_storage::<Attributes>();
        let map = ecs.fetch::<Map>();

        let entities = ecs.entities();
        let mut doors = ecs.write_storage::<Door>();
        let names = ecs.read_storage::<Name>();
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();

        for (entity, _player, pos, attributes) in (&entities, &mut players, &mut positions, &attributes).join() {
            let delta_x = i;
            let delta_y = j;

            if !(pos.x + delta_x < 0
                || pos.x + delta_x > map.width - 1
                || pos.y + delta_y < 0
                || pos.y + delta_y > map.height - 1)
            {
                let destination_idx = map.xy_idx(pos.x + delta_x, pos.y + delta_y);

                if !crate::spatial::has_tile_content(destination_idx) {
                    if rng.roll_dice(1, 20) == 20 {
                        add_effect(None, EffectType::Damage { amount: 1 }, Targets::Entity { target: entity });
                        gamelog::Logger::new().append("Ouch! You kick the open air, and pull something.").log();
                        break;
                    } else {
                        // If there's nothing at all, just kick the air and waste a turn.
                        gamelog::Logger::new().append("You kick the open air.").log();
                        break;
                    }
                } else {
                    let mut last_non_door_target: Option<Entity> = None;
                    let mut target_name = "thing";
                    crate::spatial::for_each_tile_content_with_bool(destination_idx, |potential_target| {
                        if let Some(name) = names.get(potential_target) {
                            target_name = &name.name;
                        }

                        // If it's a door,
                        let door = doors.get_mut(potential_target);
                        if let Some(door) = door {
                            // If the door is closed,
                            if door.open == false {
                                let mut particle_builder = ecs.write_resource::<ParticleBuilder>();
                                particle_builder.kick(pos.x + delta_x, pos.y + delta_y);
                                // ~33% chance of breaking it down + str
                                if rng.roll_dice(1, 10) + attributes.strength.bonus > 6 {
                                    gamelog::Logger::new()
                                        .append("As you kick the")
                                        .item_name_n(target_name)
                                        .append(", it crashes open!")
                                        .log();
                                    something_was_destroyed = Some(potential_target);
                                    destroyed_pos = Some(Point::new(pos.x + delta_x, pos.y + delta_y));
                                    gamelog::record_event("broken_doors", 1);
                                    return false;
                                // 66% chance of just kicking it.
                                } else {
                                    gamelog::Logger::new()
                                        .append("You kick the")
                                        .item_name_n(target_name)
                                        .period()
                                        .log();
                                    return false;
                                }
                            // If the door is open and there's nothing else on the tile,
                            } else if crate::spatial::length(destination_idx) == 1 {
                                // Just kick the air.
                                gamelog::Logger::new().append("You kick the open air.").log();
                                return false;
                            }
                        } else {
                            last_non_door_target = Some(potential_target);
                        }
                        return true;
                    });
                    if let Some(_) = last_non_door_target {
                        gamelog::Logger::new().append("You kick the").item_name_n(target_name).period().log();
                        let mut particle_builder = ecs.write_resource::<ParticleBuilder>();
                        particle_builder.kick(pos.x + delta_x, pos.y + delta_y);
                        // Do something here if it's anything other than a door.
                        break;
                    }
                }
            }
        }
        if let Some(destroyed_pos) = destroyed_pos {
            for viewshed in (&mut viewsheds).join() {
                if viewshed.visible_tiles.contains(&destroyed_pos) {
                    viewshed.dirty = true;
                }
            }
        }
    }
    if let Some(destroyed_thing) = something_was_destroyed {
        ecs.delete_entity(destroyed_thing).expect("Unable to delete.");
    }

    gamelog::record_event("kick_count", 1);
    return RunState::Ticking;
}

pub fn try_move_player(delta_x: i32, delta_y: i32, ecs: &mut World) -> RunState {
    let mut positions = ecs.write_storage::<Position>();
    let mut players = ecs.write_storage::<Player>();
    let mut viewsheds = ecs.write_storage::<Viewshed>();
    let mut telepaths = ecs.write_storage::<Telepath>();
    let mut entity_moved = ecs.write_storage::<EntityMoved>();
    let factions = ecs.read_storage::<Faction>();
    let ancestries = ecs.read_storage::<HasAncestry>();
    let pools = ecs.read_storage::<Pools>();
    let map = ecs.fetch::<Map>();
    let entities = ecs.entities();
    let mut wants_to_melee = ecs.write_storage::<WantsToMelee>();
    let mut doors = ecs.write_storage::<Door>();
    let names = ecs.read_storage::<Name>();
    let mut swap_entities: Vec<(Entity, i32, i32)> = Vec::new();
    let mut result: Option<RunState>;

    for (entity, _player, pos, viewshed) in (&entities, &mut players, &mut positions, &mut viewsheds).join() {
        if pos.x + delta_x < 0
            || pos.x + delta_x > map.width - 1
            || pos.y + delta_y < 0
            || pos.y + delta_y > map.height - 1
        {
            return RunState::AwaitingInput;
        }
        let destination_idx = map.xy_idx(pos.x + delta_x, pos.y + delta_y);

        result = crate::spatial::for_each_tile_content_with_runstate(destination_idx, |potential_target| {
            let mut hostile = true;
            if pools.get(potential_target).is_some() {
                let result = crate::raws::get_reactions(
                    entity,
                    potential_target,
                    &factions,
                    &ancestries,
                    &crate::raws::RAWS.lock().unwrap(),
                );
                if result != Reaction::Attack {
                    hostile = false;
                }
            }
            if !hostile {
                swap_entities.push((potential_target, pos.x, pos.y));
                pos.x = min(map.width - 1, max(0, pos.x + delta_x));
                pos.y = min(map.height - 1, max(0, pos.y + delta_y));
                entity_moved.insert(entity, EntityMoved {}).expect("Unable to insert marker");
                viewshed.dirty = true;
                let mut ppos = ecs.write_resource::<Point>();
                ppos.x = pos.x;
                ppos.y = pos.y;
            } else {
                let target = pools.get(potential_target);
                if let Some(_target) = target {
                    wants_to_melee
                        .insert(entity, WantsToMelee { target: potential_target })
                        .expect("Add target failed.");
                    return Some(RunState::Ticking);
                }
            }
            let door = doors.get_mut(potential_target);
            if let Some(door) = door {
                if door.open == false {
                    if let Some(name) = names.get(potential_target) {
                        gamelog::Logger::new().append("The").item_name(&name.name).append("is in your way.").log();
                    }
                    return Some(RunState::AwaitingInput);
                }
            }
            return None;
        });

        if result.is_some() {
            return result.unwrap();
        }

        if swap_entities.len() <= 0 {
            if crate::spatial::is_blocked(destination_idx) {
                gamelog::Logger::new().append("You can't move there.").log();
                return RunState::AwaitingInput;
            }
            let hidden = ecs.read_storage::<Hidden>();
            // Push every entity name in the pile to a vector of strings
            let mut item_names: Vec<String> = Vec::new();
            let mut some = false;
            crate::spatial::for_each_tile_content(destination_idx, |entity| {
                if !hidden.get(entity).is_some() && names.get(entity).is_some() {
                    let item_name = obfuscate_name_ecs(ecs, entity).0;
                    item_names.push(item_name);
                    some = true;
                }
            });
            // If some names were found, append. Logger = logger is necessary
            // makes logger called a mutable self. It's not the most efficient
            // but it happens infrequently enough (once per player turn at most)
            // that it shouldn't matter.
            if some {
                let mut logger = gamelog::Logger::new().append("You see a");
                for i in 0..item_names.len() {
                    if i > 0 && i < item_names.len() {
                        logger = logger.append(", a");
                    }
                    logger = logger.item_name_n(&item_names[i]);
                }
                logger.period().log();
            }
            let old_idx = map.xy_idx(pos.x, pos.y);
            pos.x = min(map.width - 1, max(0, pos.x + delta_x));
            pos.y = min(map.height - 1, max(0, pos.y + delta_y));
            let new_idx = map.xy_idx(pos.x, pos.y);
            entity_moved.insert(entity, EntityMoved {}).expect("Unable to insert marker");
            crate::spatial::move_entity(entity, old_idx, new_idx);
            // Dirty viewsheds, and check only now if telepath viewshed exists
            viewshed.dirty = true;
            if let Some(telepathy) = telepaths.get_mut(entity) {
                telepathy.dirty = true;
            }
            let mut ppos = ecs.write_resource::<Point>();
            ppos.x = pos.x;
            ppos.y = pos.y;
            return RunState::Ticking;
        }
    }

    for m in swap_entities.iter() {
        if let Some(name) = names.get(m.0) {
            let renderables = ecs.read_storage::<Renderable>();
            gamelog::Logger::new()
                .append("You swap places with the")
                .colour(renderable_colour(&renderables, m.0))
                .append_n(&name.name)
                .colour(WHITE)
                .period()
                .log();
        }
        if let Some(their_pos) = positions.get_mut(m.0) {
            let old_idx = map.xy_idx(their_pos.x, their_pos.y);
            their_pos.x = m.1;
            their_pos.y = m.2;
            let new_idx = map.xy_idx(their_pos.x, their_pos.y);
            crate::spatial::move_entity(m.0, old_idx, new_idx);
            return RunState::Ticking;
        }
    }
    return RunState::AwaitingInput;
}

fn get_item(ecs: &mut World) -> RunState {
    let player_pos = ecs.fetch::<Point>();
    let player_entity = ecs.fetch::<Entity>();
    let entities = ecs.entities();
    let items = ecs.read_storage::<Item>();
    let positions = ecs.read_storage::<Position>();

    let mut target_item: Option<Entity> = None;
    for (item_entity, _item, position) in (&entities, &items, &positions).join() {
        if position.x == player_pos.x && position.y == player_pos.y {
            target_item = Some(item_entity);
        }
    }

    match target_item {
        None => {
            gamelog::Logger::new().append("There is nothing to pick up.").log();
            return RunState::AwaitingInput;
        }
        Some(item) => {
            let mut pickup = ecs.write_storage::<WantsToPickupItem>();
            pickup
                .insert(*player_entity, WantsToPickupItem { collected_by: *player_entity, item })
                .expect("Unable to insert want to pickup item.");
            return RunState::Ticking;
        }
    }
}

pub fn player_input(gs: &mut State, ctx: &mut Rltk) -> RunState {
    match ctx.key {
        None => return RunState::AwaitingInput,
        Some(key) => match key {
            // Cardinals
            VirtualKeyCode::Left | VirtualKeyCode::Numpad4 | VirtualKeyCode::H => {
                return try_move_player(-1, 0, &mut gs.ecs)
            }
            VirtualKeyCode::Right | VirtualKeyCode::Numpad6 | VirtualKeyCode::L => {
                return try_move_player(1, 0, &mut gs.ecs);
            }
            VirtualKeyCode::Up | VirtualKeyCode::Numpad8 | VirtualKeyCode::K => {
                return try_move_player(0, -1, &mut gs.ecs);
            }
            VirtualKeyCode::Down | VirtualKeyCode::Numpad2 | VirtualKeyCode::J => {
                return try_move_player(0, 1, &mut gs.ecs);
            }
            // Diagonals
            VirtualKeyCode::Numpad9 | VirtualKeyCode::U => return try_move_player(1, -1, &mut gs.ecs),
            VirtualKeyCode::Numpad7 | VirtualKeyCode::Y => return try_move_player(-1, -1, &mut gs.ecs),
            VirtualKeyCode::Numpad3 | VirtualKeyCode::N => return try_move_player(1, 1, &mut gs.ecs),
            VirtualKeyCode::Numpad1 | VirtualKeyCode::B => return try_move_player(-1, 1, &mut gs.ecs),
            // id
            VirtualKeyCode::Period => {
                if ctx.shift {
                    if !try_next_level(&mut gs.ecs) {
                        return RunState::AwaitingInput;
                    }
                    return RunState::NextLevel; // > to descend
                } else {
                    return skip_turn(&mut gs.ecs); // (Wait a turn)
                }
            }
            VirtualKeyCode::Comma => {
                if ctx.shift {
                    if !try_previous_level(&mut gs.ecs) {
                        return RunState::AwaitingInput;
                    }
                    return RunState::PreviousLevel; // < to ascend
                }
            }
            VirtualKeyCode::Slash => {
                if ctx.shift {
                    return RunState::HelpScreen;
                }
            }
            VirtualKeyCode::NumpadDecimal => {
                return skip_turn(&mut gs.ecs);
            }

            // Items
            VirtualKeyCode::C => return RunState::ActionWithDirection { function: try_door },
            VirtualKeyCode::O => return RunState::ActionWithDirection { function: open },
            VirtualKeyCode::F => return RunState::ActionWithDirection { function: kick },
            VirtualKeyCode::G => {
                return get_item(&mut gs.ecs);
            }
            VirtualKeyCode::I => return RunState::ShowInventory,
            VirtualKeyCode::D => return RunState::ShowDropItem,
            VirtualKeyCode::R => return RunState::ShowRemoveItem,
            // Other
            VirtualKeyCode::Minus => return RunState::ShowCheatMenu,
            VirtualKeyCode::Escape => return RunState::SaveGame,
            _ => {
                return RunState::AwaitingInput;
            }
        },
    }
    return RunState::AwaitingInput;
}

pub fn try_next_level(ecs: &mut World) -> bool {
    let player_pos = ecs.fetch::<Point>();
    let map = ecs.fetch::<Map>();
    let player_idx = map.xy_idx(player_pos.x, player_pos.y);
    if map.tiles[player_idx] == TileType::DownStair {
        return true;
    } else {
        gamelog::Logger::new().append("You don't see a way down from here.").log();
        return false;
    }
}

pub fn try_previous_level(ecs: &mut World) -> bool {
    let player_pos = ecs.fetch::<Point>();
    let map = ecs.fetch::<Map>();
    let player_idx = map.xy_idx(player_pos.x, player_pos.y);
    if map.tiles[player_idx] == TileType::UpStair {
        return true;
    } else {
        gamelog::Logger::new().append("You don't see a way up from here.").log();
        return false;
    }
}

fn skip_turn(ecs: &mut World) -> RunState {
    let player_entity = ecs.fetch::<Entity>();
    let mut viewsheds = ecs.write_storage::<Viewshed>();
    let worldmap_resource = ecs.fetch::<Map>();
    let hunger_clocks = ecs.read_storage::<HungerClock>();

    // Default to being able to heal by waiting.
    let mut can_heal = true;
    let factions = ecs.read_storage::<Faction>();
    let ancestries = ecs.read_storage::<HasAncestry>();

    // Check viewshed for monsters nearby. If we can see a monster, we can't heal.
    let viewshed = viewsheds.get_mut(*player_entity).unwrap();
    for tile in viewshed.visible_tiles.iter() {
        let idx = worldmap_resource.xy_idx(tile.x, tile.y);
        crate::spatial::for_each_tile_content(idx, |entity_id| {
            let result = crate::raws::get_reactions(
                *player_entity,
                entity_id,
                &factions,
                &ancestries,
                &crate::raws::RAWS.lock().unwrap(),
            );
            if result == Reaction::Attack {
                can_heal = false;
            }
        });
    }
    // Dirty viewshed (so we search for hidden tiles whenever we wait)
    viewshed.dirty = true;

    // Check player's hunger state - if we're hungry or worse, we can't heal.
    let player_hunger_clock = hunger_clocks.get(*player_entity);
    if let Some(clock) = player_hunger_clock {
        match clock.state {
            HungerState::Hungry => can_heal = false,
            HungerState::Weak => can_heal = false,
            HungerState::Fainting => can_heal = false,
            _ => {}
        }
    }

    if can_heal {
        let mut health_components = ecs.write_storage::<Pools>();
        let pools = health_components.get_mut(*player_entity).unwrap();
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();
        let roll = rng.roll_dice(1, 6);
        if (roll == 6) && pools.hit_points.current < pools.hit_points.max {
            pools.hit_points.current += 1;
        }
    }

    gamelog::Logger::new().append("You wait a turn.").log();

    return RunState::Ticking;
}

/* Playing around with autoexplore, without having read how to do it.
pub fn auto_explore(ecs: &mut World) {
    let player_pos = ecs.fetch::<Point>();
    let positions = ecs.read_storage::<Position>();
    let entities = ecs.entities();
    let map = ecs.fetch::<Map>();
    let mut viewsheds = ecs.write_storage::<Viewshed>();

    let mut unexplored_tiles: Vec<usize> = vec![];
    for (idx, _tile) in map.tiles.iter().enumerate() {
        if !map.revealed_tiles[idx] {
            unexplored_tiles.push(idx);
        }
    }
    let mut unexplored_tile = (0, 0.0f32);

    let flow_map = DijkstraMap::new_empty(MAPWIDTH, MAPHEIGHT, 200.0);

    DijkstraMap::build(&mut flow_map, &unexplored_tiles, &map);
    for (i, tile) in map.tiles.iter().enumerate() {
        if *tile == TileType::Floor {
            let distance_to_start = flow_map.map[i];

            if distance_to_start > unexplored_tile.1 {
                unexplored_tile.0 = i;
                unexplored_tile.1 = distance_to_start;
            }
        }
    }

    let path = rltk::a_star_search(map.xy_idx(player_pos.x, player_pos.y), unexplored_tile.0, &*map);
    if path.success && path.steps.len() > 1 {
        let mut idx = map.xy_idx(player_pos.x, player_pos.y);
        map.blocked[idx] = false;
        player_pos.x = (path.steps[1] as i32) % map.width;
        player_pos.y = (path.steps[1] as i32) / map.width;
        idx = map.xy_idx(player_pos.x, player_pos.y);
        map.blocked[idx] = true;
        for (ent, viewshed, pos) in (&entities, &mut viewsheds, &positions).join() {
            viewshed.dirty = true;
        }
    }
}
*/
