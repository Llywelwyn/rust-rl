use super::{
    camera, gamelog, gamesystem, rex_assets::RexAssets, ArmourClassBonus, Attributes, Equipped, Hidden, HungerClock,
    HungerState, InBackpack, Map, Name, Player, Point, Pools, Position, Prop, Renderable, RunState, Skill, Skills,
    State, Viewshed,
};
use rltk::{Rltk, VirtualKeyCode, RGB};
use specs::prelude::*;
use std::collections::BTreeMap;
mod letter_to_option;
mod tooltip;

pub fn draw_lerping_bar(
    ctx: &mut Rltk,
    sx: i32,
    sy: i32,
    width: i32,
    n: i32,
    max: i32,
    full_colour: RGB,
    empty_colour: RGB,
) {
    let percent = n as f32 / max as f32;
    let fill_width = (percent * width as f32) as i32;
    let bg = empty_colour.lerp(full_colour, percent);
    let fg = RGB::named(rltk::BLACK);
    for x in 0..width {
        if x <= fill_width {
            ctx.print_color(sx + x, sy, fg, bg, " ");
        } else {
            ctx.print_color(sx + x, sy, RGB::named(rltk::BLACK), RGB::named(rltk::BLACK), " ");
        }
    }
    ctx.print(sx - 1, sy, "[");
    let health = format!("{}/{}", n, max);
    ctx.print_color(sx + 1, sy, fg, bg, health);
    ctx.print(sx + width, sy, "]");
}

