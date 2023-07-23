use super::{
    gamelog, rex_assets::RexAssets, CombatStats, Equipped, Hidden, HungerClock, HungerState, InBackpack, Map, Name,
    Player, Point, Position, RunState, State, Viewshed,
};
use rltk::{Rltk, VirtualKeyCode, RGB};
use specs::prelude::*;
use std::collections::BTreeMap;

pub fn draw_ui(ecs: &World, ctx: &mut Rltk) {
    ctx.draw_hollow_box_double(0, 43, 79, 7, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK));

    // Render stats
    let combat_stats = ecs.read_storage::<CombatStats>();
    let players = ecs.read_storage::<Player>();
    let hunger = ecs.read_storage::<HungerClock>();
    for (_player, stats, hunger) in (&players, &combat_stats, &hunger).join() {
        let health = format!(" HP {}/{} ", stats.hp, stats.max_hp);
        ctx.print_color_right(36, 43, RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK), &health);
        ctx.draw_bar_horizontal(38, 43, 34, stats.hp, stats.max_hp, RGB::named(rltk::RED), RGB::named(rltk::BLACK));
        match hunger.state {
            HungerState::Satiated => {
                ctx.print_color_right(72, 42, RGB::named(rltk::GREEN), RGB::named(rltk::BLACK), "Satiated")
            }
            HungerState::Normal => {}
            HungerState::Hungry => {
                ctx.print_color_right(72, 42, RGB::named(rltk::BROWN1), RGB::named(rltk::BLACK), "Hungry")
            }
            HungerState::Weak => {
                ctx.print_color_right(72, 42, RGB::named(rltk::ORANGE), RGB::named(rltk::BLACK), "Weak")
            }
            HungerState::Fainting => {
                ctx.print_color_right(72, 42, RGB::named(rltk::RED), RGB::named(rltk::BLACK), "Fainting")
            }
        }
    }

    // Render the message log at [1, 46], descending, with 6 lines.
    gamelog::print_log(&mut rltk::BACKEND_INTERNAL.lock().consoles[0].console, Point::new(1, 44), true, 6);

    // Render depth
    let map = ecs.fetch::<Map>();
    let depth = format!(" D{} ", map.depth);
    ctx.print_color_right(78, 43, RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK), &depth);

    // Render turn
    ctx.print_color_right(
        78,
        50,
        RGB::named(rltk::YELLOW),
        RGB::named(rltk::BLACK),
        &format!(" T{} ", crate::gamelog::get_event_count("Turn")),
    );

    // Render mouse cursor
    let mouse_pos = ctx.mouse_pos();
    ctx.set_bg(mouse_pos.0, mouse_pos.1, RGB::named(rltk::YELLOW));

    draw_tooltips(ecs, ctx);
}

fn draw_tooltips(ecs: &World, ctx: &mut Rltk) {
    let map = ecs.fetch::<Map>();
    let names = ecs.read_storage::<Name>();
    let positions = ecs.read_storage::<Position>();
    let hidden = ecs.read_storage::<Hidden>();

    let mouse_pos = ctx.mouse_pos();
    if mouse_pos.0 >= map.width || mouse_pos.1 >= map.height {
        return;
    }
    let mut tooltip: Vec<String> = Vec::new();
    for (name, position, _hidden) in (&names, &positions, !&hidden).join() {
        let idx = map.xy_idx(position.x, position.y);
        if position.x == mouse_pos.0 && position.y == mouse_pos.1 && map.visible_tiles[idx] {
            tooltip.push(name.name.to_string());
        }
    }

    if !tooltip.is_empty() {
        if mouse_pos.0 > 40 {
            let arrow_pos = Point::new(mouse_pos.0 - 2, mouse_pos.1);
            let left_x = mouse_pos.0 - 3;
            let mut y = mouse_pos.1;
            for s in tooltip.iter() {
                for i in 0..2 {
                    ctx.print_color(
                        arrow_pos.x - i,
                        y,
                        RGB::named(rltk::WHITE),
                        RGB::named(rltk::GREY),
                        &" ".to_string(),
                    );
                }
                ctx.print_color_right(left_x, y, RGB::named(rltk::WHITE), RGB::named(rltk::GREY), s);
                y += 1;
            }
            ctx.print_color(
                arrow_pos.x,
                arrow_pos.y,
                RGB::named(rltk::WHITE),
                RGB::named(rltk::GREY),
                &"->".to_string(),
            );
        } else {
            let arrow_pos = Point::new(mouse_pos.0 + 1, mouse_pos.1);
            let left_x = mouse_pos.0 + 3;
            let mut y = mouse_pos.1;
            for s in tooltip.iter() {
                for i in 0..2 {
                    ctx.print_color(
                        arrow_pos.x + 1 + i,
                        y,
                        RGB::named(rltk::WHITE),
                        RGB::named(rltk::GREY),
                        &" ".to_string(),
                    );
                }
                ctx.print_color(left_x + 1, y, RGB::named(rltk::WHITE), RGB::named(rltk::DARKGREY), s);
                y += 1;
            }
            ctx.print_color(
                arrow_pos.x,
                arrow_pos.y,
                RGB::named(rltk::WHITE),
                RGB::named(rltk::GREY),
                &"<-".to_string(),
            );
        }
    }
}

