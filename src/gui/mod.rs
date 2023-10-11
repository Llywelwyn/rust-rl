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
    Key,
    Item,
    ItemType,
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
use crate::consts::{ TILESIZE, FONTSIZE, DISPLAYWIDTH };
use crate::Fonts;
use notan::prelude::*;
use notan::draw::{ Draw, DrawTextSection, DrawImages, DrawShapes };
use std::collections::HashMap;
use bracket_lib::prelude::*;
use specs::prelude::*;
use std::collections::{ BTreeMap, HashSet };
use crate::invkeys::check_key;

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
mod main_menu;
pub use main_menu::*;
mod inventory;
pub use inventory::*;

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

fn draw_bar_sprite(
    draw: &mut Draw,
    atlas: &HashMap<String, Texture>,
    sx: f32,
    y: f32,
    w: i32,
    n: i32,
    max: i32,
    sprite: &str
) {
    let percent = (n as f32) / (max as f32);
    let fill_width = (percent * (w as f32)) as i32;
    for x in 0..w {
        let suffix = if x == 0 { "1" } else if x == w - 1 { "3" } else { "2" };
        let fill = if x < fill_width { "full" } else { "empty" };
        let sprite = if let Some(sprite) = atlas.get(&format!("{}_{}_{}", sprite, fill, suffix)) {
            sprite
        } else {
            panic!("No sprite found in atlas: {}_{}_{}", sprite, fill, suffix)
        };
        draw.image(sprite).position(sx + (x as f32) * TILESIZE.x, y);
    }
}

pub const TEXT_FONT_MOD: i32 = 2;

pub fn draw_bar(
    draw: &mut notan::draw::Draw,
    x: f32,
    y: f32,
    width: f32, // Tiles
    height: f32, // Px
    current: i32,
    max: i32,
    full: Color,
    empty: Color
) {
    let fill: f32 = (f32::max(current as f32, 0.0) / (max as f32)) * width;
    draw.line((x * TILESIZE.x, y * TILESIZE.x), ((x + fill) * TILESIZE.x, y * TILESIZE.x))
        .color(full)
        .width(height);
    draw.line(((x + fill) * TILESIZE.x, y * TILESIZE.x), ((x + width) * TILESIZE.x, y * TILESIZE.x))
        .color(empty)
        .width(height);
}

