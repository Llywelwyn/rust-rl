use super::{
    gamelog::GameLog, gui, CombatStats, Item, Map, Player, Position, RunState, State, Viewshed, WantsToMelee,
    WantsToPickupItem, MAPHEIGHT, MAPWIDTH,
};
use rltk::{Point, Rltk, VirtualKeyCode};
use specs::prelude::*;
use std::cmp::{max, min};

pub fn try_move_player(delta_x: i32, delta_y: i32, ecs: &mut World) {
    let mut positions = ecs.write_storage::<Position>();
    let mut players = ecs.write_storage::<Player>();
    let mut viewsheds = ecs.write_storage::<Viewshed>();
    let combat_stats = ecs.read_storage::<CombatStats>();
    let map = ecs.fetch::<Map>();

    let entities = ecs.entities();
    let mut wants_to_melee = ecs.write_storage::<WantsToMelee>();

    for (entity, _player, pos, viewshed) in (&entities, &mut players, &mut positions, &mut viewsheds).join() {
        if pos.x + delta_x < 1
            || pos.x + delta_x > map.width - 1
            || pos.y + delta_y < 1
            || pos.y + delta_y > map.height - 1
        {
            return;
        }
        let destination_idx = map.xy_idx(pos.x + delta_x, pos.y + delta_y);

        for potential_target in map.tile_content[destination_idx].iter() {
            let target = combat_stats.get(*potential_target);
            if let Some(_target) = target {
                wants_to_melee.insert(entity, WantsToMelee { target: *potential_target }).expect("Add target failed.");
                return;
            }
        }

        if !map.blocked[destination_idx] {
            pos.x = min((MAPWIDTH as i32) - 1, max(0, pos.x + delta_x));
            pos.y = min((MAPHEIGHT as i32) - 1, max(0, pos.y + delta_y));
            viewshed.dirty = true;
            let mut ppos = ecs.write_resource::<Point>();
            ppos.x = pos.x;
            ppos.y = pos.y;
        }
    }
}

fn get_item(ecs: &mut World) {
    let player_pos = ecs.fetch::<Point>();
    let player_entity = ecs.fetch::<Entity>();
    let entities = ecs.entities();
    let items = ecs.read_storage::<Item>();
    let positions = ecs.read_storage::<Position>();
    let mut gamelog = ecs.fetch_mut::<GameLog>();

    let mut target_item: Option<Entity> = None;
    for (item_entity, _item, position) in (&entities, &items, &positions).join() {
        if position.x == player_pos.x && position.y == player_pos.y {
            target_item = Some(item_entity);
        }
    }

    match target_item {
        None => gamelog.entries.push("There is nothing to pick up.".to_string()),
        Some(item) => {
            let mut pickup = ecs.write_storage::<WantsToPickupItem>();
            pickup
                .insert(*player_entity, WantsToPickupItem { collected_by: *player_entity, item })
                .expect("Unable to insert want to pickup item.");
        }
    }
}

pub fn player_input(gs: &mut State, ctx: &mut Rltk) -> RunState {
    // Player movement
    match ctx.key {
        None => return RunState::AwaitingInput,
        Some(key) => match key {
            // Cardinals
            VirtualKeyCode::Left | VirtualKeyCode::Numpad4 | VirtualKeyCode::H => {
                try_move_player(-1, 0, &mut gs.ecs);
            }
            VirtualKeyCode::Right | VirtualKeyCode::Numpad6 | VirtualKeyCode::L => {
                try_move_player(1, 0, &mut gs.ecs);
            }
            VirtualKeyCode::Up | VirtualKeyCode::Numpad8 | VirtualKeyCode::K => {
                try_move_player(0, -1, &mut gs.ecs);
            }
            VirtualKeyCode::Down | VirtualKeyCode::Numpad2 | VirtualKeyCode::J => {
                try_move_player(0, 1, &mut gs.ecs);
            }
            // Diagonals
            VirtualKeyCode::Numpad9 | VirtualKeyCode::Y => try_move_player(1, -1, &mut gs.ecs),
            VirtualKeyCode::Numpad7 | VirtualKeyCode::U => try_move_player(-1, -1, &mut gs.ecs),
            VirtualKeyCode::Numpad3 | VirtualKeyCode::N => try_move_player(1, 1, &mut gs.ecs),
            VirtualKeyCode::Numpad1 | VirtualKeyCode::B => try_move_player(-1, 1, &mut gs.ecs),
            // Items
            VirtualKeyCode::G => get_item(&mut gs.ecs),
            VirtualKeyCode::I => return RunState::ShowInventory,
            VirtualKeyCode::D => return RunState::ShowDropItem,
            VirtualKeyCode::Escape => return RunState::MainMenu { menu_selection: gui::MainMenuSelection::NewGame },
            _ => {
                return RunState::AwaitingInput;
            }
        },
    }
    RunState::PlayerTurn
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