#[derive(PartialEq, Copy, Clone)]
pub enum ItemMenuResult {
    Cancel,
    NoResponse,
    Selected,
}

pub fn print_options(inventory: BTreeMap<(String, String), i32>, mut y: i32, ctx: &mut Rltk) {
    let mut j = 0;
    for (name, item_count) in &inventory {
        // Print the character required to access this item. i.e. (a)
        ctx.set(17, y, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK), rltk::to_cp437('('));
        ctx.set(18, y, RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK), 97 + j as rltk::FontCharType);
        ctx.set(19, y, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK), rltk::to_cp437(')'));

        let mut x = 21;
        if item_count > &1 {
            // If more than one, print the number and pluralise
            // i.e. (a) 3 daggers
            ctx.print(x, y, item_count);
            x += 2;
            ctx.print(x, y, name.1.to_string());
        } else {
            if ['a', 'e', 'i', 'o', 'u'].iter().any(|&v| name.0.starts_with(v)) {
                // If one and starts with a vowel, print 'an'
                // i.e. (a) an apple
                ctx.print(x, y, "an");
                x += 3;
            } else {
                // If one and not a vowel, print 'a'
                // i.e. (a) a dagger
                ctx.print(x, y, "a");
                x += 2;
            }
            ctx.print(x, y, name.0.to_string());
        }
        y += 1;
        j += 1;
    }
}

pub fn show_inventory(gs: &mut State, ctx: &mut Rltk) -> (ItemMenuResult, Option<Entity>) {
    let player_entity = gs.ecs.fetch::<Entity>();
    let names = gs.ecs.read_storage::<Name>();
    let backpack = gs.ecs.read_storage::<InBackpack>();
    let entities = gs.ecs.entities();

    // FIXME: This is unwieldy. Having a separate data structure for (name, id) and (name, count) is not good.
    // But it works, and this might get cut anyway as I get further along in the design, so leaving as is atm.
    let mut inventory_ids: BTreeMap<String, Entity> = BTreeMap::new();
    let mut player_inventory: BTreeMap<(String, String), i32> = BTreeMap::new();
    for (entity, _pack, name) in (&entities, &backpack, &names).join().filter(|item| item.1.owner == *player_entity) {
        player_inventory
            .entry((name.name.to_string(), name.plural.to_string()))
            .and_modify(|count| *count += 1)
            .or_insert(1);
        inventory_ids.entry(name.name.to_string()).or_insert(entity);
    }

    let count = player_inventory.len();
    let y = (25 - (count / 2)) as i32;
    ctx.draw_box(15, y - 2, 45, (count + 3) as i32, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK));
    ctx.print_color(18, y - 2, RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK), "Inventory");
    ctx.print_color(18, y + count as i32 + 1, RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK), "ESC to cancel");

    print_options(player_inventory, y, ctx);

    match ctx.key {
        None => (ItemMenuResult::NoResponse, None),
        Some(key) => match key {
            VirtualKeyCode::Escape => (ItemMenuResult::Cancel, None),
            _ => {
                let selection = rltk::letter_to_option(key);
                if selection > -1 && selection < count as i32 {
                    return (ItemMenuResult::Selected, Some(*inventory_ids.iter().nth(selection as usize).unwrap().1));
                }
                (ItemMenuResult::NoResponse, None)
            }
        },
    }
}