pub fn draw_ui(ecs: &World, ctx: &mut Rltk) {
    ctx.draw_hollow_box(0, 0, 70, 8, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK)); // Log box
    ctx.draw_hollow_box(0, 9, 70, 42, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK)); // Camera box
    ctx.draw_hollow_box(0, 52, 70, 3, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK)); // Stats box
    ctx.draw_hollow_box(71, 0, 33, 55, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK)); // Side box

    // Render stats
    let pools = ecs.read_storage::<Pools>();
    let attributes = ecs.read_storage::<Attributes>();
    let players = ecs.read_storage::<Player>();
    let hunger = ecs.read_storage::<HungerClock>();
    let skills = ecs.read_storage::<Skills>();
    for (_player, stats, attributes, hunger, skills) in (&players, &pools, &attributes, &hunger, &skills).join() {
        // Draw hp/mana bars
        draw_lerping_bar(
            ctx,
            2,
            53,
            26,
            stats.hit_points.current,
            stats.hit_points.max,
            RGB::from_u8(0, 255, 0),
            RGB::from_u8(255, 0, 0),
        );
        draw_lerping_bar(
            ctx,
            2,
            54,
            26,
            stats.mana.current,
            stats.mana.max,
            RGB::named(rltk::BLUE),
            RGB::named(rltk::BLACK),
        );
        // Draw AC
        let skill_ac_bonus = gamesystem::skill_bonus(Skill::Defence, &*skills);
        let mut armour_ac_bonus = 0;
        let equipped = ecs.read_storage::<Equipped>();
        let ac = ecs.read_storage::<ArmourClassBonus>();
        let player_entity = ecs.fetch::<Entity>();
        for (wielded, ac) in (&equipped, &ac).join() {
            if wielded.owner == *player_entity {
                armour_ac_bonus += ac.amount;
            }
        }
        let armour_class = stats.bac - attributes.dexterity.bonus - skill_ac_bonus - armour_ac_bonus;
        ctx.print_color(30, 53, RGB::named(rltk::PINK), RGB::named(rltk::BLACK), "AC");
        ctx.print_color(32, 53, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK), armour_class);
        // Draw level
        ctx.print_color(
            30,
            54,
            RGB::named(rltk::WHITE),
            RGB::named(rltk::BLACK),
            format!("XP{}/{}", stats.level, stats.xp),
        );
        // Draw attributes
        let x = 38;
        ctx.print_color(x, 53, RGB::named(rltk::RED), RGB::named(rltk::BLACK), "STR");
        ctx.print_color(x + 3, 53, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK), attributes.strength.base);
        ctx.print_color(x + 7, 53, RGB::named(rltk::GREEN), RGB::named(rltk::BLACK), "DEX");
        ctx.print_color(x + 10, 53, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK), attributes.dexterity.base);
        ctx.print_color(x + 14, 53, RGB::named(rltk::ORANGE), RGB::named(rltk::BLACK), "CON");
        ctx.print_color(x + 17, 53, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK), attributes.constitution.base);
        ctx.print_color(x, 54, RGB::named(rltk::CYAN), RGB::named(rltk::BLACK), "INT");
        ctx.print_color(x + 3, 54, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK), attributes.intelligence.base);
        ctx.print_color(x + 7, 54, RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK), "WIS");
        ctx.print_color(x + 10, 54, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK), attributes.wisdom.base);
        ctx.print_color(x + 14, 54, RGB::named(rltk::PURPLE), RGB::named(rltk::BLACK), "CHA");
        ctx.print_color(x + 17, 54, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK), attributes.charisma.base);
        // Draw hunger
        match hunger.state {
            HungerState::Satiated => {
                ctx.print_color_right(70, 53, RGB::named(rltk::GREEN), RGB::named(rltk::BLACK), "Satiated")
            }
            HungerState::Normal => {}
            HungerState::Hungry => {
                ctx.print_color_right(70, 53, RGB::named(rltk::BROWN1), RGB::named(rltk::BLACK), "Hungry")
            }
            HungerState::Weak => {
                ctx.print_color_right(70, 53, RGB::named(rltk::ORANGE), RGB::named(rltk::BLACK), "Weak")
            }
            HungerState::Fainting => {
                ctx.print_color_right(70, 53, RGB::named(rltk::RED), RGB::named(rltk::BLACK), "Fainting")
            }
        }

        // Draw equipment
        let names = ecs.read_storage::<Name>();
        let mut equipment: Vec<String> = Vec::new();
        for (_equipped, name) in (&equipped, &names).join().filter(|item| item.0.owner == *player_entity) {
            equipment.push(format!("- {} (worn)", &name.name));
        }
        let mut y = 1;
        if !equipment.is_empty() {
            ctx.print_color(72, y, RGB::named(rltk::BLACK), RGB::named(rltk::WHITE), "Equipment");
            for item in equipment {
                y += 1;
                ctx.print_color(72, y, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK), item);
            }
            y += 2;
        }

        // Draw consumables
        ctx.print_color(72, y, RGB::named(rltk::BLACK), RGB::named(rltk::WHITE), "Backpack");
        y += 1;
        let (player_inventory, _inventory_ids) = get_player_inventory(&ecs);
        y = print_options(player_inventory, 72, y, ctx).0;

        // Draw entities seen on screen
        let viewsheds = ecs.read_storage::<Viewshed>();
        let renderables = ecs.read_storage::<Renderable>();
        let hidden = ecs.read_storage::<Hidden>();
        let props = ecs.read_storage::<Prop>();
        let map = ecs.fetch::<Map>();
        let viewshed = viewsheds.get(*player_entity).unwrap();
        let mut seen_entities: Vec<(String, RGB)> = Vec::new();
        for tile in viewshed.visible_tiles.iter() {
            let idx = map.xy_idx(tile.x, tile.y);
            for entity in map.tile_content[idx].iter() {
                let mut draw = true;
                let prop = props.get(*entity);
                if let Some(_) = prop {
                    draw = false;
                }
                let is_hidden = hidden.get(*entity);
                if let Some(_) = is_hidden {
                    draw = false;
                }
                if entity == &*player_entity {
                    draw = false;
                }
                let name = &names.get(*entity);
                if let Some(name) = name {
                    if draw {
                        let fg = renderables.get(*entity).unwrap().fg;
                        seen_entities.push((name.name.to_string(), fg));
                    }
                }
            }
        }
        seen_entities.sort_by(|a, b| b.0.cmp(&a.0));

        if !seen_entities.is_empty() {
            y += 1;
            ctx.print_color(72, y, RGB::named(rltk::BLACK), RGB::named(rltk::WHITE), "In View");
            for entity in seen_entities {
                y += 1;
                ctx.print_color(72, y, entity.1, RGB::named(rltk::BLACK), entity.0);
            }
        }
    }

    // Render the message log at [1, 7], ascending, with 7 lines and a max width of 68.
    gamelog::print_log(&mut rltk::BACKEND_INTERNAL.lock().consoles[0].console, Point::new(1, 7), false, 7, 68);

    // Render id
    let map = ecs.fetch::<Map>();
    let id = format!("D{}", map.id);
    ctx.print_color_right(70, 54, RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK), &id);

    // Render turn
    ctx.print_color_right(
        64,
        54,
        RGB::named(rltk::YELLOW),
        RGB::named(rltk::BLACK),
        &format!("T{}", crate::gamelog::get_event_count("turns")),
    );

    tooltip::draw_tooltips(ecs, ctx);
}

