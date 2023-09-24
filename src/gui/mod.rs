use super::{
    camera,
    gamelog,
    gamesystem,
    hunger_system::get_hunger_colour,
    rex_assets::RexAssets,
    ArmourClassBonus,
    Attributes,
    Beatitude,
    Burden,
    Charges,
    Equipped,
    Hidden,
    HungerClock,
    HungerState,
    InBackpack,
    KnownSpells,
    MagicItem,
    Map,
    MasterDungeonMap,
    Name,
    ObfuscatedName,
    Player,
    Point,
    Pools,
    Position,
    Prop,
    Renderable,
    RunState,
    states::state::*,
    Skill,
    Skills,
    Viewshed,
    BUC,
    consts::ids::get_local_col,
};
use crate::consts::prelude::*;
use crate::consts::entity::CARRY_CAPACITY_PER_STRENGTH;
use crate::consts::visuals::{
    TARGETING_LINE_COL,
    TARGETING_CURSOR_COL,
    TARGETING_AOE_COL,
    TARGETING_VALID_COL,
    VIEWPORT_W,
    VIEWPORT_H,
};
use crate::consts::{ TILESIZE, FONTSIZE };
use notan::prelude::*;
use notan::draw::{ Draw, DrawTextSection };
use std::collections::HashMap;
use bracket_lib::prelude::*;
use specs::prelude::*;
use std::collections::BTreeMap;
mod character_creation;
mod cheat_menu;
mod letter_to_option;
pub use character_creation::*;
mod remove_curse_menu;
pub use remove_curse_menu::*;
mod identify_menu;
pub use identify_menu::*;
mod tooltip;
pub use cheat_menu::*;
use crate::consts::events::*;
mod farlook;
pub use farlook::*;

/// Gives a popup box with a message and a title, and waits for a keypress.
#[allow(unused)]
pub fn yes_no(ctx: &mut BTerm, question: String) -> Option<bool> {
    ctx.print_color_centered(15, RGB::named(YELLOW), RGB::named(BLACK), question);
    ctx.print_color_centered(17, RGB::named(CYAN), RGB::named(BLACK), "(y)es or (n)o");
    match ctx.key {
        None => None,
        Some(key) =>
            match key {
                VirtualKeyCode::Y => Some(true),
                VirtualKeyCode::N => Some(false),
                _ => None,
            }
    }
}

pub fn draw_lerping_bar(
    ctx: &mut BTerm,
    sx: i32,
    sy: i32,
    width: i32,
    n: i32,
    max: i32,
    full_colour: RGB,
    empty_colour: RGB,
    with_text: bool,
    with_bg: bool
) {
    let percent = (n as f32) / (max as f32);
    let fill_width = (percent * (width as f32)) as i32;
    let bg = empty_colour.lerp(full_colour, percent);
    let black = RGB::named(BLACK);
    for x in 0..width {
        if x <= fill_width {
            ctx.print_color(sx + x, sy, black, bg, ' ');
        } else if with_bg {
            ctx.print_color(sx + x, sy, black, black, ' ');
        }
    }
    if with_text {
        ctx.print(sx - 1, sy, "[");
        let health = format!("{}/{}", n, max);
        ctx.print_color(sx + 1, sy, black, bg, health);
        ctx.print(sx + width, sy, "]");
    }
}

pub const TEXT_FONT_MOD: i32 = 2;

pub fn draw_ui2(
    ecs: &World,
    draw: &mut Draw,
    atlas: &HashMap<String, Texture>,
    font: &notan::draw::Font
) {
    let pools = ecs.read_storage::<Pools>();
    let attributes = ecs.read_storage::<Attributes>();
    let players = ecs.read_storage::<Player>();
    let hunger = ecs.read_storage::<HungerClock>();
    let burden = ecs.read_storage::<Burden>();
    let skills = ecs.read_storage::<Skills>();
    for (_p, stats, attributes, hunger, skills) in (
        &players,
        &pools,
        &attributes,
        &hunger,
        &skills,
    ).join() {
        let initial_x = 26.0 * TILESIZE;
        let mut x = initial_x;
        let y = 53.0 * TILESIZE;
        // TODO: Draw hp/mana bars
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
        let armour_class =
            stats.bac - attributes.dexterity.bonus / 2 - skill_ac_bonus - armour_ac_bonus;
        draw.text(&font, "AC").position(x, y).color(Color::PINK).size(FONTSIZE);
        x = draw.last_text_bounds().max_x();
        draw.text(&font, &format!("{}", armour_class)).position(x, y).size(FONTSIZE);
        draw.text(&font, &format!("XP{}/{}", stats.level, stats.xp))
            .position(initial_x, y + TILESIZE)
            .size(FONTSIZE);
        // TODO: Finish ui (placeholder)
    }
}