pub fn draw_ui2(ecs: &World, draw: &mut Draw, atlas: &HashMap<String, Texture>, font: &Fonts) {
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
        const BAR_X: f32 = 1.0;
        const BAR_WIDTH: f32 = 22.0;
        draw_bar(
            draw,
            BAR_X,
            55.5,
            BAR_WIDTH,
            TILESIZE.x,
            stats.hit_points.current,
            stats.hit_points.max,
            Color::GREEN,
            Color::BLACK
        );
        draw_bar(
            draw,
            BAR_X,
            56.5,
            BAR_WIDTH,
            TILESIZE.x,
            stats.mana.current,
            stats.mana.max,
            Color::BLUE,
            Color::BLACK
        );
        let initial_x = 24.0 * TILESIZE.x;
        let mut x = initial_x;
        let row1 = 55.0 * TILESIZE.x;
        let row2 = row1 + TILESIZE.x;
        let hp_colours: (RGB, RGB, RGB) = (
            RGB::named(GREEN),
            RGB::named(RED),
            RGB::from_f32(0.0, 0.5, 0.0),
        );
        let mp_colours: (RGB, RGB, RGB) = (
            RGB::named(CYAN),
            RGB::named(RED),
            RGB::from_f32(0.0, 0.5, 0.5),
        );
        let hp: RGB = hp_colours.1.lerp(
            hp_colours.0,
            (stats.hit_points.current as f32) / (stats.hit_points.max as f32)
        );
        let mp: RGB = mp_colours.1.lerp(
            mp_colours.0,
            (stats.mana.current as f32) / (stats.mana.max as f32)
        );
        draw.text(&font.b(), "HP").position(x, row1).size(FONTSIZE);
        x = draw.last_text_bounds().max_x();
        draw.text(&font.n(), &format!("{}", stats.hit_points.current))
            .position(x, row1)
            .size(FONTSIZE)
            .color(Color::from_rgb(hp.r, hp.g, hp.b));
        x = draw.last_text_bounds().max_x();
        draw.text(&font.n(), &format!("({})", stats.hit_points.max))
            .position(x, row1)
            .size(FONTSIZE)
            .color(Color::from_rgb(hp_colours.2.r, hp_colours.2.g, hp_colours.2.b));
        x = initial_x;
        draw.text(&font.b(), "MP").position(x, row2).size(FONTSIZE);
        x = draw.last_text_bounds().max_x();
        draw.text(&font.n(), &format!("{}", stats.mana.current))
            .position(x, row2)
            .size(FONTSIZE)
            .color(Color::from_rgb(mp.r, mp.g, mp.b));
        x = draw.last_text_bounds().max_x();
        draw.text(&font.n(), &format!("({})", stats.mana.max))
            .position(x, row2)
            .size(FONTSIZE)
            .color(Color::from_rgb(mp_colours.2.r, mp_colours.2.g, mp_colours.2.b));
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
        x = draw.last_text_bounds().max_x() + 2.0 * TILESIZE.x;
        let armour_class =
            stats.bac - attributes.dexterity.modifier() / 2 - skill_ac_bonus - armour_ac_bonus;
        draw.text(&font.b(), "AC").position(x, row1).color(Color::PINK).size(FONTSIZE);
        let last_x = draw.last_text_bounds().max_x();
        draw.text(&font.n(), &format!("{:<2}", armour_class)).position(last_x, row1).size(FONTSIZE);
        draw.text(&font.b(), &format!("XP")).position(x, row2).size(FONTSIZE);
        let last_x = draw.last_text_bounds().max_x();
        draw.text(&font.n(), &format!("{}/{}", stats.level, stats.xp))
            .position(last_x, row2)
            .size(FONTSIZE);
        let attribute_x = draw.last_text_bounds().max_x() + 2.0 * TILESIZE.x;
        draw.text(&font.b(), "STR").position(attribute_x, row1).color(Color::RED).size(FONTSIZE);
        x = draw.last_text_bounds().max_x();
        draw.text(&font.n(), &format!("{:<2}", attributes.strength.base))
            .position(x, row1)
            .size(FONTSIZE);
        x = draw.last_text_bounds().max_x() + TILESIZE.x;
        draw.text(&font.b(), "DEX").position(x, row1).color(Color::GREEN).size(FONTSIZE);
        x = draw.last_text_bounds().max_x();
        draw.text(&font.n(), &format!("{:<2}", attributes.dexterity.base))
            .position(x, row1)
            .size(FONTSIZE);
        x = draw.last_text_bounds().max_x() + TILESIZE.x;
        draw.text(&font.b(), "CON").position(x, row1).color(Color::ORANGE).size(FONTSIZE);
        x = draw.last_text_bounds().max_x();
        draw.text(&font.n(), &format!("{:<2}", attributes.constitution.base))
            .position(x, row1)
            .size(FONTSIZE);
        draw.text(&font.b(), "INT").position(attribute_x, row2).color(Color::BLUE).size(FONTSIZE);
        x = draw.last_text_bounds().max_x();
        draw.text(&font.n(), &format!("{:<2}", attributes.intelligence.base))
            .position(x, row2)
            .size(FONTSIZE);
        x = draw.last_text_bounds().max_x() + TILESIZE.x;
        draw.text(&font.b(), "WIS").position(x, row2).color(Color::YELLOW).size(FONTSIZE);
        x = draw.last_text_bounds().max_x();
        draw.text(&font.n(), &format!("{:<2}", attributes.wisdom.base))
            .position(x, row2)
            .size(FONTSIZE);
        x = draw.last_text_bounds().max_x() + TILESIZE.x;
        draw.text(&font.b(), "CHA").position(x, row2).color(Color::PURPLE).size(FONTSIZE);
        x = draw.last_text_bounds().max_x();
        draw.text(&font.n(), &format!("{:<2}", attributes.charisma.base))
            .position(x, row2)
            .size(FONTSIZE);
        let hungertxt = match hunger.state {
            HungerState::Satiated => "Satiated",
            HungerState::Normal => "Normal",
            HungerState::Hungry => "Hungry",
            HungerState::Weak => "Weak",
            HungerState::Fainting => "Fainting",
            HungerState::Starving => "Starving",
        };
        match hunger.state {
            HungerState::Normal => {}
            _ => {
                let col = get_hunger_colour(hunger.state);
                draw.text(&font.n(), hungertxt)
                    .position(((VIEWPORT_W + 1) as f32) * TILESIZE.x, row1)
                    .color(Color::from_bytes(col.0, col.1, col.2, 255))
                    .size(FONTSIZE)
                    .h_align_right();
            }
        }
        let map = ecs.fetch::<Map>();
        let id = if map.depth > 0 {
            format!("{}{}", map.short_name, map.depth)
        } else {
            format!("{}", map.short_name)
        };
        draw.text(&font.n(), &id)
            .position(((VIEWPORT_W + 1) as f32) * TILESIZE.x, row2)
            .color(Color::WHITE) // get_local_col()
            .size(FONTSIZE)
            .h_align_right();
        let turns = crate::gamelog::get_event_count(EVENT::COUNT_TURN);
        x = draw.last_text_bounds().min_x() - TILESIZE.x;
        draw.text(&font.n(), &format!("T{}", turns))
            .position(x, row2)
            .color(Color::YELLOW)
            .size(FONTSIZE)
            .h_align_right();
        if let Some(burden) = burden.get(*player_entity) {
            use crate::BurdenLevel;
            let (text, colour) = match burden.level {
                BurdenLevel::Burdened => ("Burdened", RGB::named(BROWN1)),
                BurdenLevel::Strained => ("Strained", RGB::named(ORANGE)),
                BurdenLevel::Overloaded => ("Overloaded", RGB::named(RED)),
            };
            draw.text(&font.n(), &text)
                .position((VIEWPORT_W as f32) * TILESIZE.x, 50.0 * TILESIZE.x)
                .color(Color::from_rgb(colour.r, colour.g, colour.b))
                .size(FONTSIZE)
                .h_align_right();
        }
        if stats.god {
            draw.text(&font.n(), "--- GODMODE: ON ---")
                .position(20.0 * TILESIZE.x, 20.0 * TILESIZE.x)
                .color(Color::YELLOW)
                .size(FONTSIZE);
        }
        // Equipment
        draw_all_items(ecs, draw, font, ((VIEWPORT_W + 3) as f32) * TILESIZE.x, TILESIZE.x);
        /*let renderables = ecs.read_storage::<Renderable>();
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
        // TODO: Fix all of this to work with notan colours, and sprites.
        if !equipment.is_empty() {
            draw.text(&font.b(), "Equipment")
                .position(((VIEWPORT_W + 3) as f32) * TILESIZE.x, (y as f32) * TILESIZE.x)
                .size(FONTSIZE);
            let mut j: u8 = 0;
            for item in equipment {
                y += 1;
                x = ((VIEWPORT_W + 3) as f32) * TILESIZE.x;
                draw.text(&font.b(), &format!("{} ", (97 + j) as char))
                    .position(x, (y as f32) * TILESIZE.x)
                    .color(Color::YELLOW)
                    .size(FONTSIZE);
                j += 1;
                x = draw.last_text_bounds().max_x();
                let mut col = item.2;
                draw.text(&font.n(), &format!("{} ", item.3 as u8 as char))
                    .position(x, (y as f32) * TILESIZE.x)
                    .size(FONTSIZE)
                    .color(Color::from_rgb(col.r, col.g, col.b)); // Colours here - and below.
                x = draw.last_text_bounds().max_x();
                col = item.1;
                draw.text(&font.n(), &item.0)
                    .position(x, (y as f32) * TILESIZE.x)
                    .size(FONTSIZE)
                    .color(Color::from_rgb(col.r, col.g, col.b));
                x = draw.last_text_bounds().max_x();
                draw.text(&font.n(), " (worn)")
                    .position(x, (y as f32) * TILESIZE.x)
                    .size(FONTSIZE);
            }
            y += 2;
        }
        // Backpack
        x = ((VIEWPORT_W + 3) as f32) * TILESIZE.x;
        draw.text(&font.b(), "Backpack")
            .position(x, (y as f32) * TILESIZE.x)
            .size(FONTSIZE);
        draw.text(
            &font.b(),
            &format!(
                "[{:.1}/{} lbs]",
                stats.weight,
                (attributes.strength.base + attributes.strength.bonuses) *
                    CARRY_CAPACITY_PER_STRENGTH
            )
        )
            .position(((DISPLAYWIDTH - 1) as f32) * TILESIZE.x, (y as f32) * TILESIZE.x)
            .size(FONTSIZE)
            .h_align_right();*/
        //let player_inventory = get_player_inventory(&ecs);
        // TODO: print_options()
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
            stats.bac - attributes.dexterity.modifier() / 2 - skill_ac_bonus - armour_ac_bonus;
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
                (attributes.strength.base + attributes.strength.bonuses) *
                    CARRY_CAPACITY_PER_STRENGTH
            )
        );
        y += 1;
        //let player_inventory = get_player_inventory(&ecs);
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
    /*gamelog::print_log(
        &mut BACKEND_INTERNAL.lock().consoles[TEXT_LAYER].console,
        Point::new(1 * TEXT_FONT_MOD, 7),
        false,
        7,
        (VIEWPORT_W - 1) * TEXT_FONT_MOD
    );*/

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
    ctx: &mut App,
    function: fn(i: i32, j: i32, ecs: &mut World) -> RunState
) -> RunState {
    let key = &ctx.keyboard;
    for keycode in key.pressed.iter() {
        match *keycode {
            KeyCode::Escape => {
                return RunState::AwaitingInput;
            }
            KeyCode::Numpad1 | KeyCode::B => {
                return function(-1, 1, ecs);
            }
            KeyCode::Numpad2 | KeyCode::J | KeyCode::Down => {
                return function(0, 1, ecs);
            }
            KeyCode::Numpad3 | KeyCode::N => {
                return function(1, 1, ecs);
            }
            KeyCode::Numpad4 | KeyCode::H | KeyCode::Left => {
                return function(-1, 0, ecs);
            }
            KeyCode::Numpad5 | KeyCode::Period => {
                return function(0, 0, ecs);
            }
            KeyCode::Numpad6 | KeyCode::L | KeyCode::Right => {
                return function(1, 0, ecs);
            }
            KeyCode::Numpad7 | KeyCode::Y => {
                return function(-1, -1, ecs);
            }
            KeyCode::Numpad8 | KeyCode::K | KeyCode::Up => {
                return function(0, -1, ecs);
            }
            KeyCode::Numpad9 | KeyCode::U => {
                return function(1, -1, ecs);
            }
            _ => {}
        }
    }
    RunState::ActionWithDirection { function }
}