pub fn get_input_direction(
    ecs: &mut World,
    ctx: &mut Rltk,
    function: fn(i: i32, j: i32, ecs: &mut World) -> RunState,
) -> RunState {
    let (_, _, _, _, x_offset, y_offset) = camera::get_screen_bounds(ecs, ctx);

    ctx.print_color(
        1 + x_offset,
        1 + y_offset,
        RGB::named(rltk::WHITE),
        RGB::named(rltk::BLACK),
        "In what direction? [0-9]/[YUHJKLBN]",
    );
    match ctx.key {
        None => return RunState::ActionWithDirection { function },
        Some(key) => match key {
            VirtualKeyCode::Escape => return RunState::AwaitingInput,
            // Cardinals
            VirtualKeyCode::Left | VirtualKeyCode::Numpad4 | VirtualKeyCode::H => return function(-1, 0, ecs),
            VirtualKeyCode::Right | VirtualKeyCode::Numpad6 | VirtualKeyCode::L => return function(1, 0, ecs),
            VirtualKeyCode::Up | VirtualKeyCode::Numpad8 | VirtualKeyCode::K => return function(0, -1, ecs),
            VirtualKeyCode::Down | VirtualKeyCode::Numpad2 | VirtualKeyCode::J => return function(0, 1, ecs),
            // Diagonals
            VirtualKeyCode::Numpad9 | VirtualKeyCode::U => return function(1, -1, ecs),
            VirtualKeyCode::Numpad7 | VirtualKeyCode::Y => return function(-1, -1, ecs),
            VirtualKeyCode::Numpad3 | VirtualKeyCode::N => return function(1, 1, ecs),
            VirtualKeyCode::Numpad1 | VirtualKeyCode::B => return function(-1, 1, ecs),
            _ => return RunState::ActionWithDirection { function },
        },
    }
}

#[derive(PartialEq, Copy, Clone)]
pub enum ItemMenuResult {
    Cancel,
    NoResponse,
    Selected,
}