pub fn draw_ui(ecs: &World, ctx: &mut BTerm) {
    ctx.set_active_console(TEXT_LAYER);
    // Render stats
    let pools = ecs.read_storage::<Pools>();
    let attributes = ecs.read_storage::<Attributes>();
    let players = ecs.read_storage::<Player>();
    let hunger = ecs.read_storage::<HungerClock>();
    let burden = ecs.read_storage::<Burden>();
    let skills = ecs.read_storage::<Skills>();
    for (_player, stats, attributes, hunger, skills) in (
        &players,
        &pools,
        &attributes,
        &hunger,
        &skills,
    ).join() {
        // Draw hp/mana bars
        draw_lerping_bar(
            ctx,
            2 * TEXT_FONT_MOD,
            53,
            22 * TEXT_FONT_MOD,
            stats.hit_points.current,
            stats.hit_points.max,
            RGB::from_u8(0, 255, 0),
            RGB::from_u8(255, 0, 0),
            true,
            true
        );
        draw_lerping_bar(
            ctx,
            2 * TEXT_FONT_MOD,
            54,
            22 * TEXT_FONT_MOD,
            stats.mana.current,
            stats.mana.max,
            RGB::named(BLUE),
            RGB::named(BLACK),
            true,
            true
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
        let armour_class =
            stats.bac - attributes.dexterity.bonus / 2 - skill_ac_bonus - armour_ac_bonus;
        ctx.print_color(26 * TEXT_FONT_MOD, 53, RGB::named(PINK), RGB::named(BLACK), "AC");
        ctx.print_color(28 * TEXT_FONT_MOD, 53, RGB::named(WHITE), RGB::named(BLACK), armour_class);
        // Draw level
        ctx.print_color(
            26 * TEXT_FONT_MOD,
            54,
            RGB::named(WHITE),
            RGB::named(BLACK),
            format!("XP{}/{}", stats.level, stats.xp)
        );
        // Draw attributes
        let x = 38 * TEXT_FONT_MOD;
        ctx.print_color(x, 53, RGB::named(RED), RGB::named(BLACK), "STR");
        ctx.print_color(x + 3, 53, RGB::named(WHITE), RGB::named(BLACK), attributes.strength.base);
        ctx.print_color(x + 7, 53, RGB::named(GREEN), RGB::named(BLACK), "DEX");
        ctx.print_color(
            x + 10,
            53,
            RGB::named(WHITE),
            RGB::named(BLACK),
            attributes.dexterity.base
        );
        ctx.print_color(x + 14, 53, RGB::named(ORANGE), RGB::named(BLACK), "CON");
        ctx.print_color(
            x + 17,
            53,
            RGB::named(WHITE),
            RGB::named(BLACK),
            attributes.constitution.base
        );
        ctx.print_color(x, 54, RGB::named(CYAN), RGB::named(BLACK), "INT");
        ctx.print_color(
            x + 3,
            54,
            RGB::named(WHITE),
            RGB::named(BLACK),
            attributes.intelligence.base
        );
        ctx.print_color(x + 7, 54, RGB::named(YELLOW), RGB::named(BLACK), "WIS");
        ctx.print_color(x + 10, 54, RGB::named(WHITE), RGB::named(BLACK), attributes.wisdom.base);
        ctx.print_color(x + 14, 54, RGB::named(PURPLE), RGB::named(BLACK), "CHA");
        ctx.print_color(x + 17, 54, RGB::named(WHITE), RGB::named(BLACK), attributes.charisma.base);
        // Draw hunger
        match hunger.state {
            HungerState::Satiated => {
                ctx.print_color_right(
                    (VIEWPORT_W + 1) * TEXT_FONT_MOD,
                    53,
                    get_hunger_colour(hunger.state),
                    RGB::named(BLACK),
                    "Satiated"
                );
            }
            HungerState::Normal => {}
            HungerState::Hungry => {
                ctx.print_color_right(
                    (VIEWPORT_W + 1) * TEXT_FONT_MOD,
                    53,
                    get_hunger_colour(hunger.state),
                    RGB::named(BLACK),
                    "Hungry"
                );
            }
            HungerState::Weak => {
                ctx.print_color_right(
                    (VIEWPORT_W + 1) * TEXT_FONT_MOD,
                    53,
                    get_hunger_colour(hunger.state),
                    RGB::named(BLACK),
                    "Weak"
                );
            }
            HungerState::Fainting => {
                ctx.print_color_right(
                    (VIEWPORT_W + 1) * TEXT_FONT_MOD,
                    53,
                    get_hunger_colour(hunger.state),
                    RGB::named(BLACK),
                    "Fainting"
                );
            }
            HungerState::Starving => {
                ctx.print_color_right(
                    (VIEWPORT_W + 1) * TEXT_FONT_MOD,
                    53,
                    get_hunger_colour(hunger.state),
                    RGB::named(BLACK),
                    "Starving"
                );
            }
        }
        // Burden
        if let Some(burden) = burden.get(*player_entity) {
            match burden.level {
                crate::BurdenLevel::Burdened => {
                    ctx.print_color_right(
                        (VIEWPORT_W + 1) * TEXT_FONT_MOD,
                        50,
                        RGB::named(BROWN1),
                        RGB::named(BLACK),
                        "Burdened"
                    );
                }
                crate::BurdenLevel::Strained => {
                    ctx.print_color_right(
                        (VIEWPORT_W + 1) * TEXT_FONT_MOD,
                        50,
                        RGB::named(ORANGE),
                        RGB::named(BLACK),
                        "Strained"
                    );
                }
                crate::BurdenLevel::Overloaded => {
                    ctx.print_color_right(
                        (VIEWPORT_W + 1) * TEXT_FONT_MOD,
                        50,
                        RGB::named(RED),
                        RGB::named(BLACK),
                        "Overloaded"
                    );
                }
            }
        }
        if stats.god {
            ctx.print_color(
                20 * TEXT_FONT_MOD,
                20,
                RGB::named(YELLOW),
                RGB::named(BLACK),
                "--- GODMODE: ON ---"
            );
        }
        // Draw equipment
        let renderables = ecs.read_storage::<Renderable>();
        let mut equipment: Vec<(String, RGB, RGB, FontCharType)> = Vec::new();
        let entities = ecs.entities();
        for (entity, _equipped, renderable) in (&entities, &equipped, &renderables)
            .join()
            .filter(|item| item.1.owner == *player_entity) {
            equipment.push((
                obfuscate_name_ecs(ecs, entity).0,
                RGB::named(item_colour_ecs(ecs, entity)),
                renderable.fg,
                renderable.glyph,
            ));
        }
        let mut y = 1;
        if !equipment.is_empty() {
            ctx.print_color(
                (VIEWPORT_W + 3) * TEXT_FONT_MOD,
                y,
                RGB::named(BLACK),
                RGB::named(WHITE),
                "Equipment"
            );
            let mut j = 0;
            for item in equipment {
                y += 1;
                ctx.set(
                    (VIEWPORT_W + 3) * TEXT_FONT_MOD,
                    y,
                    RGB::named(YELLOW),
                    RGB::named(BLACK),
                    97 + (j as FontCharType)
                );
                j += 1;
                ctx.set((VIEWPORT_W + 3) * TEXT_FONT_MOD + 2, y, item.2, RGB::named(BLACK), item.3);
                ctx.print_color(
                    (VIEWPORT_W + 3) * TEXT_FONT_MOD + 4,
                    y,
                    item.1,
                    RGB::named(BLACK),
                    &item.0
                );
                ctx.print_color(
                    (VIEWPORT_W + 3) * TEXT_FONT_MOD + 4 + (item.0.len() as i32) + 1,
                    y,
                    RGB::named(WHITE),
                    RGB::named(BLACK),
                    "(worn)"
                );
            }
            y += 2;
        }

        // Draw consumables
        ctx.print_color(
            (VIEWPORT_W + 3) * TEXT_FONT_MOD,
            y,
            RGB::named(BLACK),
            RGB::named(WHITE),
            "Backpack"
        );
        ctx.print_color(
            (VIEWPORT_W + 12) * TEXT_FONT_MOD,
            y,
            RGB::named(WHITE),
            RGB::named(BLACK),
            &format!(
                "[{:.1}/{} lbs]",
                stats.weight,
                (attributes.strength.base + attributes.strength.modifiers) *
                    CARRY_CAPACITY_PER_STRENGTH
            )
        );
        y += 1;
        let player_inventory = get_player_inventory(&ecs);
        y = print_options(&player_inventory, (VIEWPORT_W + 3) * TEXT_FONT_MOD, y, ctx).0;

        // Draw spells - if we have any -- NYI!
        if let Some(known_spells) = ecs.read_storage::<KnownSpells>().get(*player_entity) {
            y += 1;
            // Draw known spells
            ctx.print_color(
                (VIEWPORT_W + 3) * TEXT_FONT_MOD,
                y,
                RGB::named(BLACK),
                RGB::named(WHITE),
                "Known Spells"
            );
            y += 1;
            let mut index = 1;
            for spell in known_spells.list.iter() {
                ctx.print_color(
                    (VIEWPORT_W + 3) * TEXT_FONT_MOD,
                    y,
                    RGB::named(YELLOW),
                    RGB::named(BLACK),
                    &format!("{}", index)
                );
                ctx.print_color(
                    (VIEWPORT_W + 3) * TEXT_FONT_MOD + 2,
                    y,
                    RGB::named(CYAN),
                    RGB::named(BLACK),
                    &format!("{} ({})", "Force Bolt - NYI!", spell.mana_cost)
                );
                index += 1;
                y += 1;
            }
        }

        // Draw entities seen on screen
        let viewsheds = ecs.read_storage::<Viewshed>();
        let renderables = ecs.read_storage::<Renderable>();
        let names = ecs.read_storage::<Name>();
        let hidden = ecs.read_storage::<Hidden>();
        let props = ecs.read_storage::<Prop>();
        let map = ecs.fetch::<Map>();
        let viewshed = viewsheds.get(*player_entity).unwrap();
        let mut seen_entities: Vec<(String, RGB, RGB, u16)> = Vec::new();
        for tile in viewshed.visible_tiles.iter() {
            let idx = map.xy_idx(tile.x, tile.y);
            crate::spatial::for_each_tile_content(idx, |entity| {
                let mut draw = false;
                if let Some(_) = names.get(entity) {
                    draw = true;
                }
                let prop = props.get(entity);
                if let Some(_) = prop {
                    draw = false;
                }
                let is_hidden = hidden.get(entity);
                if let Some(_) = is_hidden {
                    draw = false;
                }
                if entity == *player_entity {
                    draw = false;
                }
                if draw {
                    let (render_fg, glyph) = if let Some(renderable) = renderables.get(entity) {
                        (renderable.fg, renderable.glyph)
                    } else {
                        (RGB::named(WHITE), to_cp437('-'))
                    };
                    seen_entities.push((
                        obfuscate_name_ecs(ecs, entity).0,
                        RGB::named(item_colour_ecs(ecs, entity)),
                        render_fg,
                        glyph,
                    ));
                }
            });
        }
        seen_entities.sort_by(|a, b| b.0.cmp(&a.0));

        if !seen_entities.is_empty() {
            y += 1;
            ctx.print_color(
                (VIEWPORT_W + 3) * TEXT_FONT_MOD,
                y,
                RGB::named(BLACK),
                RGB::named(WHITE),
                "In View"
            );
            for entity in seen_entities {
                y += 1;
                ctx.set((VIEWPORT_W + 3) * TEXT_FONT_MOD, y, entity.2, RGB::named(BLACK), entity.3);
                ctx.print_color(
                    (VIEWPORT_W + 3) * TEXT_FONT_MOD + 2,
                    y,
                    entity.1,
                    RGB::named(BLACK),
                    entity.0
                );
            }
        }
    }

    // Render the message log at [1, 7], ascending, with 7 lines and a max width of 68.
    gamelog::print_log(
        &mut BACKEND_INTERNAL.lock().consoles[TEXT_LAYER].console,
        Point::new(1 * TEXT_FONT_MOD, 7),
        false,
        7,
        (VIEWPORT_W - 1) * TEXT_FONT_MOD
    );

    // Render id
    let map = ecs.fetch::<Map>();
    let id = if map.depth > 0 {
        format!("{}{}", map.short_name, map.depth)
    } else {
        format!("{}", map.short_name)
    };
    ctx.print_color_right(
        (VIEWPORT_W + 1) * TEXT_FONT_MOD,
        54,
        get_local_col(map.id),
        RGB::named(BLACK),
        &id
    );

    // Render turn
    let turns = crate::gamelog::get_event_count(EVENT::COUNT_TURN);
    ctx.print_color_right(
        VIEWPORT_W * TEXT_FONT_MOD - (id.len() as i32),
        54,
        RGB::named(YELLOW),
        RGB::named(BLACK),
        &format!("T{}", turns)
    );

    // Boxes and tooltips last, so they draw over everything else.
    ctx.draw_hollow_box(
        0 * TEXT_FONT_MOD,
        0,
        (VIEWPORT_W + 1) * TEXT_FONT_MOD,
        8,
        RGB::named(WHITE),
        RGB::named(BLACK)
    ); // Log box
    ctx.draw_hollow_box(
        0 * TEXT_FONT_MOD,
        9,
        (VIEWPORT_W + 1) * TEXT_FONT_MOD,
        42,
        RGB::named(WHITE),
        RGB::named(BLACK)
    ); // Camera box
    ctx.draw_hollow_box(
        0 * TEXT_FONT_MOD,
        52,
        (VIEWPORT_W + 1) * TEXT_FONT_MOD,
        3,
        RGB::named(WHITE),
        RGB::named(BLACK)
    ); // Stats box
    ctx.draw_hollow_box(
        (VIEWPORT_W + 2) * TEXT_FONT_MOD,
        0,
        33 * TEXT_FONT_MOD,
        55,
        RGB::named(WHITE),
        RGB::named(BLACK)
    ); // Side box
    ctx.set_active_console(TILE_LAYER);
    tooltip::draw_tooltips(ecs, ctx, None);
}

pub fn get_input_direction(
    ecs: &mut World,
    ctx: &mut BTerm,
    function: fn(i: i32, j: i32, ecs: &mut World) -> RunState
) -> RunState {
    let offsets = camera::get_offset();

    ctx.print_color(
        1 + offsets.x,
        1 + offsets.y,
        RGB::named(WHITE),
        RGB::named(BLACK),
        "In what direction? [0-9]/[YUHJKLBN]"
    );
    match ctx.key {
        None => {
            return RunState::ActionWithDirection { function };
        }
        Some(key) =>
            match key {
                VirtualKeyCode::Escape => {
                    return RunState::AwaitingInput;
                }
                // Cardinals
                VirtualKeyCode::Left | VirtualKeyCode::Numpad4 | VirtualKeyCode::H => {
                    return function(-1, 0, ecs);
                }
                VirtualKeyCode::Right | VirtualKeyCode::Numpad6 | VirtualKeyCode::L => {
                    return function(1, 0, ecs);
                }
                VirtualKeyCode::Up | VirtualKeyCode::Numpad8 | VirtualKeyCode::K => {
                    return function(0, -1, ecs);
                }
                VirtualKeyCode::Down | VirtualKeyCode::Numpad2 | VirtualKeyCode::J => {
                    return function(0, 1, ecs);
                }
                // Diagonals
                VirtualKeyCode::Numpad9 | VirtualKeyCode::U => {
                    return function(1, -1, ecs);
                }
                VirtualKeyCode::Numpad7 | VirtualKeyCode::Y => {
                    return function(-1, -1, ecs);
                }
                VirtualKeyCode::Numpad3 | VirtualKeyCode::N => {
                    return function(1, 1, ecs);
                }
                VirtualKeyCode::Numpad1 | VirtualKeyCode::B => {
                    return function(-1, 1, ecs);
                }
                _ => {
                    return RunState::ActionWithDirection { function };
                }
            }
    }
}

#[derive(PartialEq, Copy, Clone)]
pub enum ItemMenuResult {
    Cancel,
    NoResponse,
    Selected,
}

pub fn print_options(
    inventory: &PlayerInventory,
    mut x: i32,
    mut y: i32,
    ctx: &mut BTerm
) -> (i32, i32) {
    let mut j = 0;
    let initial_x: i32 = x;
    let mut width: i32 = -1;
    for (item, (_e, item_count)) in inventory {
        x = initial_x;
        // Print the character required to access this item. i.e. (a)
        if j < 26 {
            ctx.set(x, y, RGB::named(YELLOW), RGB::named(BLACK), 97 + (j as FontCharType));
        } else {
            // If we somehow have more than 26, start using capitals
            ctx.set(x, y, RGB::named(YELLOW), RGB::named(BLACK), 65 - 26 + (j as FontCharType));
        }

        x += 2;
        let fg = RGB::from_u8(item.renderables.0, item.renderables.1, item.renderables.2);
        ctx.set(x, y, fg, RGB::named(BLACK), item.glyph);
        x += 2;

        let fg = RGB::from_u8(item.rgb.0, item.rgb.1, item.rgb.2);
        if item_count > &1 {
            // If more than one, print the number and pluralise
            // i.e. (a) 3 daggers
            ctx.print_color(x, y, fg, RGB::named(BLACK), item_count);
            x += 2;
            ctx.print_color(x, y, fg, RGB::named(BLACK), item.display_name.plural.to_string());
            let this_width = x - initial_x + (item.display_name.plural.len() as i32);
            width = if width > this_width { width } else { this_width };
        } else {
            if item.display_name.singular.to_lowercase().ends_with("s") {
                ctx.print_color(x, y, fg, RGB::named(BLACK), "some");
                x += 5;
            } else if
                ['a', 'e', 'i', 'o', 'u']
                    .iter()
                    .any(|&v| item.display_name.singular.to_lowercase().starts_with(v))
            {
                // If one and starts with a vowel, print 'an'
                // i.e. (a) an apple
                ctx.print_color(x, y, fg, RGB::named(BLACK), "an");
                x += 3;
            } else {
                // If one and not a vowel, print 'a'
                // i.e. (a) a dagger
                ctx.print_color(x, y, fg, RGB::named(BLACK), "a");
                x += 2;
            }
            ctx.print_color(x, y, fg, RGB::named(BLACK), item.display_name.singular.to_string());
            let this_width = x - initial_x + (item.display_name.singular.len() as i32);
            width = if width > this_width { width } else { this_width };
        }

        y += 1;
        j += 1;
    }

    return (y, width);
}

pub fn get_max_inventory_width(inventory: &PlayerInventory) -> i32 {
    let mut width: i32 = 0;
    for (item, (_e, count)) in inventory {
        let mut this_width = 4; // The spaces before and after the character to select this item, etc.
        if count <= &1 {
            this_width += item.display_name.singular.len() as i32;
            if item.display_name.singular == item.display_name.plural {
                this_width += 4; // "some".len
            } else if
                ['a', 'e', 'i', 'o', 'u'].iter().any(|&v| item.display_name.singular.starts_with(v))
            {
                this_width += 2; // "an".len
            } else {
                this_width += 1; // "a".len
            }
        } else {
            this_width += item.display_name.plural.len() as i32;
            this_width += count.to_string().len() as i32; // i.e. "12".len
        }
        width = if width > this_width { width } else { this_width };
    }
    return width;
}

// Inside the ECS
pub fn obfuscate_name(
    item: Entity,
    names: &ReadStorage<Name>,
    magic_items: &ReadStorage<MagicItem>,
    obfuscated_names: &ReadStorage<ObfuscatedName>,
    beatitudes: &ReadStorage<Beatitude>,
    dm: &MasterDungeonMap,
    wand: Option<&ReadStorage<Charges>>
) -> (String, String) {
    let (mut singular, mut plural) = (
        "nameless item (bug)".to_string(),
        "nameless items (bug)".to_string(),
    );
    if let Some(name) = names.get(item) {
        if magic_items.get(item).is_some() {
            if dm.identified_items.contains(&name.name) {
                (singular, plural) = (name.name.clone(), name.plural.clone());
                if wand.is_some() {
                    let wands = wand.unwrap();
                    if let Some(wand) = wands.get(item) {
                        let used = wand.max_uses - wand.uses;
                        for _i in 0..used {
                            singular.push_str("*");
                            plural.push_str("*");
                        }
                    }
                }
            } else if let Some(obfuscated) = obfuscated_names.get(item) {
                (singular, plural) = (obfuscated.name.clone(), obfuscated.plural.clone());
            } else {
                (singular, plural) = (
                    "unid magic item".to_string(),
                    "unid magic items".to_string(),
                );
            }
        } else {
            (singular, plural) = (name.name.clone(), name.plural.clone());
        }
    }
    if let Some(has_beatitude) = beatitudes.get(item) {
        if has_beatitude.known {
            let prefix = match has_beatitude.buc {
                BUC::Cursed => Some("cursed "),
                BUC::Uncursed => None,
                BUC::Blessed => Some("blessed "),
            };
            if prefix.is_some() {
                singular.insert_str(0, prefix.unwrap());
                plural.insert_str(0, prefix.unwrap());
            }
        }
    }
    return (singular, plural);
}

// Outside the ECS
pub fn obfuscate_name_ecs(ecs: &World, item: Entity) -> (String, String) {
    let (mut singular, mut plural) = (
        "nameless item (bug)".to_string(),
        "nameless items (bug)".to_string(),
    );
    if let Some(name) = ecs.read_storage::<Name>().get(item) {
        if ecs.read_storage::<MagicItem>().get(item).is_some() {
            let dm = ecs.fetch::<MasterDungeonMap>();
            if dm.identified_items.contains(&name.name) {
                (singular, plural) = (name.name.clone(), name.plural.clone());
                if let Some(wand) = ecs.read_storage::<Charges>().get(item) {
                    let used = wand.max_uses - wand.uses;
                    for _i in 0..used {
                        singular.push_str("*");
                        plural.push_str("*");
                    }
                }
            } else if let Some(obfuscated) = ecs.read_storage::<ObfuscatedName>().get(item) {
                (singular, plural) = (obfuscated.name.clone(), obfuscated.plural.clone());
            } else {
                (singular, plural) = (
                    "unid magic item".to_string(),
                    "unid magic items".to_string(),
                );
            }
        } else {
            (singular, plural) = (name.name.clone(), name.plural.clone());
        }
    }
    if let Some(has_beatitude) = ecs.read_storage::<Beatitude>().get(item) {
        if has_beatitude.known {
            let prefix = match has_beatitude.buc {
                BUC::Cursed => Some("cursed "),
                BUC::Uncursed => Some("uncursed "),
                BUC::Blessed => Some("blessed "),
            };
            if prefix.is_some() {
                singular.insert_str(0, prefix.unwrap());
                plural.insert_str(0, prefix.unwrap());
            }
        }
    }
    return (singular, plural);
}

pub fn unobf_name_ecs(ecs: &World, item: Entity) -> (String, String) {
    let (mut singular, mut plural) = ("nameless (bug)".to_string(), "nameless (bug)".to_string());
    if let Some(name) = ecs.read_storage::<Name>().get(item) {
        (singular, plural) = (name.name.clone(), name.plural.clone());
    }
    if let Some(has_beatitude) = ecs.read_storage::<Beatitude>().get(item) {
        let prefix = match has_beatitude.buc {
            BUC::Cursed => "cursed ",
            BUC::Uncursed => "uncursed ",
            BUC::Blessed => "blessed ",
        };
        singular.insert_str(0, prefix);
        plural.insert_str(0, prefix);
    }
    return (singular, plural);
}

/// Gets renderable colour as tuple of u8
pub fn renderable_colour(renderables: &ReadStorage<Renderable>, entity: Entity) -> (u8, u8, u8) {
    return if let Some(renderable) = renderables.get(entity) {
        (
            (renderable.fg.r * 255.0) as u8,
            (renderable.fg.g * 255.0) as u8,
            (renderable.fg.b * 255.0) as u8,
        )
    } else {
        WHITE
    };
}

/// Gets renderable colour as tuple of u8
pub fn renderable_colour_ecs(ecs: &World, entity: Entity) -> (u8, u8, u8) {
    return if let Some(renderable) = ecs.read_storage::<Renderable>().get(entity) {
        (
            (renderable.fg.r * 255.0) as u8,
            (renderable.fg.g * 255.0) as u8,
            (renderable.fg.b * 255.0) as u8,
        )
    } else {
        WHITE
    };
}

pub fn item_colour_ecs(ecs: &World, item: Entity) -> (u8, u8, u8) {
    if let Some(beatitude) = ecs.read_storage::<Beatitude>().get(item) {
        if beatitude.known {
            match beatitude.buc {
                BUC::Blessed => {
                    return GREEN;
                }
                BUC::Uncursed => {
                    return WHITE;
                }
                BUC::Cursed => {
                    return RED;
                }
            }
        } else {
            // Unidentified magic item
            return GREY;
        }
    }
    // If nonmagic, just use white
    return WHITE;
}

pub fn item_colour(item: Entity, beatitudes: &ReadStorage<Beatitude>) -> (u8, u8, u8) {
    if let Some(beatitude) = beatitudes.get(item) {
        if beatitude.known {
            match beatitude.buc {
                BUC::Blessed => {
                    return GREEN;
                }
                BUC::Uncursed => {
                    return WHITE;
                }
                BUC::Cursed => {
                    return RED;
                }
            }
        } else {
            // Unidentified magic item
            return GREY;
        }
    }
    // If nonmagic, just use white
    return WHITE;
}

pub fn show_help(ctx: &mut BTerm) -> YesNoResult {
    let mut x = 3;
    let mut y = 12;
    let height = 22;
    let width = 25;
    ctx.draw_box(x, y, width, height, RGB::named(WHITE), RGB::named(BLACK));
    ctx.print_color(x + 3, y, RGB::named(YELLOW), RGB::named(BLACK), " Controls ");
    ctx.print_color(x + 3, y + height, RGB::named(YELLOW), RGB::named(BLACK), " ESC/? to close ");
    x += 2;
    y += 2;
    ctx.print_color(x, y, RGB::named(GREEN), RGB::named(BLACK), "MOVE COMMANDS");
    y += 2;
    ctx.print(x, y, "y k u   7 8 9   > down");
    ctx.print(x, y + 1, " \\|/     \\|/");
    ctx.print(x, y + 2, "h-.-l   4-.-6   < up");
    ctx.print(x, y + 3, " /|\\     /|\\");
    ctx.print(x, y + 4, "b j n   1 2 3   . wait");
    y += 7;
    ctx.print_color(x, y, RGB::named(GREEN), RGB::named(BLACK), "OBJECT INTERACTION");
    y += 2;
    ctx.print(x, y, "g get    d drop");
    y += 1;
    ctx.print(x, y, "i use    r unequip");
    y += 1;
    ctx.print(x, y, "o open   c close");
    y += 1;
    ctx.print(x, y, "f force  x farlook");
    y += 2;
    ctx.print_color(x, y, RGB::named(GREEN), RGB::named(BLACK), "MOUSE CONTROL");
    y += 2;
    ctx.print(x, y, "hover for tooltips");

    match ctx.key {
        None => YesNoResult::NoSelection,
        Some(key) =>
            match key {
                VirtualKeyCode::Escape => YesNoResult::Yes,
                VirtualKeyCode::Slash => {
                    if ctx.shift {
                        return YesNoResult::Yes;
                    }
                    return YesNoResult::NoSelection;
                }
                _ => YesNoResult::NoSelection,
            }
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
struct DisplayName {
    singular: String,
    plural: String,
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct UniqueInventoryItem {
    display_name: DisplayName,
    rgb: (u8, u8, u8),
    renderables: (u8, u8, u8),
    glyph: u16,
    beatitude_status: i32,
    name: String,
}

pub type PlayerInventory = BTreeMap<UniqueInventoryItem, (Entity, i32)>;

pub fn get_player_inventory(ecs: &World) -> PlayerInventory {
    let player_entity = ecs.fetch::<Entity>();
    let names = ecs.read_storage::<Name>();
    let backpack = ecs.read_storage::<InBackpack>();
    let entities = ecs.entities();
    let renderables = ecs.read_storage::<Renderable>();

    let mut player_inventory: BTreeMap<UniqueInventoryItem, (Entity, i32)> = BTreeMap::new();
    for (entity, _pack, name, renderable) in (&entities, &backpack, &names, &renderables)
        .join()
        .filter(|item| item.1.owner == *player_entity) {
        // RGB can't be used as a key. This is converting the RGB (tuple of f32) into a tuple of u8s.
        let item_colour = item_colour_ecs(ecs, entity);
        let renderables = (
            (renderable.fg.r * 255.0) as u8,
            (renderable.fg.g * 255.0) as u8,
            (renderable.fg.b * 255.0) as u8,
        );
        let (singular, plural) = obfuscate_name_ecs(ecs, entity);
        let beatitude_status = if let Some(beatitude) = ecs.read_storage::<Beatitude>().get(entity) {
            match beatitude.buc {
                BUC::Blessed => 1,
                BUC::Uncursed => 2,
                BUC::Cursed => 3,
            }
        } else {
            0
        };
        let unique_item = UniqueInventoryItem {
            display_name: DisplayName { singular: singular.clone(), plural: plural },
            rgb: item_colour,
            renderables: renderables,
            glyph: renderable.glyph,
            beatitude_status: beatitude_status,
            name: name.name.clone(),
        };
        player_inventory
            .entry(unique_item)
            .and_modify(|(_e, count)| {
                *count += 1;
            })
            .or_insert((entity, 1));
    }

    return player_inventory;
}

pub fn show_inventory(gs: &mut State, ctx: &mut BTerm) -> (ItemMenuResult, Option<Entity>) {
    ctx.set_active_console(TEXT_LAYER);

    let player_inventory = get_player_inventory(&gs.ecs);
    let count = player_inventory.len();

    let (x_offset, y_offset) = (1 * TEXT_FONT_MOD, 10);

    let on_overmap = gs.ecs.fetch::<Map>().overmap;
    let message = if !on_overmap {
        "Interact with what item? [aA-zZ][Esc.]"
    } else {
        "You can't use items on the overmap [Esc.]"
    };

    ctx.print_color(1 + x_offset, 1 + y_offset, RGB::named(WHITE), RGB::named(BLACK), message);

    let x = 1 + x_offset;
    let y = 3 + y_offset;
    let width = get_max_inventory_width(&player_inventory);
    ctx.draw_box(x, y, width + 2, (count + 1) as i32, RGB::named(WHITE), RGB::named(BLACK));
    print_options(&player_inventory, x + 1, y + 1, ctx);

    ctx.set_active_console(TILE_LAYER);

    match ctx.key {
        None => (ItemMenuResult::NoResponse, None),
        Some(key) =>
            match key {
                VirtualKeyCode::Escape => (ItemMenuResult::Cancel, None),
                _ => {
                    let selection = letter_to_option::letter_to_option(key, ctx.shift);
                    if selection > -1 && selection < (count as i32) {
                        if on_overmap {
                            gamelog::Logger
                                ::new()
                                .append("You can't use items on the overmap.")
                                .log();
                        } else {
                            return (
                                ItemMenuResult::Selected,
                                Some(
                                    player_inventory
                                        .iter()
                                        .nth(selection as usize)
                                        .unwrap().1.0
                                ),
                            );
                        }
                    }
                    (ItemMenuResult::NoResponse, None)
                }
            }
    }
}

pub fn drop_item_menu(gs: &mut State, ctx: &mut BTerm) -> (ItemMenuResult, Option<Entity>) {
    let player_inventory = get_player_inventory(&gs.ecs);
    let count = player_inventory.len();

    let (x_offset, y_offset) = (1, 10);

    let on_overmap = gs.ecs.fetch::<Map>().overmap;
    let message = if !on_overmap {
        "Drop what? [aA-zZ][Esc.]"
    } else {
        "You can't drop items on the overmap [Esc.]"
    };

    ctx.print_color(1 + x_offset, 1 + y_offset, RGB::named(WHITE), RGB::named(BLACK), message);

    let x = 1 + x_offset;
    let y = 3 + y_offset;
    let width = get_max_inventory_width(&player_inventory);
    ctx.draw_box(x, y, width + 2, (count + 1) as i32, RGB::named(WHITE), RGB::named(BLACK));
    print_options(&player_inventory, x + 1, y + 1, ctx);

    match ctx.key {
        None => (ItemMenuResult::NoResponse, None),
        Some(key) =>
            match key {
                VirtualKeyCode::Escape => (ItemMenuResult::Cancel, None),
                _ => {
                    let selection = letter_to_option(key);
                    if selection > -1 && selection < (count as i32) {
                        if on_overmap {
                            gamelog::Logger
                                ::new()
                                .append("You can't drop items on the overmap.")
                                .log();
                        } else {
                            return (
                                ItemMenuResult::Selected,
                                Some(
                                    player_inventory
                                        .iter()
                                        .nth(selection as usize)
                                        .unwrap().1.0
                                ),
                            );
                        }
                    }
                    (ItemMenuResult::NoResponse, None)
                }
            }
    }
}

pub fn remove_item_menu(gs: &mut State, ctx: &mut BTerm) -> (ItemMenuResult, Option<Entity>) {
    let player_entity = gs.ecs.fetch::<Entity>();
    let backpack = gs.ecs.read_storage::<Equipped>();
    let entities = gs.ecs.entities();
    let inventory = (&backpack).join().filter(|item| item.owner == *player_entity);
    let count = inventory.count();

    let (x_offset, y_offset) = (1, 10);

    ctx.print_color(
        1 + x_offset,
        1 + y_offset,
        RGB::named(WHITE),
        RGB::named(BLACK),
        "Unequip what? [aA-zZ][Esc.]"
    );

    let mut equippable: Vec<(Entity, String)> = Vec::new();
    let mut width = 2;
    for (entity, _pack) in (&entities, &backpack)
        .join()
        .filter(|item| item.1.owner == *player_entity) {
        let this_name = &obfuscate_name_ecs(&gs.ecs, entity).0;
        let this_width = 5 + this_name.len();
        width = if width > this_width { width } else { this_width };
        equippable.push((entity, this_name.to_string()));
    }

    let x = 1 + x_offset;
    let mut y = 3 + y_offset;

    ctx.draw_box(x, y, width, (count + 1) as i32, RGB::named(WHITE), RGB::named(BLACK));
    y += 1;

    let mut j = 0;
    let renderables = gs.ecs.read_storage::<Renderable>();
    for (e, name) in &equippable {
        let (mut fg, glyph) = if let Some(renderable) = renderables.get(*e) {
            (renderable.fg, renderable.glyph)
        } else {
            (RGB::named(WHITE), to_cp437('-'))
        };
        ctx.set(x + 1, y, RGB::named(YELLOW), RGB::named(BLACK), 97 + (j as FontCharType));
        ctx.set(x + 3, y, fg, RGB::named(BLACK), glyph);
        fg = RGB::named(item_colour_ecs(&gs.ecs, *e));
        ctx.print_color(x + 5, y, fg, RGB::named(BLACK), name);
        y += 1;
        j += 1;
    }

    match ctx.key {
        None => (ItemMenuResult::NoResponse, None),
        Some(key) =>
            match key {
                VirtualKeyCode::Escape => (ItemMenuResult::Cancel, None),
                _ => {
                    let selection = letter_to_option(key);
                    if selection > -1 && selection < (count as i32) {
                        return (ItemMenuResult::Selected, Some(equippable[selection as usize].0));
                    }
                    (ItemMenuResult::NoResponse, None)
                }
            }
    }
}

#[derive(PartialEq, Copy, Clone)]
pub enum TargetResult {
    Cancel,
    NoResponse {
        x: i32,
        y: i32,
    },
    Selected,
}

pub fn ranged_target(
    gs: &mut State,
    ctx: &mut BTerm,
    x: i32,
    y: i32,
    range: i32,
    aoe: i32
) -> (TargetResult, Option<Point>) {
    let bounds = camera::get_screen_bounds(&gs.ecs, false);
    let player_entity = gs.ecs.fetch::<Entity>();
    let player_pos = gs.ecs.fetch::<Point>();
    let viewsheds = gs.ecs.read_storage::<Viewshed>();

    ctx.print_color(
        1 + bounds.x_offset,
        1 + bounds.y_offset,
        RGB::named(WHITE),
        RGB::named(BLACK),
        "Targeting which tile? [mouse input]"
    );

    // Highlight available cells
    let mut available_cells = Vec::new();
    let visible = viewsheds.get(*player_entity);
    if let Some(visible) = visible {
        // We have a viewshed
        for idx in visible.visible_tiles.iter() {
            let distance = DistanceAlg::Pythagoras.distance2d(*player_pos, *idx);
            if distance <= (range as f32) {
                let screen_x = idx.x - bounds.min_x;
                let screen_y = idx.y - bounds.min_y;
                if
                    screen_x > 1 &&
                    screen_x < bounds.max_x - bounds.min_x - 1 &&
                    screen_y > 1 &&
                    screen_y < bounds.max_y - bounds.min_y - 1
                {
                    ctx.set_bg(
                        screen_x + bounds.x_offset,
                        screen_y + bounds.y_offset,
                        TARGETING_VALID_COL
                    );
                    available_cells.push(idx);
                }
            }
        }
    } else {
        return (TargetResult::Cancel, None);
    }

    // Draw mouse cursor
    let mouse_pos = (x, y);
    let bounds = camera::get_screen_bounds(&gs.ecs, false);
    let x = x.clamp(bounds.x_offset, bounds.x_offset - 1 + VIEWPORT_W);
    let y = y.clamp(bounds.y_offset, bounds.y_offset - 1 + VIEWPORT_H);

    let mut mouse_pos_adjusted = mouse_pos;
    mouse_pos_adjusted.0 += bounds.min_x - bounds.x_offset;
    mouse_pos_adjusted.1 += bounds.min_y - bounds.y_offset;
    let map = gs.ecs.fetch::<Map>();
    let mut valid_target = false;
    for idx in available_cells.iter() {
        if idx.x == mouse_pos_adjusted.0 && idx.y == mouse_pos_adjusted.1 {
            valid_target = true;
        }
    }
    let mut result = (TargetResult::NoResponse { x, y }, None);
    if valid_target {
        let path = line2d(
            LineAlg::Bresenham,
            Point::new(player_pos.x, player_pos.y),
            Point::new(mouse_pos_adjusted.0, mouse_pos_adjusted.1)
        );
        for (i, point) in path.iter().enumerate() {
            if i == 0 || i == path.len() - 1 {
                continue;
            }
            ctx.set(
                point.x + bounds.x_offset - bounds.min_x,
                point.y + bounds.y_offset - bounds.min_y,
                RGB::named(TARGETING_LINE_COL),
                RGB::named(TARGETING_VALID_COL),
                to_cp437('~')
            );
        }
        if aoe > 0 {
            // We adjust for camera position when getting FOV, but then we need to adjust back
            // when iterating through the tiles themselves, by taking away min_x/min_y.
            let mut blast_tiles = field_of_view(
                Point::new(mouse_pos_adjusted.0, mouse_pos_adjusted.1),
                aoe,
                &*map
            );
            blast_tiles.retain(
                |p| p.x > 0 && p.x < map.width - 1 && p.y > 0 && p.y < map.height - 1
            );
            for tile in blast_tiles.iter() {
                let bg = if available_cells.contains(&tile) {
                    let col1 = TARGETING_AOE_COL;
                    let col2 = TARGETING_VALID_COL;
                    ((col1.0 + col2.0) / 2, (col1.1 + col2.1) / 2, (col1.2 + col2.2) / 2)
                } else {
                    let col1 = TARGETING_AOE_COL;
                    let col2 = BLACK;
                    ((col1.0 + col2.0) / 2, (col1.1 + col2.1) / 2, (col1.2 + col2.2) / 2)
                };
                ctx.set_bg(
                    tile.x - bounds.min_x + bounds.x_offset,
                    tile.y - bounds.min_y + bounds.y_offset,
                    bg
                );
            }
        }

        ctx.set_bg(mouse_pos.0, mouse_pos.1, RGB::named(TARGETING_CURSOR_COL));
        result = match ctx.key {
            None => result,
            Some(key) =>
                match key {
                    VirtualKeyCode::Return => {
                        return (
                            TargetResult::Selected,
                            Some(Point::new(mouse_pos_adjusted.0, mouse_pos_adjusted.1)),
                        );
                    }
                    _ => result,
                }
        };
    } else {
        ctx.set_bg(mouse_pos.0, mouse_pos.1, RGB::named(RED));
    }

    result = match ctx.key {
        None => result,
        Some(key) =>
            match key {
                VirtualKeyCode::Escape => (TargetResult::Cancel, None),
                VirtualKeyCode::Numpad9 => (TargetResult::NoResponse { x: x + 1, y: y - 1 }, None),
                VirtualKeyCode::Numpad7 => (TargetResult::NoResponse { x: x - 1, y: y - 1 }, None),
                VirtualKeyCode::Numpad6 => (TargetResult::NoResponse { x: x + 1, y }, None),
                VirtualKeyCode::Numpad4 => (TargetResult::NoResponse { x: x - 1, y }, None),
                VirtualKeyCode::Numpad8 => (TargetResult::NoResponse { x, y: y - 1 }, None),
                VirtualKeyCode::Numpad3 => (TargetResult::NoResponse { x: x + 1, y: y + 1 }, None),
                VirtualKeyCode::Numpad2 => (TargetResult::NoResponse { x, y: y + 1 }, None),
                VirtualKeyCode::Numpad1 => (TargetResult::NoResponse { x: x - 1, y: y + 1 }, None),
                _ => result,
            }
    };
    return result;
}

#[derive(PartialEq, Copy, Clone)]
pub enum MainMenuSelection {
    NewGame,
    LoadGame,
    Quit,
}

#[derive(PartialEq, Copy, Clone)]
pub enum MainMenuResult {
    NoSelection {
        selected: MainMenuSelection,
    },
    Selected {
        selected: MainMenuSelection,
    },
}

pub fn main_menu(gs: &mut State, ctx: &mut BTerm) -> MainMenuResult {
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

    ctx.draw_box_double(x, y - 4, 13, height, RGB::named(WHITE), RGB::named(BLACK));
    ctx.print_color(x + 3, y - 2, RGB::named(YELLOW), RGB::named(BLACK), "RUST-RL!");

    if let RunState::MainMenu { menu_selection: selection } = *runstate {
        if save_exists {
            if selection == MainMenuSelection::LoadGame {
                ctx.print_color(x + 2, y, RGB::named(YELLOW), RGB::named(BLACK), "[");
                ctx.print_color(x + 3, y, RGB::named(GREEN), RGB::named(BLACK), "continue");
                ctx.print_color(x + 11, y, RGB::named(YELLOW), RGB::named(BLACK), "]");
            } else {
                ctx.print_color(x + 3, y, RGB::named(WHITE), RGB::named(BLACK), "continue");
            }
            y += 1;
        }
        if selection == MainMenuSelection::NewGame {
            ctx.print_color(x + 2, y, RGB::named(YELLOW), RGB::named(BLACK), "[");
            ctx.print_color(x + 3, y, RGB::named(GREEN), RGB::named(BLACK), "new game");
            ctx.print_color(x + 11, y, RGB::named(YELLOW), RGB::named(BLACK), "]");
        } else {
            ctx.print_color(x + 3, y, RGB::named(WHITE), RGB::named(BLACK), "new game");
        }
        y += 1;
        if selection == MainMenuSelection::Quit {
            ctx.print_color(x + 2, y, RGB::named(YELLOW), RGB::named(BLACK), "[");
            ctx.print_color(x + 3, y, RGB::named(GREEN), RGB::named(BLACK), "goodbye!");
            ctx.print_color(x + 11, y, RGB::named(YELLOW), RGB::named(BLACK), "]");
        } else {
            ctx.print_color(x + 5, y, RGB::named(WHITE), RGB::named(BLACK), "quit");
        }

        match ctx.key {
            None => {
                return MainMenuResult::NoSelection { selected: selection };
            }
            Some(key) =>
                match key {
                    VirtualKeyCode::Escape | VirtualKeyCode::C => {
                        return MainMenuResult::NoSelection { selected: MainMenuSelection::Quit };
                    }
                    VirtualKeyCode::N => {
                        return MainMenuResult::NoSelection { selected: MainMenuSelection::NewGame };
                    }
                    VirtualKeyCode::L => {
                        return MainMenuResult::NoSelection {
                            selected: MainMenuSelection::LoadGame,
                        };
                    }
                    VirtualKeyCode::Up | VirtualKeyCode::Numpad8 | VirtualKeyCode::K => {
                        let mut new_selection;
                        match selection {
                            MainMenuSelection::NewGame => {
                                new_selection = MainMenuSelection::LoadGame;
                            }
                            MainMenuSelection::LoadGame => {
                                new_selection = MainMenuSelection::Quit;
                            }
                            MainMenuSelection::Quit => {
                                new_selection = MainMenuSelection::NewGame;
                            }
                        }
                        if new_selection == MainMenuSelection::LoadGame && !save_exists {
                            new_selection = MainMenuSelection::NewGame;
                        }
                        return MainMenuResult::NoSelection { selected: new_selection };
                    }
                    VirtualKeyCode::Down | VirtualKeyCode::Numpad2 | VirtualKeyCode::J => {
                        let mut new_selection;
                        match selection {
                            MainMenuSelection::NewGame => {
                                new_selection = MainMenuSelection::Quit;
                            }
                            MainMenuSelection::LoadGame => {
                                new_selection = MainMenuSelection::NewGame;
                            }
                            MainMenuSelection::Quit => {
                                new_selection = MainMenuSelection::LoadGame;
                            }
                        }
                        if new_selection == MainMenuSelection::LoadGame && !save_exists {
                            new_selection = MainMenuSelection::Quit;
                        }
                        return MainMenuResult::NoSelection { selected: new_selection };
                    }
                    VirtualKeyCode::Return | VirtualKeyCode::NumpadEnter => {
                        return MainMenuResult::Selected { selected: selection };
                    }
                    _ => {
                        return MainMenuResult::NoSelection { selected: selection };
                    }
                }
        }
    }
    MainMenuResult::NoSelection { selected: MainMenuSelection::NewGame }
}

#[derive(PartialEq, Copy, Clone)]
pub enum YesNoResult {
    NoSelection,
    Yes,
    No,
}

pub fn game_over(ctx: &mut BTerm) -> YesNoResult {
    let mut x = 3;
    let mut y = 12;
    let width = 45;
    let height = 20;
    ctx.draw_box(x, y, width, height, RGB::named(WHITE), RGB::named(BLACK));
    ctx.print_color(x + 3, y, RGB::named(YELLOW), RGB::named(BLACK), "You died!");
    ctx.print_color(
        x + 3,
        y + height,
        RGB::named(YELLOW),
        RGB::named(BLACK),
        " Write a morgue file? [y/n] "
    );
    x += 2;
    y += 2;
    ctx.print_color(
        x,
        y,
        RGB::named(GREEN),
        RGB::named(BLACK),
        format!("You survived for {} turns.", crate::gamelog::get_event_count(EVENT::COUNT_TURN))
    );
    y += 2;
    ctx.print_color(x, y, RGB::named(GREEN), RGB::named(BLACK), format!("And in the process, you"));
    y += 1;
    if crate::gamelog::get_event_count(EVENT::COUNT_CHANGED_FLOOR) > 0 {
        ctx.print_color(
            x + 1,
            y,
            RGB::named(WHITE),
            RGB::named(BLACK),
            format!(
                "- changed floor {} times",
                crate::gamelog::get_event_count(EVENT::COUNT_CHANGED_FLOOR)
            )
        );
        y += 1;
    }
    if crate::gamelog::get_event_count(EVENT::COUNT_KICK) > 0 {
        ctx.print_color(
            x + 1,
            y,
            RGB::named(WHITE),
            RGB::named(BLACK),
            format!(
                "- kicked {} time(s), breaking {} object(s)",
                crate::gamelog::get_event_count(EVENT::COUNT_KICK),
                crate::gamelog::get_event_count(EVENT::COUNT_BROKE_DOOR)
            )
        );
        y += 1;
    }
    if crate::gamelog::get_event_count(EVENT::COUNT_KILLED) > 0 {
        ctx.print_color(
            x + 1,
            y,
            RGB::named(WHITE),
            RGB::named(BLACK),
            format!(
                "- slew {} other creature(s)",
                crate::gamelog::get_event_count(EVENT::COUNT_KILLED)
            )
        );
        y += 1;
    }
    if crate::gamelog::get_event_count(EVENT::COUNT_LOOKED_FOR_HELP) > 0 {
        ctx.print_color(
            x + 1,
            y,
            RGB::named(WHITE),
            RGB::named(BLACK),
            format!(
                "- forgot the controls {} time(s)",
                crate::gamelog::get_event_count(EVENT::COUNT_LOOKED_FOR_HELP)
            )
        );
    }

    match ctx.key {
        None => YesNoResult::NoSelection,
        Some(key) =>
            match key {
                VirtualKeyCode::N => YesNoResult::No,
                VirtualKeyCode::Y => YesNoResult::Yes,
                _ => YesNoResult::NoSelection,
            }
    }
}

pub fn with_article(name: &String) -> String {
    // If first letter is a capital
    if name.chars().nth(0).unwrap().is_uppercase() {
        return format!("{}", name);
    }
    // a/an
    let vowels = ['a', 'e', 'i', 'o', 'u'];
    if vowels.contains(&name.chars().nth(0).unwrap()) {
        return format!("an {}", name);
    }
    format!("a {}", name)
}
