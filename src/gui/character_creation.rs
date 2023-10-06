use super::{
    gamesystem::attr_bonus,
    gamesystem::get_attribute_rolls,
    gamesystem::mana_at_level,
    Attributes,
    Pools,
    Renderable,
    RunState,
    State,
};
use crate::consts::entity;
use crate::consts::char_create::*;
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
use bracket_lib::prelude::*;
use serde::{ Deserialize, Serialize };
use specs::prelude::*;
use std::collections::HashMap;
use crate::consts::prelude::*;

#[derive(Serialize, Deserialize, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Ancestry {
    Unset,
    Human,
    Dwarf,
    Gnome,
    Elf,
    Catfolk,
}

#[derive(Serialize, Deserialize, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Class {
    Unset,
    Fighter,
    Rogue,
    Wizard,
    Villager,
}

lazy_static! {
    static ref ANCESTRYDATA: HashMap<Ancestry, Vec<String>> = {
        let mut m = HashMap::new();
        m.insert(
            Ancestry::Human,
            vec![
                "nothing".to_string()]);
        m.insert(
            Ancestry::Dwarf,
            vec![
                "a natural bonus to defence".to_string()]);
        m.insert(
            Ancestry::Elf,
            vec![
                "minor telepathy".to_string(),
                "a slightly increased speed".to_string()]);
        m.insert(
            Ancestry::Catfolk,
            vec![
                "increased speed".to_string(),
                "increased unarmed damage".to_string()]);
        return m;
    };
}