#[derive(PartialEq, Copy, Clone)]
pub enum ItemMenuResult {
    Cancel,
    NoResponse,
    Selected,
}

pub fn print_options(
    ecs: &World,
    draw: &mut Draw,
    font: &Fonts,
    inventory: &PlayerInventory,
    mut x: f32,
    mut y: f32
) -> f32 {
    let initial_x: f32 = x;
    let mut sorted: Vec<_> = inventory.iter().collect();
    sorted.sort_by(|a, b| a.1.idx.cmp(&b.1.idx));
    for (info, slot) in sorted {
        x = initial_x;
        // Print the character required to access this item. i.e. (a)
        if slot.idx < 26 {
            draw.text(&font.b(), &format!("{} ", (97 + slot.idx) as u8 as char))
                .position(x, y)
                .color(Color::YELLOW)
                .size(FONTSIZE);
        } else {
            // If we somehow have more than 26, start using capitals
            draw.text(&font.b(), &format!("{} ", (65 - 26 + slot.idx) as u8 as char))
                .position(x, y)
                .color(Color::YELLOW)
                .size(FONTSIZE);
        }
        x = draw.last_text_bounds().max_x();
        let fg = RGB::from_u8(info.renderables.0, info.renderables.1, info.renderables.2);
        draw.text(&font.n(), &format!("{} ", info.glyph as u8 as char))
            .position(x, y)
            .size(FONTSIZE)
            .color(Color::from_rgb(fg.r, fg.g, fg.b));
        x = draw.last_text_bounds().max_x();

        let fg = RGB::from_u8(info.rgb.0, info.rgb.1, info.rgb.2);
        if slot.count > 1 {
            draw.text(&font.n(), &format!("{} {}", slot.count, info.display_name.plural))
                .position(x, y)
                .color(Color::from_rgb(fg.r, fg.g, fg.b))
                .size(FONTSIZE);
        } else {
            let prefix = if info.display_name.singular.to_lowercase().ends_with("s") {
                "some"
            } else if
                ['a', 'e', 'i', 'o', 'u']
                    .iter()
                    .any(|&v| info.display_name.singular.to_lowercase().starts_with(v))
            {
                "an"
            } else {
                "a"
            };
            draw.text(&font.n(), &format!("{} {}", prefix, info.display_name.singular))
                .position(x, y)
                .color(Color::from_rgb(fg.r, fg.g, fg.b))
                .size(FONTSIZE);
            if let Some(worn) = ecs.read_storage::<Equipped>().get(slot.item) {
                x = draw.last_text_bounds().max_x();
                use crate::EquipmentSlot;
                let text = match worn.slot {
                    EquipmentSlot::Melee | EquipmentSlot::Shield => "being held",
                    _ => "being worn",
                };
                draw.text(&font.b(), &format!(" ({})", text))
                    .position(x, y)
                    .color(Color::WHITE)
                    .size(FONTSIZE);
            };
        }
        y += TILESIZE.x;
    }
    return y;
}

