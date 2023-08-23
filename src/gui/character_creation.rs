use super::{ gamesystem::attr_bonus, gamesystem::get_attribute_rolls, Attributes, Pools, Renderable, RunState, State };
use crate::config::entity::NORMAL_SPEED;
use crate::{
    raws,
    Attribute,
    Energy,
    HasAncestry,
    HasClass,
    KnownSpell,
    KnownSpells,
    Pool,
    Skill,
    Skills,
    Telepath,
    BUC,
};
use rltk::prelude::*;
use serde::{ Deserialize, Serialize };
use specs::prelude::*;
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Copy, Clone, PartialEq)]
pub enum Ancestry {
    NULL,
    Human,
    Dwarf,
    Gnome,
    Elf,
    Catfolk,
}

#[derive(Serialize, Deserialize, Copy, Clone, PartialEq)]
pub enum Class {
    Fighter,
    Rogue,
    Wizard,
    Villager,
}

lazy_static! {
    static ref ANCESTRY_CLASS_DATA: HashMap<String, Vec<String>> = {
        let mut m = HashMap::new();
        // Ancestry
        m.insert(
            "human".to_string(),
            vec![
                "nothing".to_string()]);
        m.insert(
            "dwarf".to_string(),
            vec![
                "a natural bonus to defence".to_string()]);
        m.insert(
            "elf".to_string(),
            vec![
                "minor telepathy".to_string(),
                "a slightly increased speed".to_string()]);
        m.insert(
            "catfolk".to_string(),
            vec![
                "increased speed".to_string(),
                "increased unarmed damage".to_string()]);
        // Class
        m.insert(
            "fighter".to_string(),
            vec![
                "a longsword, ring mail, and 1d2+1 food".to_string(),
                "10 str, 8 dex, 10 con, 6 int, 6 wis, 8 cha".to_string(),
                "and 27 random stat points".to_string()]);
        m.insert(
            "rogue".to_string(),
            vec![
                "a rapier, leather armour, and 1d2+2 food".to_string(),
                "8 str, 10 dex, 8 con, 6 int, 8 wis, 10 cha".to_string(),
                "and 35 random stat points".to_string()]);
        m.insert(
            "wizard".to_string(),
            vec![
                "a dagger, random scrolls/potions, and 1d2+1 food".to_string(),
                "6 str, 8 dex, 6 con, 10 int, 10 wis, 8 cha".to_string(),
                "and 17 random stat points".to_string()]);
        m.insert(
            "villager".to_string(),
            vec![
                "the first weapon you could find, and 1d3+2 food".to_string(),
                "6 str, 6 dex, 6 con, 6 int, 6 wis, 6 cha".to_string(),
                "and 39 random stat points".to_string()]);
        return m;
    };
}

#[derive(PartialEq, Copy, Clone)]
pub enum CharCreateResult {
    NoSelection {
        ancestry: Ancestry,
        class: Class,
    },
    Selected {
        ancestry: Ancestry,
        class: Class,
    },
}