pub fn print_options(inventory: BTreeMap<(String, String), i32>, mut x: i32, mut y: i32, ctx: &mut Rltk) -> (i32, i32) {
    let mut j = 0;
    let initial_x: i32 = x;
    let mut width: i32 = -1;
    for (name, item_count) in &inventory {
        x = initial_x;
        // Print the character required to access this item. i.e. (a)
        if j < 26 {
            ctx.set(x, y, RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK), 97 + j as rltk::FontCharType);
        } else {
            // If we somehow have more than 26, start using capitals
            ctx.set(x, y, RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK), 65 - 26 + j as rltk::FontCharType);
        }

        x += 2;

        if item_count > &1 {
            // If more than one, print the number and pluralise
            // i.e. (a) 3 daggers
            ctx.print(x, y, item_count);
            x += 2;
            ctx.print(x, y, name.1.to_string());
        } else {
            if name.0.ends_with("s") {
                ctx.print(x, y, "some");
                x += 5;
            } else if ['a', 'e', 'i', 'o', 'u'].iter().any(|&v| name.0.starts_with(v)) {
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
        let this_width = x - initial_x + name.0.len() as i32;
        width = if width > this_width { width } else { this_width };

        y += 1;
        j += 1;
    }

    return (y, width);
}

pub fn get_max_inventory_width(inventory: &BTreeMap<(String, String), i32>) -> i32 {
    let mut width: i32 = 0;
    for (name, count) in inventory {
        let mut this_width = name.0.len() as i32;
        if count < &1 {
            this_width += 4;
            if name.0.ends_with("s") {
                this_width += 3;
            } else if ['a', 'e', 'i', 'o', 'u'].iter().any(|&v| name.0.starts_with(v)) {
                this_width += 1;
            }
        } else {
            this_width += 4;
        }
        width = if width > this_width { width } else { this_width };
    }
    return width;
}

pub fn show_help(ctx: &mut Rltk) -> YesNoResult {
    let mut x = 3;
    let mut y = 12;
    let height = 22;
    let width = 25;
    ctx.draw_box(x, y, width, height, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK));
    ctx.print_color(x + 3, y, RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK), " Controls ");
    ctx.print_color(x + 3, y + height, RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK), " ESC/? to close ");
    x += 2;
    y += 2;
    ctx.print_color(x, y, RGB::named(rltk::GREEN), RGB::named(rltk::BLACK), "MOVE COMMANDS");
    y += 2;
    ctx.print(x, y, "y k u   7 8 9   > down");
    ctx.print(x, y + 1, " \\|/     \\|/");
    ctx.print(x, y + 2, "h-.-l   4-.-6   < up");
    ctx.print(x, y + 3, " /|\\     /|\\");
    ctx.print(x, y + 4, "b j n   1 2 3   . wait");
    y += 7;
    ctx.print_color(x, y, RGB::named(rltk::GREEN), RGB::named(rltk::BLACK), "OBJECT INTERACTION");
    y += 2;
    ctx.print(x, y, "g get    d drop");
    y += 1;
    ctx.print(x, y, "i use    r unequip");
    y += 1;
    ctx.print(x, y, "o open   c close");
    y += 1;
    ctx.print(x, y, "f force");
    y += 2;
    ctx.print_color(x, y, RGB::named(rltk::GREEN), RGB::named(rltk::BLACK), "MOUSE CONTROL");
    y += 2;
    ctx.print(x, y, "hover for tooltips");

    match ctx.key {
        None => YesNoResult::NoSelection,
        Some(key) => match key {
            VirtualKeyCode::Escape => YesNoResult::Yes,
            VirtualKeyCode::Slash => {
                if ctx.shift {
                    return YesNoResult::Yes;
                }
                return YesNoResult::NoSelection;
            }
            _ => YesNoResult::NoSelection,
        },
    }
}

pub fn get_player_inventory(ecs: &World) -> (BTreeMap<(String, String), i32>, BTreeMap<String, Entity>) {
    let player_entity = ecs.fetch::<Entity>();
    let names = ecs.read_storage::<Name>();
    let backpack = ecs.read_storage::<InBackpack>();
    let entities = ecs.entities();

    let mut inventory_ids: BTreeMap<String, Entity> = BTreeMap::new();
    let mut player_inventory: BTreeMap<(String, String), i32> = BTreeMap::new();
    for (entity, _pack, name) in (&entities, &backpack, &names).join().filter(|item| item.1.owner == *player_entity) {
        player_inventory
            .entry((name.name.to_string(), name.plural.to_string()))
            .and_modify(|count| *count += 1)
            .or_insert(1);
        inventory_ids.entry(name.name.to_string()).or_insert(entity);
    }

    return (player_inventory, inventory_ids);
}