pub fn get_max_inventory_width(inventory: &PlayerInventory) -> i32 {
    let mut width: i32 = 0;
    for (item, slot) in inventory {
        let mut this_width = 4; // The spaces before and after the character to select this item, etc.
        if slot.count <= 1 {
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
            this_width += slot.count.to_string().len() as i32; // i.e. "12".len
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
    if let Some(worn) = ecs.read_storage::<Equipped>().get(item) {
        if worn.owner == *ecs.fetch::<Entity>() {
            singular.insert_str(singular.len(), " (worn)");
            plural.insert_str(plural.len(), " (worn)");
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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct DisplayName {
    singular: String,
    plural: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UniqueInventoryItem {
    display_name: DisplayName,
    rgb: (u8, u8, u8),
    renderables: (u8, u8, u8),
    glyph: u16,
    beatitude_status: i32,
    name: String,
}

pub fn unique(
    entity: Entity,
    names: &ReadStorage<Name>,
    obfuscated_names: &ReadStorage<ObfuscatedName>,
    renderables: &ReadStorage<Renderable>,
    beatitudes: &ReadStorage<Beatitude>,
    magic_items: &ReadStorage<MagicItem>,
    charges: Option<&ReadStorage<Charges>>,
    dm: &MasterDungeonMap
) -> UniqueInventoryItem {
    let item_colour = item_colour(entity, beatitudes);
    let (singular, plural) = obfuscate_name(
        entity,
        names,
        magic_items,
        obfuscated_names,
        beatitudes,
        dm,
        charges
    );
    let (renderables, glyph) = if let Some(renderable) = renderables.get(entity) {
        (
            (
                (renderable.fg.r * 255.0) as u8,
                (renderable.fg.g * 255.0) as u8,
                (renderable.fg.b * 255.0) as u8,
            ),
            renderable.glyph,
        )
    } else {
        unreachable!("Item has no renderable component.")
    };
    let name = if let Some(name) = names.get(entity) {
        name
    } else {
        unreachable!("Item has no name component.")
    };
    let beatitude_status = if let Some(beatitude) = beatitudes.get(entity) {
        match beatitude.buc {
            BUC::Blessed => 1,
            BUC::Uncursed => 2,
            BUC::Cursed => 3,
        }
    } else {
        0
    };
    UniqueInventoryItem {
        display_name: DisplayName { singular: singular.clone(), plural },
        rgb: item_colour,
        renderables,
        glyph,
        beatitude_status,
        name: name.name.clone(),
    }
}

pub fn unique_ecs(ecs: &World, entity: Entity) -> UniqueInventoryItem {
    return unique(
        entity,
        &ecs.read_storage::<Name>(),
        &ecs.read_storage::<ObfuscatedName>(),
        &ecs.read_storage::<Renderable>(),
        &ecs.read_storage::<Beatitude>(),
        &ecs.read_storage::<MagicItem>(),
        Some(&ecs.read_storage::<Charges>()),
        &ecs.fetch::<MasterDungeonMap>()
    );
}

pub struct InventorySlot {
    pub item: Entity,
    pub count: i32,
    pub idx: usize,
}

pub type PlayerInventory = HashMap<UniqueInventoryItem, InventorySlot>;

pub enum Filter {
    All(Option<ItemType>),
    Backpack(Option<ItemType>),
    Equipped,
}

macro_rules! includeitem {
    ($inv:expr, $ecs:expr, $e:expr, $k:expr) => {
        $inv.entry(unique_ecs($ecs, $e))
            .and_modify(|slot| {
                slot.count += 1;
            })
            .or_insert(InventorySlot {
                item: $e,
                count: 1,
                idx: $k.idx,
            });
    };
}

pub fn items(ecs: &World, filter: Filter) -> HashMap<UniqueInventoryItem, InventorySlot> {
    let entities = ecs.entities();
    let keys = ecs.read_storage::<Key>();

    let mut inv: HashMap<UniqueInventoryItem, InventorySlot> = HashMap::new();

    match filter {
        Filter::All(itemtype) => {
            if itemtype.is_some() {
                let items = ecs.read_storage::<Item>();
                for (e, k, _i) in (&entities, &keys, &items)
                    .join()
                    .filter(|e| e.2.category == itemtype.unwrap()) {
                    includeitem!(inv, ecs, e, k);
                }
            } else {
                for (e, k) in (&entities, &keys).join() {
                    includeitem!(inv, ecs, e, k);
                }
            }
        }
        Filter::Backpack(itemtype) => {
            let backpack = ecs.read_storage::<InBackpack>();
            if itemtype.is_some() {
                let items = ecs.read_storage::<Item>();
                for (e, k, _i, _b) in (&entities, &keys, &items, &backpack)
                    .join()
                    .filter(|e| e.2.category == itemtype.unwrap()) {
                    includeitem!(inv, ecs, e, k);
                }
            } else {
                for (e, k, _b) in (&entities, &keys, &backpack).join() {
                    includeitem!(inv, ecs, e, k);
                }
            }
        }
        Filter::Equipped => {
            let equipped = ecs.read_storage::<Equipped>();
            for (e, k, _e) in (&entities, &keys, &equipped).join() {
                includeitem!(inv, ecs, e, k);
            }
        }
    }

    inv
}

pub fn show_inventory(gs: &mut State, ctx: &mut App) -> (ItemMenuResult, Option<Entity>) {
    let on_overmap = gs.ecs.fetch::<Map>().overmap;

    let key = &ctx.keyboard;
    for keycode in key.pressed.iter() {
        match *keycode {
            KeyCode::Escape => {
                return (ItemMenuResult::Cancel, None);
            }
            _ => {
                let shift = key.shift();
                let selection = if
                    let Some(key) = letter_to_option::letter_to_option(*keycode, shift)
                {
                    key
                } else {
                    continue;
                };
                if check_key(selection) {
                    if on_overmap {
                        gamelog::Logger::new().append("You can't use items on the overmap.").log();
                    } else {
                        // Get the first entity with a Key {} component that has an idx matching "selection".
                        let entities = gs.ecs.entities();
                        let keyed_items = gs.ecs.read_storage::<Key>();
                        let backpack = gs.ecs.read_storage::<InBackpack>();
                        for (e, key, _b) in (&entities, &keyed_items, &backpack).join() {
                            if key.idx == selection {
                                return (ItemMenuResult::Selected, Some(e));
                            }
                        }
                        // TODO: Probably some gamelog about not having the selected item?
                    }
                }
            }
        }
    }
    return (ItemMenuResult::NoResponse, None);
}

pub fn drop_item_menu(gs: &mut State, ctx: &mut App) -> (ItemMenuResult, Option<Entity>) {
    let on_overmap = gs.ecs.fetch::<Map>().overmap;

    let key = &ctx.keyboard;
    for keycode in key.pressed.iter() {
        match *keycode {
            KeyCode::Escape => {
                return (ItemMenuResult::Cancel, None);
            }
            _ => {
                let shift = key.shift();
                let selection = if
                    let Some(key) = letter_to_option::letter_to_option(*keycode, shift)
                {
                    key
                } else {
                    continue;
                };
                if check_key(selection) {
                    if on_overmap {
                        gamelog::Logger::new().append("You can't drop items on the overmap.").log();
                    } else {
                        // Get the first entity with a Key {} component that has an idx matching "selection".
                        let entities = gs.ecs.entities();
                        let keyed_items = gs.ecs.read_storage::<Key>();
                        let backpack = gs.ecs.read_storage::<InBackpack>();
                        for (e, key, _b) in (&entities, &keyed_items, &backpack).join() {
                            if key.idx == selection {
                                return (ItemMenuResult::Selected, Some(e));
                            }
                        }
                    }
                }
            }
        }
    }
    (ItemMenuResult::NoResponse, None)
}

pub fn remove_item_menu(gs: &mut State, ctx: &mut App) -> (ItemMenuResult, Option<Entity>) {
    let key = &ctx.keyboard;
    for keycode in key.pressed.iter() {
        match *keycode {
            KeyCode::Escape => {
                return (ItemMenuResult::Cancel, None);
            }
            _ => {
                let shift = key.shift();
                let selection = if
                    let Some(key) = letter_to_option::letter_to_option(*keycode, shift)
                {
                    key
                } else {
                    continue;
                };
                if check_key(selection) {
                    // Get the first entity with a Key {} component that has an idx matching "selection".
                    let entities = gs.ecs.entities();
                    let keyed_items = gs.ecs.read_storage::<Key>();
                    let equipped = gs.ecs.read_storage::<Equipped>();
                    for (e, key, _e) in (&entities, &keyed_items, &equipped).join() {
                        if key.idx == selection {
                            return (ItemMenuResult::Selected, Some(e));
                        }
                    }
                }
            }
        }
    }
    (ItemMenuResult::NoResponse, None)
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

pub fn draw_targeting(
    ecs: &World,
    draw: &mut Draw,
    atlas: &HashMap<String, Texture>,
    x: i32,
    y: i32,
    range: i32,
    aoe: i32
) {
    let bounds = camera::get_screen_bounds(ecs, false);
    let player_entity = ecs.fetch::<Entity>();
    let player_pos = ecs.fetch::<Point>();
    let viewsheds = ecs.read_storage::<Viewshed>();

    let mut needs_draw: HashMap<Point, u32> = HashMap::new();

    const AVAILABLE: u32 = 0b00000001;
    const AOE: u32 = 0b00000010;
    const LINE_TO_CURSOR: u32 = 0b00000100;
    const CURSOR: u32 = 0b00001000;
    const CURSOR_UNAVAILABLE: u32 = 0b00010000;

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
                    needs_draw.insert(
                        Point::new(screen_x + bounds.x_offset, screen_y + bounds.y_offset),
                        AVAILABLE
                    );
                    available_cells.push(idx);
                }
            }
        }
    }

    // Draw mouse cursor
    let mouse_pos = (x, y);
    let bounds = camera::get_screen_bounds(ecs, false);
    let mut mouse_pos_adjusted = mouse_pos;
    mouse_pos_adjusted.0 += bounds.min_x - bounds.x_offset;
    mouse_pos_adjusted.1 += bounds.min_y - bounds.y_offset;
    let map = ecs.fetch::<Map>();
    let mut valid_target = false;
    for idx in available_cells.iter() {
        if idx.x == mouse_pos_adjusted.0 && idx.y == mouse_pos_adjusted.1 {
            valid_target = true;
        }
    }
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
            *needs_draw
                .entry(
                    Point::new(
                        point.x + bounds.x_offset - bounds.min_x,
                        point.y + bounds.y_offset - bounds.min_y
                    )
                )
                .or_insert(0) |= LINE_TO_CURSOR;
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
                *needs_draw
                    .entry(
                        Point::new(
                            tile.x - bounds.min_x + bounds.x_offset,
                            tile.y - bounds.min_y + bounds.y_offset
                        )
                    )
                    .or_insert(0) |= AOE;
            }
        }
        *needs_draw.entry(Point::new(mouse_pos.0, mouse_pos.1)).or_insert(0) |= CURSOR;
    } else {
        *needs_draw.entry(Point::new(mouse_pos.0, mouse_pos.1)).or_insert(0) |= CURSOR_UNAVAILABLE;
    }

    for (k, v) in needs_draw {
        let pos = ((k.x as f32) * TILESIZE.x, (k.y as f32) * TILESIZE.x);
        let tex = atlas.get("217").unwrap();
        if (v & CURSOR_UNAVAILABLE) != 0 {
            draw.image(tex).position(pos.0, pos.1).alpha(0.5).color(Color::RED);
            continue;
        }
        if (v & AVAILABLE) != 0 {
            draw.image(tex).position(pos.0, pos.1).alpha(0.2).color(Color::WHITE);
        }
        if (v & CURSOR) != 0 {
            draw.image(tex).position(pos.0, pos.1).alpha(0.2).color(Color::WHITE);
        }
        if (v & AOE) != 0 {
            draw.image(tex).position(pos.0, pos.1).alpha(0.2).color(Color::YELLOW);
        }
        if (v & LINE_TO_CURSOR) != 0 {
            draw.image(tex).position(pos.0, pos.1).alpha(0.2).color(Color::WHITE);
        }
    }
}