pub fn drop_item_menu(gs: &mut State, ctx: &mut Rltk) -> (ItemMenuResult, Option<Entity>) {
    let player_entity = gs.ecs.fetch::<Entity>();
    let names = gs.ecs.read_storage::<Name>();
    let backpack = gs.ecs.read_storage::<InBackpack>();
    let entities = gs.ecs.entities();

    let mut inventory_ids: BTreeMap<String, Entity> = BTreeMap::new();
    let mut player_inventory: BTreeMap<(String, String), i32> = BTreeMap::new();
    for (entity, _pack, name) in (&entities, &backpack, &names).join().filter(|item| item.1.owner == *player_entity) {
        player_inventory
            .entry((name.name.to_string(), name.plural.to_string()))
            .and_modify(|count| *count += 1)
            .or_insert(1);
        inventory_ids.entry(name.name.to_string()).or_insert(entity);
    }

    let count = player_inventory.len();
    let y = (25 - (count / 2)) as i32;
    ctx.draw_box(15, y - 2, 45, (count + 3) as i32, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK));
    ctx.print_color(18, y - 2, RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK), "Drop what?");
    ctx.print_color(18, y + count as i32 + 1, RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK), "ESC to cancel");

    print_options(player_inventory, y, ctx);

    match ctx.key {
        None => (ItemMenuResult::NoResponse, None),
        Some(key) => match key {
            VirtualKeyCode::Escape => (ItemMenuResult::Cancel, None),
            _ => {
                let selection = rltk::letter_to_option(key);
                if selection > -1 && selection < count as i32 {
                    return (ItemMenuResult::Selected, Some(*inventory_ids.iter().nth(selection as usize).unwrap().1));
                }
                (ItemMenuResult::NoResponse, None)
            }
        },
    }
}

pub fn remove_item_menu(gs: &mut State, ctx: &mut Rltk) -> (ItemMenuResult, Option<Entity>) {
    let player_entity = gs.ecs.fetch::<Entity>();
    let names = gs.ecs.read_storage::<Name>();
    let backpack = gs.ecs.read_storage::<Equipped>();
    let entities = gs.ecs.entities();

    let inventory = (&backpack, &names).join().filter(|item| item.0.owner == *player_entity);
    let count = inventory.count();

    let mut y = (25 - (count / 2)) as i32;
    ctx.draw_box(15, y - 2, 31, (count + 3) as i32, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK));
    ctx.print_color(18, y - 2, RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK), "Remove what?");
    ctx.print_color(18, y + count as i32 + 1, RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK), "ESC to cancel");

    let mut equippable: Vec<Entity> = Vec::new();
    let mut j = 0;
    for (entity, _pack, name) in (&entities, &backpack, &names).join().filter(|item| item.1.owner == *player_entity) {
        ctx.set(17, y, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK), rltk::to_cp437('('));
        ctx.set(18, y, RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK), 97 + j as rltk::FontCharType);
        ctx.set(19, y, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK), rltk::to_cp437(')'));

        ctx.print(21, y, &name.name.to_string());
        equippable.push(entity);
        y += 1;
        j += 1;
    }

    match ctx.key {
        None => (ItemMenuResult::NoResponse, None),
        Some(key) => match key {
            VirtualKeyCode::Escape => (ItemMenuResult::Cancel, None),
            _ => {
                let selection = rltk::letter_to_option(key);
                if selection > -1 && selection < count as i32 {
                    return (ItemMenuResult::Selected, Some(equippable[selection as usize]));
                }
                (ItemMenuResult::NoResponse, None)
            }
        },
    }
}