/// Handles the player character creation screen.
pub fn character_creation(gs: &mut State, ctx: &mut Rltk) -> CharCreateResult {
    let runstate = gs.ecs.fetch::<RunState>();

    let mut x = 2;
    let mut y = 11;
    let column_width = 20;

    ctx.print_color(x, y, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK), "Who are you? [Aa-Zz]");
    y += 2;

    if let RunState::CharacterCreation { ancestry, class } = *runstate {
        let selected_fg = RGB::named(GREEN);
        let unselected_fg = RGB::named(WHITE);
        let mut fg;
        let bg = RGB::named(BLACK);

        // Ancestry
        ctx.print_color(x, y, bg, unselected_fg, "Ancestry");
        ctx.print_color(x + column_width, y, bg, unselected_fg, "Class");
        y += 1;
        let mut race_str = "human";
        if ancestry == Ancestry::Human {
            fg = selected_fg;
        } else {
            fg = unselected_fg;
        }
        ctx.print_color(x, y, fg, bg, "h. Human");
        if ancestry == Ancestry::Elf {
            fg = selected_fg;
            race_str = "elf";
        } else {
            fg = unselected_fg;
        }
        ctx.print_color(x, y + 1, fg, bg, "e. Elf");
        if ancestry == Ancestry::Dwarf {
            fg = selected_fg;
            race_str = "dwarf";
        } else {
            fg = unselected_fg;
        }
        ctx.print_color(x, y + 2, fg, bg, "d. Dwarf");
        if ancestry == Ancestry::Catfolk {
            fg = selected_fg;
            race_str = "catfolk";
        } else {
            fg = unselected_fg;
        }
        ctx.print_color(x, y + 3, fg, bg, "c. Catfolk");
        // Class
        let mut class_str = "fighter";
        x += column_width;
        if class == Class::Fighter {
            fg = selected_fg;
        } else {
            fg = unselected_fg;
        }
        ctx.print_color(x, y, fg, bg, "f. Fighter");
        if class == Class::Rogue {
            fg = selected_fg;
            class_str = "rogue";
        } else {
            fg = unselected_fg;
        }
        ctx.print_color(x, y + 1, fg, bg, "r. Rogue");
        if class == Class::Wizard {
            fg = selected_fg;
            class_str = "wizard";
        } else {
            fg = unselected_fg;
        }
        ctx.print_color(x, y + 2, fg, bg, "w. Wizard");
        if class == Class::Villager {
            fg = selected_fg;
            class_str = "villager";
        } else {
            fg = unselected_fg;
        }
        ctx.print_color(x, y + 3, fg, bg, "v. Villager");
        // Selected ancestry/class benefits
        x += column_width;
        ctx.print_color(x, y, selected_fg, bg, "Your ancestry grants...");
        for line in ANCESTRY_CLASS_DATA.get(race_str).unwrap().iter() {
            y += 1;
            ctx.print_color(x + 1, y, unselected_fg, bg, line);
        }
        y += 2;
        ctx.print_color(x, y, selected_fg, bg, "Your class grants...");
        for line in ANCESTRY_CLASS_DATA.get(class_str).unwrap().iter() {
            y += 1;
            ctx.print_color(x + 1, y, unselected_fg, bg, line);
        }

        match ctx.key {
            None => {
                return CharCreateResult::NoSelection { ancestry, class };
            }
            Some(key) =>
                match key {
                    VirtualKeyCode::Escape => {
                        return CharCreateResult::Selected { ancestry: Ancestry::NULL, class };
                    }
                    VirtualKeyCode::Return => {
                        return CharCreateResult::Selected { ancestry, class };
                    }
                    VirtualKeyCode::H => {
                        return CharCreateResult::NoSelection { ancestry: Ancestry::Human, class };
                    }
                    VirtualKeyCode::E => {
                        return CharCreateResult::NoSelection { ancestry: Ancestry::Elf, class };
                    }
                    VirtualKeyCode::D => {
                        return CharCreateResult::NoSelection { ancestry: Ancestry::Dwarf, class };
                    }
                    VirtualKeyCode::C => {
                        return CharCreateResult::NoSelection { ancestry: Ancestry::Catfolk, class };
                    }
                    VirtualKeyCode::F => {
                        return CharCreateResult::NoSelection { ancestry, class: Class::Fighter };
                    }
                    VirtualKeyCode::R => {
                        return CharCreateResult::NoSelection { ancestry, class: Class::Rogue };
                    }
                    VirtualKeyCode::W => {
                        return CharCreateResult::NoSelection { ancestry, class: Class::Wizard };
                    }
                    VirtualKeyCode::V => {
                        return CharCreateResult::NoSelection { ancestry, class: Class::Villager };
                    }
                    _ => {
                        return CharCreateResult::NoSelection { ancestry, class };
                    }
                }
        }
    }
    return CharCreateResult::NoSelection { ancestry: Ancestry::Human, class: Class::Fighter };
}

/// Handles player ancestry setup.
pub fn setup_player_ancestry(ecs: &mut World, ancestry: Ancestry) {
    let player = ecs.fetch::<Entity>();
    let mut renderables = ecs.write_storage::<Renderable>();
    // SKILLS
    let mut skills = ecs.write_storage::<Skills>();
    let player_skills = if let Some(skills) = skills.get_mut(*player) {
        skills
    } else {
        skills.insert(*player, Skills { skills: HashMap::new() }).expect("Unable to insert skills component");
        skills.get_mut(*player).unwrap()
    };
    let mut ancestries = ecs.write_storage::<HasAncestry>();
    ancestries.insert(*player, HasAncestry { name: ancestry }).expect("Unable to insert ancestry");
    match ancestry {
        Ancestry::Human => {}
        Ancestry::Dwarf => {
            renderables
                .insert(*player, Renderable {
                    glyph: rltk::to_cp437('h'),
                    fg: RGB::named(rltk::RED),
                    bg: RGB::named(rltk::BLACK),
                    render_order: 0,
                })
                .expect("Unable to insert renderable component");
            *player_skills.skills.entry(Skill::Defence).or_insert(0) += 1;
        }
        Ancestry::Elf => {
            renderables
                .insert(*player, Renderable {
                    glyph: rltk::to_cp437('@'),
                    fg: RGB::named(rltk::GREEN),
                    bg: RGB::named(rltk::BLACK),
                    render_order: 0,
                })
                .expect("Unable to insert renderable component");
            let mut telepaths = ecs.write_storage::<Telepath>();
            telepaths
                .insert(*player, Telepath { telepath_tiles: Vec::new(), range: 6, dirty: true })
                .expect("Unable to insert telepath component");
            let mut speeds = ecs.write_storage::<Energy>();
            speeds
                .insert(*player, Energy { current: 0, speed: NORMAL_SPEED + 1 })
                .expect("Unable to insert energy component");
        }
        Ancestry::Catfolk => {
            let mut speeds = ecs.write_storage::<Energy>();
            speeds
                .insert(*player, Energy { current: 0, speed: NORMAL_SPEED + 2 })
                .expect("Unable to insert energy component");
        }
        _ => {}
    }
}