lazy_static! {
    static ref CLASSDATA: HashMap<Class, Vec<String>> = {
        let mut m = HashMap::new();
        m.insert(
            Class::Fighter,
            vec![
                format!("a longsword, ring mail, and {} food", FIGHTER_STARTING_FOOD),
                "10 str, 8 dex, 10 con, 6 int, 6 wis, 8 cha".to_string(),
                "and 27 random stat points".to_string()]);
        m.insert(
            Class::Rogue,
            vec![
                format!("a rapier, leather armour, and {} food", ROGUE_STARTING_FOOD),
                "8 str, 10 dex, 8 con, 6 int, 8 wis, 10 cha".to_string(),
                "and 35 random stat points".to_string()]);
        m.insert(
            Class::Wizard,
            vec![
                format!("a dagger, random scrolls/potions, and {} food", WIZARD_STARTING_FOOD),
                "6 str, 8 dex, 6 con, 10 int, 10 wis, 8 cha".to_string(),
                "and 17 random stat points".to_string()]);
        m.insert(
            Class::Villager,
            vec![
                format!("the first weapon you could find, and {} food", VILLAGER_STARTING_FOOD),
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

use notan::prelude::*;
use notan::draw::{ Draw, DrawTextSection };
use super::{ FONTSIZE, TILESIZE };
use crate::consts::DISPLAYHEIGHT;
use crate::Fonts;

pub fn draw_charcreation(
    ecs: &World,
    draw: &mut Draw,
    atlas: &HashMap<String, Texture>,
    font: &Fonts
) {
    let runstate = ecs.read_resource::<RunState>();
    let (class, ancestry) = match *runstate {
        RunState::CharacterCreation { class, ancestry } => (class, ancestry),
        _ => unreachable!("draw_charcreation() called outside of CharacterCreation runstate."),
    };
    let (mut x, mut y) = (2.0 * TILESIZE.x, ((DISPLAYHEIGHT as f32) * TILESIZE.x) / 4.0);
    const COLUMN_WIDTH: f32 = 20.0 * TILESIZE.x;
    draw.text(&font.ib(), "Who are you?")
        .size(FONTSIZE * 2.0)
        .position(x, y)
        .h_align_left();
    y = draw.last_text_bounds().max_y();
    let initial_y = y;
    let ancestries = [
        ("h. Human", Ancestry::Human),
        ("e. Elf", Ancestry::Elf),
        ("d. Dwarf", Ancestry::Dwarf),
        ("c. Catfolk", Ancestry::Catfolk),
    ];
    for (k, v) in &ancestries {
        draw.text(font.n(), k)
            .size(FONTSIZE)
            .position(x, y)
            .h_align_left()
            .color(get_colour(ancestry, *v));
        y = draw.last_text_bounds().max_y();
    }
    y = initial_y;
    x += COLUMN_WIDTH;
    let classes = [
        ("f. Fighter", Class::Fighter),
        ("r. Rogue", Class::Rogue),
        ("w. Wizard", Class::Wizard),
        ("v. Villager", Class::Villager),
    ];
    for (k, v) in &classes {
        draw.text(font.n(), k)
            .size(FONTSIZE)
            .position(x, y)
            .h_align_left()
            .color(get_colour(class, *v));
        y = draw.last_text_bounds().max_y();
    }
    y = initial_y;
    x += COLUMN_WIDTH;
    for line in ANCESTRYDATA.get(&ancestry).unwrap().iter() {
        draw.text(font.n(), line).size(FONTSIZE).position(x, y).h_align_left();
        y = draw.last_text_bounds().max_y();
    }
    y += TILESIZE.x;
    for line in CLASSDATA.get(&class).unwrap().iter() {
        draw.text(font.n(), line).size(FONTSIZE).position(x, y).h_align_left();
        y = draw.last_text_bounds().max_y();
    }
}

fn get_colour<T>(selected: T, desired: T) -> Color where T: PartialEq {
    if selected == desired { Color::from_rgb(0.0, 1.0, 0.0) } else { Color::WHITE }
}

/// Handles the player character creation screen.
pub fn character_creation(gs: &mut State, ctx: &mut App) -> CharCreateResult {
    let runstate = gs.ecs.fetch::<RunState>();
    let (ancestry, class) = match *runstate {
        RunState::CharacterCreation { ancestry, class } => (ancestry, class),
        _ => unreachable!("character_creation() called outside of CharacterCreation runstate."),
    };

    let key = &ctx.keyboard;
    for keycode in key.pressed.iter() {
        match *keycode {
            KeyCode::Escape => {
                return CharCreateResult::Selected { ancestry: Ancestry::Unset, class };
            }
            KeyCode::Return => {
                return CharCreateResult::Selected { ancestry, class };
            }
            KeyCode::H => {
                return CharCreateResult::NoSelection { ancestry: Ancestry::Human, class };
            }
            KeyCode::E => {
                return CharCreateResult::NoSelection { ancestry: Ancestry::Elf, class };
            }
            KeyCode::D => {
                return CharCreateResult::NoSelection { ancestry: Ancestry::Dwarf, class };
            }
            KeyCode::C => {
                return CharCreateResult::NoSelection { ancestry: Ancestry::Catfolk, class };
            }
            KeyCode::F => {
                return CharCreateResult::NoSelection { ancestry, class: Class::Fighter };
            }
            KeyCode::R => {
                return CharCreateResult::NoSelection { ancestry, class: Class::Rogue };
            }
            KeyCode::W => {
                return CharCreateResult::NoSelection { ancestry, class: Class::Wizard };
            }
            KeyCode::V => {
                return CharCreateResult::NoSelection { ancestry, class: Class::Villager };
            }
            _ => {}
        };
    }
    return CharCreateResult::NoSelection { ancestry, class };
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
        skills
            .insert(*player, Skills { skills: HashMap::new() })
            .expect("Unable to insert skills component");
        skills.get_mut(*player).unwrap()
    };
    let mut ancestries = ecs.write_storage::<HasAncestry>();
    ancestries.insert(*player, HasAncestry { name: ancestry }).expect("Unable to insert ancestry");
    match ancestry {
        Ancestry::Human => {}
        Ancestry::Dwarf => {
            renderables
                .insert(
                    *player,
                    Renderable::new(
                        to_cp437(DWARF_GLYPH),
                        "gnome".to_string(),
                        RGB::named(DWARF_COLOUR),
                        2
                    )
                )
                .expect("Unable to insert renderable component");
            *player_skills.skills.entry(Skill::Defence).or_insert(0) += DWARF_DEFENCE_MOD;
        }
        Ancestry::Elf => {
            renderables
                .insert(
                    *player,
                    Renderable::new(
                        to_cp437(ELF_GLYPH),
                        "gnome".to_string(),
                        RGB::named(ELF_COLOUR),
                        2
                    )
                )
                .expect("Unable to insert renderable component");
            let mut telepaths = ecs.write_storage::<Telepath>();
            telepaths
                .insert(*player, Telepath {
                    telepath_tiles: Vec::new(),
                    range: ELF_TELEPATH_RANGE,
                    dirty: true,
                })
                .expect("Unable to insert telepath component");
            let mut speeds = ecs.write_storage::<Energy>();
            speeds
                .insert(*player, Energy {
                    current: 0,
                    speed: entity::NORMAL_SPEED + ELF_SPEED_BONUS,
                })
                .expect("Unable to insert energy component");
        }
        Ancestry::Catfolk => {
            renderables
                .insert(
                    *player,
                    Renderable::new(
                        to_cp437(CATFOLK_GLYPH),
                        "gnome".to_string(),
                        RGB::named(CATFOLK_COLOUR),
                        2
                    )
                )
                .expect("Unable to insert renderable component");
            let mut speeds = ecs.write_storage::<Energy>();
            speeds
                .insert(*player, Energy {
                    current: 0,
                    speed: entity::NORMAL_SPEED + CATFOLK_SPEED_BONUS,
                })
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
                    list: vec![KnownSpell { display_name: "zap".to_string(), mana_cost: 1 }],
                })
                .expect("Unable to insert known spells component");
        }
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();
        let mut attributes = ecs.write_storage::<Attributes>();
        let (str, dex, con, int, wis, cha) = get_attribute_rolls(&mut rng, class, ancestry);
        attributes
            .insert(player, Attributes::with_stats(str, dex, con, int, wis, cha))
            .expect("Unable to insert attributes component");

        let mut pools = ecs.write_storage::<Pools>();
        let starting_mp = mana_at_level(&mut rng, int, 1);
        pools
            .insert(player, Pools {
                hit_points: Pool {
                    current: 8 + attr_bonus(con),
                    max: entity::STANDARD_HIT_DIE + attr_bonus(con),
                },
                mana: Pool {
                    current: starting_mp,
                    max: starting_mp,
                },
                xp: 0,
                level: 1,
                bac: entity::STANDARD_BAC,
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

fn get_starting_inventory(
    class: Class,
    rng: &mut RandomNumberGenerator
) -> (Vec<String>, Vec<String>) {
    let mut equipped: Vec<String> = Vec::new();
    let mut carried: Vec<String> = Vec::new();
    let starting_food: &str;
    match class {
        Class::Unset => {
            starting_food = VILLAGER_STARTING_FOOD;
        }
        Class::Fighter => {
            starting_food = FIGHTER_STARTING_FOOD;
            equipped = vec![
                FIGHTER_STARTING_WEAPON.to_string(),
                FIGHTER_STARTING_ARMOUR.to_string(),
                FIGHTER_STARTING_SHIELD.to_string()
            ];
        }
        Class::Rogue => {
            starting_food = ROGUE_STARTING_FOOD;
            equipped = vec![ROGUE_STARTING_WEAPON.to_string(), ROGUE_STARTING_ARMOUR.to_string()];
            carried = vec!["equip_dagger".to_string(), "equip_dagger".to_string()];
        }
        Class::Wizard => {
            starting_food = WIZARD_STARTING_FOOD;
            equipped = vec![WIZARD_STARTING_WEAPON.to_string(), WIZARD_STARTING_ARMOUR.to_string()];
            pick_random_table_item(
                rng,
                &mut carried,
                "scrolls",
                WIZARD_SCROLL_AMOUNT,
                Some(WIZARD_MAX_SCROLL_LVL)
            );
            pick_random_table_item(
                rng,
                &mut carried,
                "potions",
                WIZARD_POTION_AMOUNT,
                Some(WIZARD_MAX_SCROLL_LVL)
            );
        }
        Class::Villager => {
            starting_food = VILLAGER_STARTING_FOOD;
            pick_random_table_item(rng, &mut equipped, "villager_equipment", "1d1", None);
        }
    }
    pick_random_table_item(rng, &mut carried, "food", starting_food, None);
    return (equipped, carried);
}

fn pick_random_table_item(
    rng: &mut RandomNumberGenerator,
    push_to: &mut Vec<String>,
    table: &'static str,
    dice_str: &'static str,
    difficulty: Option<i32>
) {
    let dice = parse_dice_string(dice_str).expect(
        format!("Error parsing dice: {}", dice_str).as_str()
    );
    for _i in 0..rng.roll_dice(dice.n_dice, dice.die_type) + dice.bonus {
        push_to.push(raws::table_by_name(&raws::RAWS.lock().unwrap(), table, difficulty).roll(rng));
    }
}