pub fn ranged_target(gs: &mut State, ctx: &mut Rltk, range: i32, aoe: i32) -> (ItemMenuResult, Option<Point>) {
    let player_entity = gs.ecs.fetch::<Entity>();
    let player_pos = gs.ecs.fetch::<Point>();
    let viewsheds = gs.ecs.read_storage::<Viewshed>();

    ctx.print_color(5, 0, RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK), "select target");

    // Highlight available cells
    let mut available_cells = Vec::new();
    let visible = viewsheds.get(*player_entity);
    if let Some(visible) = visible {
        // We have a viewshed
        for idx in visible.visible_tiles.iter() {
            let distance = rltk::DistanceAlg::Pythagoras.distance2d(*player_pos, *idx);
            if distance <= range as f32 {
                ctx.set_bg(idx.x, idx.y, RGB::named(rltk::BLUE));
                available_cells.push(idx);
            }
        }
    } else {
        return (ItemMenuResult::Cancel, None);
    }

    // Draw mouse cursor
    let mouse_pos = ctx.mouse_pos();
    let map = gs.ecs.fetch::<Map>();
    let mut valid_target = false;
    for idx in available_cells.iter() {
        if idx.x == mouse_pos.0 && idx.y == mouse_pos.1 {
            valid_target = true;
        }
    }
    if valid_target {
        if aoe > 0 {
            let mut blast_tiles = rltk::field_of_view(Point::new(mouse_pos.0, mouse_pos.1), aoe, &*map);
            blast_tiles.retain(|p| p.x > 0 && p.x < map.width - 1 && p.y > 0 && p.y < map.height - 1);
            for tile in blast_tiles.iter() {
                ctx.set_bg(tile.x, tile.y, RGB::named(rltk::DARKCYAN));
            }
        }
        ctx.set_bg(mouse_pos.0, mouse_pos.1, RGB::named(rltk::CYAN));
        if ctx.left_click {
            return (ItemMenuResult::Selected, Some(Point::new(mouse_pos.0, mouse_pos.1)));
        }
    } else {
        ctx.set_bg(mouse_pos.0, mouse_pos.1, RGB::named(rltk::RED));
        if ctx.left_click {
            return (ItemMenuResult::Cancel, None);
        }
    }

    (ItemMenuResult::NoResponse, None)
}

#[derive(PartialEq, Copy, Clone)]
pub enum MainMenuSelection {
    NewGame,
    LoadGame,
    Quit,
}

#[derive(PartialEq, Copy, Clone)]
pub enum MainMenuResult {
    NoSelection { selected: MainMenuSelection },
    Selected { selected: MainMenuSelection },
}