pub fn show_inventory(gs: &mut State, ctx: &mut Rltk) -> (ItemMenuResult, Option<Entity>) {
    let (player_inventory, inventory_ids) = get_player_inventory(&gs.ecs);
    let count = player_inventory.len();

    let (x_offset, y_offset) = (1, 10);

    ctx.print_color(
        1 + x_offset,
        1 + y_offset,
        RGB::named(rltk::WHITE),
        RGB::named(rltk::BLACK),
        "Interact with what item? [aA-zZ][Esc.]",
    );

    let x = 1 + x_offset;
    let y = 3 + y_offset;
    let width = get_max_inventory_width(&player_inventory);
    ctx.draw_box(x, y, width + 2, (count + 1) as i32, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK));
    print_options(player_inventory, x + 1, y + 1, ctx);

    match ctx.key {
        None => (ItemMenuResult::NoResponse, None),
        Some(key) => match key {
            VirtualKeyCode::Escape => (ItemMenuResult::Cancel, None),
            _ => {
                let selection = letter_to_option::letter_to_option(key, ctx.shift);
                if selection > -1 && selection < count as i32 {
                    return (ItemMenuResult::Selected, Some(*inventory_ids.iter().nth(selection as usize).unwrap().1));
                }
                (ItemMenuResult::NoResponse, None)
            }
        },
    }
}

pub fn drop_item_menu(gs: &mut State, ctx: &mut Rltk) -> (ItemMenuResult, Option<Entity>) {
    let (player_inventory, inventory_ids) = get_player_inventory(&gs.ecs);
    let count = player_inventory.len();

    let (x_offset, y_offset) = (1, 10);

    ctx.print_color(
        1 + x_offset,
        1 + y_offset,
        RGB::named(rltk::WHITE),
        RGB::named(rltk::BLACK),
        "Drop what? [aA-zZ][Esc.]",
    );

    let x = 1 + x_offset;
    let y = 3 + y_offset;
    let width = get_max_inventory_width(&player_inventory);
    ctx.draw_box(x, y, width + 2, (count + 1) as i32, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK));
    print_options(player_inventory, x + 1, y + 1, ctx);

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

    let (x_offset, y_offset) = (1, 10);

    ctx.print_color(
        1 + x_offset,
        1 + y_offset,
        RGB::named(rltk::WHITE),
        RGB::named(rltk::BLACK),
        "Drop what? [aA-zZ][Esc.]",
    );

    let mut equippable: Vec<(Entity, String)> = Vec::new();
    let mut width = 3;
    for (entity, _pack, name) in (&entities, &backpack, &names).join().filter(|item| item.1.owner == *player_entity) {
        let this_name = &name.name;
        let this_width = 3 + this_name.len();
        width = if width > this_width { width } else { this_width };
        equippable.push((entity, this_name.to_string()));
    }

    let x = 1 + x_offset;
    let mut y = 3 + y_offset;

    ctx.draw_box(x, y, width, (count + 1) as i32, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK));
    y += 1;

    let mut j = 0;
    for (_, name) in &equippable {
        ctx.set(x + 1, y, RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK), 97 + j as rltk::FontCharType);
        ctx.print(x + 3, y, name);
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
                    return (ItemMenuResult::Selected, Some(equippable[selection as usize].0));
                }
                (ItemMenuResult::NoResponse, None)
            }
        },
    }
}