pub fn ranged_target(
    gs: &mut State,
    ctx: &mut App,
    x: i32,
    y: i32,
    range: i32,
    _aoe: i32
) -> (TargetResult, Option<Point>) {
    let bounds = camera::get_screen_bounds(&gs.ecs, false);
    let x = x.clamp(bounds.x_offset, bounds.x_offset - 1 + VIEWPORT_W);
    let y = y.clamp(bounds.y_offset, bounds.y_offset - 1 + VIEWPORT_H);

    let key = &ctx.keyboard;
    for keycode in key.pressed.iter() {
        match *keycode {
            KeyCode::Escape => {
                return (TargetResult::Cancel, None);
            }
            KeyCode::Numpad1 => {
                return (TargetResult::NoResponse { x: x - 1, y: y + 1 }, None);
            }
            KeyCode::Numpad2 => {
                return (TargetResult::NoResponse { x, y: y + 1 }, None);
            }
            KeyCode::Numpad3 => {
                return (TargetResult::NoResponse { x: x + 1, y: y + 1 }, None);
            }
            KeyCode::Numpad4 => {
                return (TargetResult::NoResponse { x: x - 1, y }, None);
            }
            KeyCode::Numpad6 => {
                return (TargetResult::NoResponse { x: x + 1, y }, None);
            }
            KeyCode::Numpad7 => {
                return (TargetResult::NoResponse { x: x - 1, y: y - 1 }, None);
            }
            KeyCode::Numpad8 => {
                return (TargetResult::NoResponse { x, y: y - 1 }, None);
            }
            KeyCode::Numpad9 => {
                return (TargetResult::NoResponse { x: x + 1, y: y - 1 }, None);
            }
            KeyCode::Return => {
                let bounds = camera::get_screen_bounds(&gs.ecs, false);
                let player_entity = gs.ecs.fetch::<Entity>();
                let player_pos = gs.ecs.fetch::<Point>();
                let viewsheds = gs.ecs.read_storage::<Viewshed>();

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
                                available_cells.push(idx);
                            }
                        }
                    }
                }
                let mouse_pos = (x, y);
                let bounds = camera::get_screen_bounds(&gs.ecs, false);
                let mut mouse_pos_adjusted = mouse_pos;
                mouse_pos_adjusted.0 += bounds.min_x - bounds.x_offset;
                mouse_pos_adjusted.1 += bounds.min_y - bounds.y_offset;
                let mut valid_target = false;
                for idx in available_cells.iter() {
                    if idx.x == mouse_pos_adjusted.0 && idx.y == mouse_pos_adjusted.1 {
                        valid_target = true;
                    }
                }
                if valid_target {
                    return (
                        TargetResult::Selected,
                        Some(Point::new(mouse_pos_adjusted.0, mouse_pos_adjusted.1)),
                    );
                }
            }
            _ => {}
        }
    }
    (TargetResult::NoResponse { x, y }, None)
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

