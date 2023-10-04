use super::{
    effects::{ add_effect, EffectType, Targets },
    gamelog,
    gui::obfuscate_name_ecs,
    gui::renderable_colour_ecs,
    gui::item_colour_ecs,
    camera::get_screen_bounds,
    raws::Reaction,
    Attributes,
    BlocksTile,
    BlocksVisibility,
    Door,
    EntityMoved,
    Faction,
    HasAncestry,
    Hidden,
    HungerClock,
    HungerState,
    Item,
    Map,
    Name,
    Player,
    Pools,
    Position,
    Renderable,
    RunState,
    states::state::*,
    Telepath,
    TileType,
    Viewshed,
    WantsToMelee,
    WantsToPickupItem,
    WantsToAssignKey,
    get_dest,
    Destination,
    DamageType,
};
use bracket_lib::prelude::*;
use specs::prelude::*;
use std::cmp::{ max, min };
use crate::consts::events::*;
use crate::consts::ids::*;
use crate::gui::with_article;
use notan::prelude::*;
use std::collections::HashMap;

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
    let names = ecs.read_storage::<Name>();
    let mut rng = ecs.write_resource::<RandomNumberGenerator>();

    let mut result = RunState::AwaitingInput;
    let mut door_pos: Option<Point> = None;

    for (_entity, _player, pos, attributes) in (
        &entities,
        &mut players,
        &mut positions,
        &attributes,
    ).join() {
        let delta_x = i;
        let delta_y = j;

        if
            !(
                pos.x + delta_x < 0 ||
                pos.x + delta_x > map.width - 1 ||
                pos.y + delta_y < 0 ||
                pos.y + delta_y > map.height - 1
            )
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
                        let renderables = ecs.read_storage::<Renderable>();
                        if multiple_tile_content {
                            if let Some(name) = names.get(potential_target) {
                                gamelog::Logger
                                    ::new()
                                    .append("The")
                                    .colour(renderable_colour_ecs(ecs, potential_target))
                                    .append(&name.name)
                                    .colour(WHITE)
                                    .append("is blocked.")
                                    .log();
                            }
                        } else if rng.roll_dice(1, 6) + attributes.strength.modifier() < 2 {
                            if let Some(name) = names.get(potential_target) {
                                gamelog::Logger
                                    ::new()
                                    .append("The")
                                    .colour(renderable_colour_ecs(ecs, potential_target))
                                    .append(&name.name)
                                    .colour(WHITE)
                                    .append("resists!")
                                    .log();
                            }
                        } else {
                            door.open = false;
                            if door.blocks_vis {
                                blocks_visibility
                                    .insert(potential_target, BlocksVisibility {})
                                    .expect("Unable to insert BlocksVisibility.");
                            }
                            if door.blocks_move {
                                blocks_movement
                                    .insert(potential_target, BlocksTile {})
                                    .expect("Unable to insert BlocksTile.");
                            }
                            if let Some(name) = names.get(potential_target) {
                                gamelog::Logger
                                    ::new()
                                    .append("You close the")
                                    .colour(renderable_colour_ecs(ecs, potential_target))
                                    .append_n(&name.name)
                                    .colour(WHITE)
                                    .period()
                                    .log();
                            }
                            //Re-get renderables as mutable
                            std::mem::drop(renderables);
                            let mut renderables = ecs.write_storage::<Renderable>();
                            let render_data = renderables.get_mut(potential_target).unwrap();
                            render_data.swap();
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
    let names = ecs.read_storage::<Name>();
    let mut rng = ecs.write_resource::<RandomNumberGenerator>();

    let mut result = RunState::AwaitingInput;
    let mut door_pos: Option<Point> = None;

    for (_entity, _player, pos, attributes) in (
        &entities,
        &mut players,
        &mut positions,
        &attributes,
    ).join() {
        let delta_x = i;
        let delta_y = j;

        if
            !(
                pos.x + delta_x < 0 ||
                pos.x + delta_x > map.width - 1 ||
                pos.y + delta_y < 0 ||
                pos.y + delta_y > map.height - 1
            )
        {
            let destination_idx = map.xy_idx(pos.x + delta_x, pos.y + delta_y);

            if !crate::spatial::has_tile_content(destination_idx) {
                gamelog::Logger::new().append("You see no door there.").log();
            }
            crate::spatial::for_each_tile_content(destination_idx, |potential_target| {
                let door = doors.get_mut(potential_target);
                if let Some(door) = door {
                    if door.open == false {
                        let renderables = ecs.read_storage::<Renderable>();
                        if rng.roll_dice(1, 6) + attributes.strength.modifier() < 2 {
                            if let Some(name) = names.get(potential_target) {
                                gamelog::Logger
                                    ::new()
                                    .append("The")
                                    .colour(renderable_colour_ecs(ecs, potential_target))
                                    .append(&name.name)
                                    .colour(WHITE)
                                    .append("resists!")
                                    .log();
                            }
                        } else {
                            door.open = true;
                            blocks_visibility.remove(potential_target);
                            blocks_movement.remove(potential_target);
                            if let Some(name) = names.get(potential_target) {
                                gamelog::Logger
                                    ::new()
                                    .append("You open the")
                                    .colour(renderable_colour_ecs(ecs, potential_target))
                                    .append_n(&name.name)
                                    .colour(WHITE)
                                    .period()
                                    .log();
                            }
                            std::mem::drop(renderables);
                            let mut renderables = ecs.write_storage::<Renderable>();
                            let render_data = renderables.get_mut(potential_target).unwrap();
                            render_data.swap();
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

        for (entity, _player, pos, attributes) in (
            &entities,
            &mut players,
            &mut positions,
            &attributes,
        ).join() {
            let delta_x = i;
            let delta_y = j;

            if
                !(
                    pos.x + delta_x < 0 ||
                    pos.x + delta_x > map.width - 1 ||
                    pos.y + delta_y < 0 ||
                    pos.y + delta_y > map.height - 1
                )
            {
                let destination_idx = map.xy_idx(pos.x + delta_x, pos.y + delta_y);

                if !crate::spatial::has_tile_content(destination_idx) {
                    if rng.roll_dice(1, 20) == 20 {
                        add_effect(
                            None,
                            EffectType::Damage { amount: 1, damage_type: DamageType::Physical },
                            Targets::Entity { target: entity }
                        );
                        gamelog::Logger
                            ::new()
                            .append("Ouch! You kick the open air, and pull something.")
                            .log();
                        break;
                    } else {
                        // If there's nothing at all, just kick the air and waste a turn.
                        gamelog::Logger::new().append("You kick the open air.").log();
                        break;
                    }
                } else {
                    let mut last_non_door_target: Option<Entity> = None;
                    let mut target_name = "thing";
                    let mut colour = WHITE;
                    crate::spatial::for_each_tile_content_with_bool(
                        destination_idx,
                        |potential_target| {
                            if let Some(name) = names.get(potential_target) {
                                target_name = &name.name;
                            }
                            let items = ecs.read_storage::<Item>();
                            colour = if let Some(_) = items.get(potential_target) {
                                item_colour_ecs(ecs, potential_target)
                            } else {
                                renderable_colour_ecs(ecs, potential_target)
                            };

                            // If it's a door,
                            let door = doors.get_mut(potential_target);
                            if let Some(door) = door {
                                // If the door is closed,
                                if door.open == false {
                                    add_effect(
                                        None,
                                        EffectType::Particle {
                                            glyph: to_cp437('‼'),
                                            fg: RGB::named(CHOCOLATE),
                                            bg: RGB::named(BLACK),
                                            lifespan: 150.0,
                                            delay: 0.0,
                                        },
                                        Targets::Entity { target: potential_target }
                                    );
                                    // ~33% chance of breaking it down + str
                                    if rng.roll_dice(1, 10) + attributes.strength.modifier() > 6 {
                                        gamelog::Logger
                                            ::new()
                                            .append("As you kick the")
                                            .colour(colour)
                                            .append_n(obfuscate_name_ecs(ecs, potential_target).0)
                                            .colour(WHITE)
                                            .append(", it crashes open!")
                                            .log();
                                        something_was_destroyed = Some(potential_target);
                                        destroyed_pos = Some(
                                            Point::new(pos.x + delta_x, pos.y + delta_y)
                                        );
                                        gamelog::record_event(EVENT::BrokeDoor(1));
                                        return false;
                                        // 66% chance of just kicking it.
                                    } else {
                                        gamelog::Logger
                                            ::new()
                                            .append("You kick the")
                                            .colour(colour)
                                            .append_n(obfuscate_name_ecs(ecs, potential_target).0)
                                            .colour(WHITE)
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
                        }
                    );
                    if let Some(e) = last_non_door_target {
                        gamelog::Logger
                            ::new()
                            .append("You kick the")
                            .colour(colour)
                            .append_n(obfuscate_name_ecs(ecs, e).0)
                            .colour(WHITE)
                            .period()
                            .log();
                        add_effect(
                            None,
                            EffectType::Particle {
                                glyph: to_cp437('‼'),
                                fg: RGB::named(CHOCOLATE),
                                bg: RGB::named(BLACK),
                                lifespan: 150.0,
                                delay: 0.0,
                            },
                            Targets::Entity { target: e }
                        );
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

    gamelog::record_event(EVENT::KickedSomething(1));
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

    for (entity, _player, pos, viewshed) in (
        &entities,
        &mut players,
        &mut positions,
        &mut viewsheds,
    ).join() {
        if
            pos.x + delta_x < 0 ||
            pos.x + delta_x > map.width - 1 ||
            pos.y + delta_y < 0 ||
            pos.y + delta_y > map.height - 1
        {
            return RunState::AwaitingInput;
        }
        let destination_idx = map.xy_idx(pos.x + delta_x, pos.y + delta_y);

        result = crate::spatial::for_each_tile_content_with_runstate(
            destination_idx,
            |potential_target| {
                let mut hostile = true;
                if pools.get(potential_target).is_some() {
                    // We get the reaction of the target to this entity --
                    // i.e. in reverse to usual. We want to know if the target
                    //      is hostile to us. If it isn't, we can swap places.
                    let result = crate::raws::get_reactions(
                        potential_target,
                        entity,
                        &factions,
                        &ancestries,
                        &crate::raws::RAWS.lock().unwrap()
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
                    if door.open == false && door.blocks_move {
                        if let Some(name) = names.get(potential_target) {
                            let colour = if
                                let Some(_) = ecs.read_storage::<Item>().get(potential_target)
                            {
                                item_colour_ecs(ecs, potential_target)
                            } else {
                                renderable_colour_ecs(ecs, potential_target)
                            };
                            gamelog::Logger
                                ::new()
                                .append("The")
                                .colour(colour)
                                .append(&name.name)
                                .colour(WHITE)
                                .append("is in your way.")
                                .log();
                        }
                        return Some(RunState::AwaitingInput);
                    }
                }
                return None;
            }
        );

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
            let mut seen_items: Vec<(String, (u8, u8, u8))> = Vec::new();
            let mut some = false;
            crate::spatial::for_each_tile_content(destination_idx, |entity| {
                if !hidden.get(entity).is_some() && names.get(entity).is_some() {
                    let item_name = obfuscate_name_ecs(ecs, entity).0;
                    let item_colour = item_colour_ecs(ecs, entity);
                    seen_items.push((item_name, item_colour));
                    some = true;
                }
            });
            match map.tiles[destination_idx] {
                TileType::ToLocal(id) => {
                    let name = get_local_desc(id);
                    let colour = rgb_to_u8(get_local_col(id));
                    gamelog::Logger
                        ::new()
                        .append("You see")
                        .colour(colour)
                        .append_n(&name)
                        .colour(WHITE)
                        .period()
                        .log();
                }
                _ => {}
            }
            // If some names were found, append. Logger = logger is necessary
            // makes logger called a mutable self. It's not the most efficient
            // but it happens infrequently enough (once per player turn at most)
            // that it shouldn't matter.
            if some {
                let mut logger = gamelog::Logger::new().append("You see");
                for i in 0..seen_items.len() {
                    if i > 0 && i < seen_items.len() {
                        logger = logger.append(", ");
                    }
                    logger = logger
                        .colour(seen_items[i].1)
                        .append_n(with_article(&seen_items[i].0))
                        .colour(WHITE);
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
            if map.tiles[new_idx] == TileType::ToOvermap(map.id) {
                return RunState::GoToLevel(ID_OVERMAP, TileType::ToLocal(map.id));
            }
            return RunState::Ticking;
        }
    }

    for m in swap_entities.iter() {
        if let Some(name) = names.get(m.0) {
            gamelog::Logger
                ::new()
                .append("You swap places with the")
                .colour(renderable_colour_ecs(ecs, m.0))
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
            let mut assignkey = ecs.write_storage::<WantsToAssignKey>();
            let mut pickup = ecs.write_storage::<WantsToPickupItem>();
            assignkey.insert(item, WantsToAssignKey {}).expect("Unable to insert WantsToAssignKey");
            pickup
                .insert(*player_entity, WantsToPickupItem { collected_by: *player_entity, item })
                .expect("Unable to insert want to pickup item.");
            return RunState::Ticking;
        }
    }
}

fn try_descend(ecs: &mut World) -> RunState {
    let dest = try_change_level(ecs, false);
    let curr_map_id = ecs.fetch::<Map>().id;
    return match dest {
        Destination::None => RunState::AwaitingInput,
        Destination::NextLevel => RunState::GoToLevel(curr_map_id + 1, TileType::UpStair),
        Destination::PreviousLevel => RunState::GoToLevel(curr_map_id - 1, TileType::DownStair),
        Destination::ToLocal(id) => RunState::GoToLevel(ID_OVERMAP, TileType::ToLocal(id)),
        Destination::ToOvermap(id) => RunState::GoToLevel(id, TileType::ToOvermap(id)),
    };
}
fn try_ascend(ecs: &mut World) -> RunState {
    let dest = try_change_level(ecs, true);
    let curr_map_id = ecs.fetch::<Map>().id;
    return match dest {
        Destination::None => RunState::AwaitingInput,
        Destination::NextLevel => RunState::GoToLevel(curr_map_id + 1, TileType::UpStair),
        Destination::PreviousLevel => RunState::GoToLevel(curr_map_id - 1, TileType::DownStair),
        Destination::ToLocal(id) => RunState::GoToLevel(ID_OVERMAP, TileType::ToLocal(id)),
        Destination::ToOvermap(id) => RunState::GoToLevel(id, TileType::ToOvermap(id)),
    };
}

pub fn player_input(gs: &mut State, ctx: &mut App, on_overmap: bool) -> RunState {
    let key = &ctx.keyboard;
    // Movement
    for keycode in key.pressed.iter() {
        match *keycode {
            KeyCode::Numpad1 | KeyCode::B => {
                return try_move_player(-1, 1, &mut gs.ecs);
            }
            KeyCode::Numpad2 | KeyCode::Down | KeyCode::J => {
                return try_move_player(0, 1, &mut gs.ecs);
            }
            KeyCode::Numpad3 | KeyCode::N => {
                return try_move_player(1, 1, &mut gs.ecs);
            }
            KeyCode::Numpad4 | KeyCode::Left | KeyCode::H => {
                return try_move_player(-1, 0, &mut gs.ecs);
            }
            KeyCode::Numpad6 | KeyCode::Right | KeyCode::L => {
                return try_move_player(1, 0, &mut gs.ecs);
            }
            KeyCode::Numpad7 | KeyCode::Y => {
                return try_move_player(-1, -1, &mut gs.ecs);
            }
            KeyCode::Numpad8 | KeyCode::Up | KeyCode::K => {
                return try_move_player(0, -1, &mut gs.ecs);
            }
            KeyCode::Numpad9 | KeyCode::U => {
                return try_move_player(1, -1, &mut gs.ecs);
            }
            KeyCode::Period => {
                if key.shift() {
                    return try_descend(&mut gs.ecs);
                }
                return skip_turn(&mut gs.ecs);
            }
            KeyCode::Comma => {
                if key.shift() {
                    return try_ascend(&mut gs.ecs);
                }
            }
            KeyCode::Slash => {
                if key.shift() {
                    return RunState::HelpScreen;
                }
            }
            KeyCode::C => {
                if !on_overmap {
                    return RunState::ActionWithDirection { function: try_door };
                }
            }
            KeyCode::O => {
                if !on_overmap {
                    return RunState::ActionWithDirection { function: open };
                }
            }
            KeyCode::F => {
                if !on_overmap {
                    return RunState::ActionWithDirection { function: kick };
                }
            }
            KeyCode::G => {
                return get_item(&mut gs.ecs);
            }
            KeyCode::I => {
                return RunState::ShowInventory;
            }
            KeyCode::D => {
                return RunState::ShowDropItem;
            }
            KeyCode::R => {
                return RunState::ShowRemoveItem;
            }
            KeyCode::Minus => {
                return RunState::ShowCheatMenu;
            }
            KeyCode::Escape => {
                return RunState::SaveGame;
            }
            KeyCode::X => {
                let bounds = get_screen_bounds(&gs.ecs, false);
                let ppos = gs.ecs.fetch::<Point>();
                let (x, y) = (
                    ppos.x + bounds.x_offset - bounds.min_x,
                    ppos.y + bounds.y_offset - bounds.min_y,
                );
                return RunState::Farlook { x, y };
            }
            _ => {}
        }
    }
    return RunState::AwaitingInput;
}

fn try_change_level(ecs: &mut World, backtracking: bool) -> Destination {
    let player_pos = ecs.fetch::<Point>();
    let map = ecs.fetch::<Map>();
    let player_idx = map.xy_idx(player_pos.x, player_pos.y);
    let this_tile = map.tiles[player_idx];
    let mut blocked = false;
    crate::spatial::for_each_tile_content(player_idx, |potential| {
        if let Some(is_door) = ecs.read_storage::<Door>().get(potential) {
            if is_door.open == false {
                blocked = true;
                gamelog::Logger::new().append("The way is blocked.").log();
            }
        }
    });
    if blocked {
        return Destination::None;
    }
    return get_dest(this_tile, backtracking);
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
                &crate::raws::RAWS.lock().unwrap()
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
            HungerState::Hungry => {
                can_heal = false;
            }
            HungerState::Weak => {
                can_heal = false;
            }
            HungerState::Fainting => {
                can_heal = false;
            }
            _ => {}
        }
    }

    if can_heal {
        let mut health_components = ecs.write_storage::<Pools>();
        let pools = health_components.get_mut(*player_entity).unwrap();
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();
        let roll = rng.roll_dice(1, 6);
        if roll == 6 && pools.hit_points.current < pools.hit_points.max {
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

    let path = a_star_search(map.xy_idx(player_pos.x, player_pos.y), unexplored_tile.0, &*map);
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