/// Handles player class setup
pub fn setup_player_class(ecs: &mut World, class: Class, ancestry: Ancestry) {
    let player = *ecs.fetch::<Entity>();
    // ATTRIBUTES
    {
        let mut classes = ecs.write_storage::<HasClass>();
        classes.insert(player, HasClass { name: class }).expect("Unable to insert class component");
        if class == Class::Wizard {
            let mut spells = ecs.write_storage::<KnownSpells>();
            spells
                .insert(player, KnownSpells {
                    spells: vec![KnownSpell { display_name: "zap".to_string(), mana_cost: 1 }],
                })
                .expect("Unable to insert known spells component");
        }
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();
        let mut attributes = ecs.write_storage::<Attributes>();
        let (str, dex, con, int, wis, cha) = get_attribute_rolls(&mut rng, class, ancestry);
        attributes
            .insert(player, Attributes {
                strength: Attribute { base: str, modifiers: 0, bonus: attr_bonus(str) },
                dexterity: Attribute { base: dex, modifiers: 0, bonus: attr_bonus(dex) },
                constitution: Attribute { base: con, modifiers: 0, bonus: attr_bonus(con) },
                intelligence: Attribute { base: int, modifiers: 0, bonus: attr_bonus(int) },
                wisdom: Attribute { base: wis, modifiers: 0, bonus: attr_bonus(wis) },
                charisma: Attribute { base: cha, modifiers: 0, bonus: attr_bonus(cha) },
            })
            .expect("Unable to insert attributes component");

        let mut pools = ecs.write_storage::<Pools>();
        pools
            .insert(player, Pools {
                hit_points: Pool { current: 8 + attr_bonus(con), max: 8 + attr_bonus(con) },
                mana: Pool { current: 1 + attr_bonus(int), max: 1 + attr_bonus(int) },
                xp: 0,
                level: 1,
                bac: 10,
                weight: 0.0,
                god: false,
            })
            .expect("Unable to insert pools component");
    }

    // TODO: use seeded RNG here
    let mut rng = RandomNumberGenerator::new();
    let starts_with = get_starting_inventory(class, &mut rng);
    for item in starts_with.0.iter() {
        let buc = if rng.roll_dice(1, 3) == 1 { Some(BUC::Blessed) } else { Some(BUC::Uncursed) };
        raws::spawn_named_entity(
            &raws::RAWS.lock().unwrap(),
            ecs,
            item,
            buc,
            raws::SpawnType::Equipped { by: player },
            0
        );
    }
    for item in starts_with.1.iter() {
        raws::spawn_named_entity(
            &raws::RAWS.lock().unwrap(),
            ecs,
            item,
            None,
            raws::SpawnType::Carried { by: player },
            0
        );
    }
}

fn get_starting_inventory(class: Class, rng: &mut RandomNumberGenerator) -> (Vec<String>, Vec<String>) {
    let mut equipped: Vec<String> = Vec::new();
    let mut carried: Vec<String> = Vec::new();
    let starting_food: &str;
    match class {
        Class::Fighter => {
            starting_food = "1d2+1";
            equipped = vec![
                "equip_shortsword".to_string(),
                "equip_body_ringmail".to_string(),
                "equip_mediumshield".to_string()
            ];
        }
        Class::Rogue => {
            starting_food = "1d2+2";
            equipped = vec!["equip_rapier".to_string(), "equip_body_weakleather".to_string()];
            carried = vec!["equip_dagger".to_string(), "equip_dagger".to_string()];
        }
        Class::Wizard => {
            starting_food = "1d2+1";
            equipped = vec!["equip_dagger".to_string(), "equip_back_protection".to_string()];
            pick_random_table_item(rng, &mut carried, "scrolls", "1d3", Some(3));
            pick_random_table_item(rng, &mut carried, "potions", "1d3-1", Some(3));
        }
        Class::Villager => {
            starting_food = "1d3+2";
            pick_random_table_item(rng, &mut equipped, "villager_equipment", "1", None);
        }
    }
    pick_random_table_item(rng, &mut carried, "food", starting_food, None);
    return (equipped, carried);
}

fn pick_random_table_item(
    rng: &mut RandomNumberGenerator,
    push_to: &mut Vec<String>,
    table: &'static str,
    dice: &'static str,
    difficulty: Option<i32>
) {
    let (n, d, m) = raws::parse_dice_string(dice);
    for _i in 0..rng.roll_dice(n, d) + m {
        push_to.push(raws::table_by_name(&raws::RAWS.lock().unwrap(), table, difficulty).roll(rng));
    }
}