pub fn main_menu(gs: &mut State, ctx: &mut App) -> MainMenuResult {
    let save_exists = super::saveload_system::does_save_exist();
    let runstate = gs.ecs.fetch::<RunState>();
    let selection = match *runstate {
        RunState::MainMenu { menu_selection: sel } => sel,
        _ => MainMenuSelection::NewGame,
    };

    let key = &ctx.keyboard;
    for keycode in key.pressed.iter() {
        match *keycode {
            KeyCode::Escape | KeyCode::C => {
                return MainMenuResult::NoSelection { selected: MainMenuSelection::Quit };
            }
            KeyCode::N => {
                return MainMenuResult::NoSelection { selected: MainMenuSelection::NewGame };
            }
            KeyCode::L => {
                return MainMenuResult::NoSelection { selected: MainMenuSelection::LoadGame };
            }
            KeyCode::Down | KeyCode::Numpad2 | KeyCode::J => {
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
            KeyCode::Up | KeyCode::Numpad8 | KeyCode::K => {
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
            KeyCode::Return | KeyCode::NumpadEnter => {
                return MainMenuResult::Selected { selected: selection };
            }
            _ => {}
        }
    }
    return MainMenuResult::NoSelection { selected: selection };
}

#[derive(PartialEq, Copy, Clone)]
pub enum YesNoResult {
    NoSelection,
    Yes,
    No,
}

pub fn game_over(ctx: &mut App) -> YesNoResult {
    for keycode in &ctx.keyboard.pressed {
        match *keycode {
            KeyCode::N => {
                return YesNoResult::No;
            }
            KeyCode::Y => {
                return YesNoResult::Yes;
            }
            _ => {}
        }
    }
    YesNoResult::NoSelection
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

/// Returns the map index of a tile in the viewport.
pub fn viewport_to_idx(ecs: &World, x: i32, y: i32) -> usize {
    let bounds = crate::camera::get_screen_bounds(ecs, false);
    let x = x + bounds.min_x;
    let y = y + bounds.min_y;
    return ecs.fetch::<Map>().xy_idx(x, y);
}

/// Denotes a pixel location on the screen.
pub struct Px {
    x: f32,
    y: f32,
}

impl Px {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

/// Returns the pixel location of a tile in the viewport.
pub fn viewport_to_px(x: i32, y: i32) -> Px {
    let offsets = crate::camera::get_offset();
    Px::new(
        (x as f32) * TILESIZE.sprite_x + (offsets.x as f32) * TILESIZE.x,
        (y as f32) * TILESIZE.sprite_y + (offsets.y as f32) * TILESIZE.y
    )
}