pub fn main_menu(gs: &mut State, ctx: &mut Rltk) -> MainMenuResult {
    let save_exists = super::saveload_system::does_save_exist();
    let runstate = gs.ecs.fetch::<RunState>();
    let assets = gs.ecs.fetch::<RexAssets>();

    ctx.render_xp_sprite(&assets.menu, 0, 0);

    ctx.print_color(50, 26, RGB::named(rltk::GREEN), RGB::from_f32(0.11, 0.11, 0.11), "RUST-RL");

    if let RunState::MainMenu { menu_selection: selection } = *runstate {
        let mut y = 29;
        if selection == MainMenuSelection::NewGame {
            ctx.print_color(47, y, RGB::named(rltk::YELLOW), RGB::from_f32(0.11, 0.11, 0.11), "[");
            ctx.print_color(49, y, RGB::named(rltk::GREEN), RGB::from_f32(0.11, 0.11, 0.11), "new game");
            ctx.print_color(58, y, RGB::named(rltk::YELLOW), RGB::from_f32(0.11, 0.11, 0.11), "]");
        } else {
            ctx.print_color(49, y, RGB::named(rltk::WHITE), RGB::from_f32(0.11, 0.11, 0.11), "new game");
        }
        y += 2;
        if save_exists {
            if selection == MainMenuSelection::LoadGame {
                ctx.print_color(46, y, RGB::named(rltk::YELLOW), RGB::from_f32(0.11, 0.11, 0.11), "[");
                ctx.print_color(48, y, RGB::named(rltk::GREEN), RGB::from_f32(0.11, 0.11, 0.11), "load game");
                ctx.print_color(58, y, RGB::named(rltk::YELLOW), RGB::from_f32(0.11, 0.11, 0.11), "]");
            } else {
                ctx.print_color(48, y, RGB::named(rltk::WHITE), RGB::from_f32(0.11, 0.11, 0.11), "load game");
            }
            y += 2;
        }
        if selection == MainMenuSelection::Quit {
            ctx.print_color(47, y, RGB::named(rltk::YELLOW), RGB::from_f32(0.11, 0.11, 0.11), "[");
            ctx.print_color(49, y, RGB::named(rltk::GREEN), RGB::from_f32(0.11, 0.11, 0.11), "goodbye!");
            ctx.print_color(58, y, RGB::named(rltk::YELLOW), RGB::from_f32(0.11, 0.11, 0.11), "]");
        } else {
            ctx.print_color(53, y, RGB::named(rltk::WHITE), RGB::from_f32(0.11, 0.11, 0.11), "quit");
        }

        match ctx.key {
            None => return MainMenuResult::NoSelection { selected: selection },
            Some(key) => match key {
                VirtualKeyCode::Escape | VirtualKeyCode::C => {
                    return MainMenuResult::NoSelection { selected: MainMenuSelection::Quit }
                }
                VirtualKeyCode::N => return MainMenuResult::NoSelection { selected: MainMenuSelection::NewGame },
                VirtualKeyCode::L => return MainMenuResult::NoSelection { selected: MainMenuSelection::LoadGame },
                VirtualKeyCode::Up | VirtualKeyCode::Numpad8 | VirtualKeyCode::K => {
                    let mut new_selection;
                    match selection {
                        MainMenuSelection::NewGame => new_selection = MainMenuSelection::Quit,
                        MainMenuSelection::LoadGame => new_selection = MainMenuSelection::NewGame,
                        MainMenuSelection::Quit => new_selection = MainMenuSelection::LoadGame,
                    }
                    if new_selection == MainMenuSelection::LoadGame && !save_exists {
                        new_selection = MainMenuSelection::NewGame;
                    }
                    return MainMenuResult::NoSelection { selected: new_selection };
                }
                VirtualKeyCode::Down | VirtualKeyCode::Numpad2 | VirtualKeyCode::J => {
                    let mut new_selection;
                    match selection {
                        MainMenuSelection::NewGame => new_selection = MainMenuSelection::LoadGame,
                        MainMenuSelection::LoadGame => new_selection = MainMenuSelection::Quit,
                        MainMenuSelection::Quit => new_selection = MainMenuSelection::NewGame,
                    }
                    if new_selection == MainMenuSelection::LoadGame && !save_exists {
                        new_selection = MainMenuSelection::Quit;
                    }
                    return MainMenuResult::NoSelection { selected: new_selection };
                }
                VirtualKeyCode::Return | VirtualKeyCode::NumpadEnter => {
                    return MainMenuResult::Selected { selected: selection }
                }
                _ => return MainMenuResult::NoSelection { selected: selection },
            },
        }
    }
    MainMenuResult::NoSelection { selected: MainMenuSelection::NewGame }
}

#[derive(PartialEq, Copy, Clone)]
pub enum GameOverResult {
    NoSelection,
    QuitToMenu,
}

pub fn game_over(ctx: &mut Rltk) -> GameOverResult {
    ctx.print_color_centered(15, RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK), "Your journey has ended!");
    ctx.print_color_centered(
        17,
        RGB::named(rltk::WHITE),
        RGB::named(rltk::BLACK),
        format!("You died after {} turns.", crate::gamelog::get_event_count("Turn")),
    );

    ctx.print_color_centered(
        19,
        RGB::named(rltk::MAGENTA),
        RGB::named(rltk::BLACK),
        "Press Escape to return to the menu.",
    );

    match ctx.key {
        None => GameOverResult::NoSelection,
        Some(key) => match key {
            VirtualKeyCode::Escape => GameOverResult::QuitToMenu,
            _ => GameOverResult::NoSelection,
        },
    }
}