pub fn ranged_target(gs: &mut State, ctx: &mut Rltk, range: i32, aoe: i32) -> (ItemMenuResult, Option<Point>) {
    let (min_x, max_x, min_y, max_y, x_offset, y_offset) = camera::get_screen_bounds(&gs.ecs, ctx);
    let player_entity = gs.ecs.fetch::<Entity>();
    let player_pos = gs.ecs.fetch::<Point>();
    let viewsheds = gs.ecs.read_storage::<Viewshed>();

    ctx.print_color(
        1 + x_offset,
        1 + y_offset,
        RGB::named(rltk::WHITE),
        RGB::named(rltk::BLACK),
        "Targeting which tile? [mouse input]",
    );

    // Highlight available cells
    let mut available_cells = Vec::new();
    let visible = viewsheds.get(*player_entity);
    if let Some(visible) = visible {
        // We have a viewshed
        for idx in visible.visible_tiles.iter() {
            let distance = rltk::DistanceAlg::Pythagoras.distance2d(*player_pos, *idx);
            if distance <= range as f32 {
                let screen_x = idx.x - min_x;
                let screen_y = idx.y - min_y;
                if screen_x > 1 && screen_x < (max_x - min_x) - 1 && screen_y > 1 && screen_y < (max_y - min_y) - 1 {
                    ctx.set_bg(screen_x + x_offset, screen_y + y_offset, RGB::named(rltk::BLUE));
                    available_cells.push(idx);
                }
            }
        }
    } else {
        return (ItemMenuResult::Cancel, None);
    }

    // Draw mouse cursor
    let mouse_pos = ctx.mouse_pos();
    let mut mouse_pos_adjusted = mouse_pos;
    mouse_pos_adjusted.0 += min_x - x_offset;
    mouse_pos_adjusted.1 += min_y - y_offset;
    let map = gs.ecs.fetch::<Map>();
    let mut valid_target = false;
    for idx in available_cells.iter() {
        if idx.x == mouse_pos_adjusted.0 && idx.y == mouse_pos_adjusted.1 {
            valid_target = true;
        }
    }
    if valid_target {
        if aoe > 0 {
            // We adjust for camera position when getting FOV, but then we need to adjust back
            // when iterating through the tiles themselves, by taking away min_x/min_y.
            let mut blast_tiles =
                rltk::field_of_view(Point::new(mouse_pos_adjusted.0, mouse_pos_adjusted.1), aoe, &*map);
            blast_tiles.retain(|p| p.x > 0 && p.x < map.width - 1 && p.y > 0 && p.y < map.height - 1);
            for tile in blast_tiles.iter() {
                ctx.set_bg(tile.x - min_x + x_offset, tile.y - min_y + y_offset, RGB::named(rltk::DARKCYAN));
            }
        }
        ctx.set_bg(mouse_pos.0, mouse_pos.1, RGB::named(rltk::CYAN));
        if ctx.left_click {
            return (ItemMenuResult::Selected, Some(Point::new(mouse_pos_adjusted.0, mouse_pos_adjusted.1)));
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

    let x = 46;
    let mut y = 26;
    let mut height = 8;
    if !save_exists {
        height -= 1;
    }

    ctx.draw_box_double(x, y - 4, 13, height, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK));
    ctx.print_color(x + 3, y - 2, RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK), "RUST-RL!");

    if let RunState::MainMenu { menu_selection: selection } = *runstate {
        if save_exists {
            if selection == MainMenuSelection::LoadGame {
                ctx.print_color(x + 2, y, RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK), "[");
                ctx.print_color(x + 3, y, RGB::named(rltk::GREEN), RGB::named(rltk::BLACK), "continue");
                ctx.print_color(x + 11, y, RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK), "]");
            } else {
                ctx.print_color(x + 3, y, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK), "continue");
            }
            y += 1;
        }
        if selection == MainMenuSelection::NewGame {
            ctx.print_color(x + 2, y, RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK), "[");
            ctx.print_color(x + 3, y, RGB::named(rltk::GREEN), RGB::named(rltk::BLACK), "new game");
            ctx.print_color(x + 11, y, RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK), "]");
        } else {
            ctx.print_color(x + 3, y, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK), "new game");
        }
        y += 1;
        if selection == MainMenuSelection::Quit {
            ctx.print_color(x + 2, y, RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK), "[");
            ctx.print_color(x + 3, y, RGB::named(rltk::GREEN), RGB::named(rltk::BLACK), "goodbye!");
            ctx.print_color(x + 11, y, RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK), "]");
        } else {
            ctx.print_color(x + 5, y, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK), "quit");
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
                        MainMenuSelection::NewGame => new_selection = MainMenuSelection::LoadGame,
                        MainMenuSelection::LoadGame => new_selection = MainMenuSelection::Quit,
                        MainMenuSelection::Quit => new_selection = MainMenuSelection::NewGame,
                    }
                    if new_selection == MainMenuSelection::LoadGame && !save_exists {
                        new_selection = MainMenuSelection::NewGame;
                    }
                    return MainMenuResult::NoSelection { selected: new_selection };
                }
                VirtualKeyCode::Down | VirtualKeyCode::Numpad2 | VirtualKeyCode::J => {
                    let mut new_selection;
                    match selection {
                        MainMenuSelection::NewGame => new_selection = MainMenuSelection::Quit,
                        MainMenuSelection::LoadGame => new_selection = MainMenuSelection::NewGame,
                        MainMenuSelection::Quit => new_selection = MainMenuSelection::LoadGame,
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
pub enum YesNoResult {
    NoSelection,
    Yes,
}

pub fn game_over(ctx: &mut Rltk) -> YesNoResult {
    let mut x = 3;
    let mut y = 12;
    let width = 45;
    let height = 20;
    ctx.draw_box(x, y, width, height, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK));
    ctx.print_color(x + 3, y, RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK), "You died!");
    ctx.print_color(x + 3, y + height, RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK), "ESC to close");
    x += 2;
    y += 2;
    ctx.print_color(
        x,
        y,
        RGB::named(rltk::GREEN),
        RGB::named(rltk::BLACK),
        format!("You survived for {} turns.", crate::gamelog::get_event_count("turns")),
    );
    y += 2;
    ctx.print_color(x, y, RGB::named(rltk::GREEN), RGB::named(rltk::BLACK), format!("And in the process, you"));
    y += 1;
    if crate::gamelog::get_event_count("descended") > 0 {
        ctx.print_color(
            x + 1,
            y,
            RGB::named(rltk::WHITE),
            RGB::named(rltk::BLACK),
            format!("- descended {} floor(s)", crate::gamelog::get_event_count("descended")),
        );
        y += 1;
    }
    if crate::gamelog::get_event_count("kick_count") > 0 {
        ctx.print_color(
            x + 1,
            y,
            RGB::named(rltk::WHITE),
            RGB::named(rltk::BLACK),
            format!(
                "- kicked {} time(s), breaking {} object(s)",
                crate::gamelog::get_event_count("kick_count"),
                crate::gamelog::get_event_count("broken_doors")
            ),
        );
        y += 1;
    }
    if crate::gamelog::get_event_count("death_count") > 0 {
        ctx.print_color(
            x + 1,
            y,
            RGB::named(rltk::WHITE),
            RGB::named(rltk::BLACK),
            format!("- slew {} other creature(s)", crate::gamelog::get_event_count("death_count")),
        );
        y += 1;
    }
    if crate::gamelog::get_event_count("looked_for_help") > 0 {
        ctx.print_color(
            x + 1,
            y,
            RGB::named(rltk::WHITE),
            RGB::named(rltk::BLACK),
            format!("- forgot the controls {} time(s)", crate::gamelog::get_event_count("looked_for_help")),
        );
    }

    match ctx.key {
        None => YesNoResult::NoSelection,
        Some(key) => match key {
            VirtualKeyCode::Escape => YesNoResult::Yes,
            _ => YesNoResult::NoSelection,
        },
    }
}
