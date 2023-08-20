use super::{gamesystem::attr_bonus, gamesystem::get_attribute_rolls, Attributes, Pools, Renderable, RunState, State};
use crate::{
    ai::NORMAL_SPEED, raws, spawner::potion_table, spawner::scroll_table, Attribute, Energy, Pool, Skill, Skills,
    Telepath,
};
use rltk::prelude::*;
use specs::prelude::*;
use std::collections::HashMap;

#[derive(PartialEq, Copy, Clone)]
pub enum Races {
    NULL,
    Human,
    Dwarf,
    Elf,
}

#[derive(PartialEq, Copy, Clone)]
pub enum Classes {
    Fighter,
    Rogue,
    Wizard,
    Villager,
}

#[derive(PartialEq, Copy, Clone)]
pub enum CharCreateResult {
    NoSelection { race: Races, class: Classes },
    Selected { race: Races, class: Classes },
}

/// Handles the player character creation screen.
pub fn character_creation(gs: &mut State, ctx: &mut Rltk) -> CharCreateResult {
    let runstate = gs.ecs.fetch::<RunState>();

    let mut x = 2;
    let mut y = 11;
    let column_width = 20;

    ctx.print_color(x, y, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK), "Who are you? [Aa-Zz]");
    y += 2;

    if let RunState::CharacterCreation { race, class } = *runstate {
        let selected_fg = RGB::named(GREEN);
        let unselected_fg = RGB::named(WHITE);
        let mut fg;
        let bg = RGB::named(BLACK);

        // Races
        if race == Races::Human {
            fg = selected_fg;
        } else {
            fg = unselected_fg;
        }
        ctx.print_color(x, y, fg, bg, "h. Human");
        if race == Races::Elf {
            fg = selected_fg;
        } else {
            fg = unselected_fg;
        }
        ctx.print_color(x, y + 1, fg, bg, "e. Elf");
        if race == Races::Dwarf {
            fg = selected_fg;
        } else {
            fg = unselected_fg;
        }
        ctx.print_color(x, y + 2, fg, bg, "d. Dwarf");
        // Classes
        x += column_width;
        if class == Classes::Fighter {
            fg = selected_fg;
        } else {
            fg = unselected_fg;
        }
        ctx.print_color(x, y, fg, bg, "f. Fighter");
        if class == Classes::Rogue {
            fg = selected_fg;
        } else {
            fg = unselected_fg;
        }
        ctx.print_color(x, y + 1, fg, bg, "r. Rogue");
        if class == Classes::Wizard {
            fg = selected_fg;
        } else {
            fg = unselected_fg;
        }
        ctx.print_color(x, y + 2, fg, bg, "w. Wizard");
        if class == Classes::Villager {
            fg = selected_fg;
        } else {
            fg = unselected_fg;
        }
        ctx.print_color(x, y + 3, fg, bg, "v. Villager");

        match ctx.key {
            None => return CharCreateResult::NoSelection { race, class },
            Some(key) => match key {
                VirtualKeyCode::Escape => return CharCreateResult::Selected { race: Races::NULL, class },
                VirtualKeyCode::Return => return CharCreateResult::Selected { race, class },
                VirtualKeyCode::H => return CharCreateResult::NoSelection { race: Races::Human, class },
                VirtualKeyCode::E => return CharCreateResult::NoSelection { race: Races::Elf, class },
                VirtualKeyCode::D => return CharCreateResult::NoSelection { race: Races::Dwarf, class },
                VirtualKeyCode::F => return CharCreateResult::NoSelection { race, class: Classes::Fighter },
                VirtualKeyCode::R => return CharCreateResult::NoSelection { race, class: Classes::Rogue },
                VirtualKeyCode::W => return CharCreateResult::NoSelection { race, class: Classes::Wizard },
                VirtualKeyCode::V => return CharCreateResult::NoSelection { race, class: Classes::Villager },
                _ => return CharCreateResult::NoSelection { race, class },
            },
        }
    }
    return CharCreateResult::NoSelection { race: Races::Human, class: Classes::Fighter };
}

/// Handles player race setup.
pub fn setup_player_race(ecs: &mut World, race: Races) {
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
    match race {
        Races::Human => {}
        Races::Dwarf => {
            renderables
                .insert(
                    *player,
                    Renderable {
                        glyph: rltk::to_cp437('h'),
                        fg: RGB::named(rltk::RED),
                        bg: RGB::named(rltk::BLACK),
                        render_order: 0,
                    },
                )
                .expect("Unable to insert renderable component");
            *player_skills.skills.entry(Skill::Defence).or_insert(0) += 1;
        }
        Races::Elf => {
            renderables
                .insert(
                    *player,
                    Renderable {
                        glyph: rltk::to_cp437('@'),
                        fg: RGB::named(rltk::GREEN),
                        bg: RGB::named(rltk::BLACK),
                        render_order: 0,
                    },
                )
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
        _ => {}
    }
}

/// Handles player class setup
pub fn setup_player_class(ecs: &mut World, class: Classes) {
    let player = *ecs.fetch::<Entity>();
    // ATTRIBUTES
    {
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();
        let mut attributes = ecs.write_storage::<Attributes>();

        let (str, dex, con, int, wis, cha) = get_attribute_rolls(&mut rng, class);
        attributes
            .insert(
                player,
                Attributes {
                    strength: Attribute { base: str, modifiers: 0, bonus: attr_bonus(str) },
                    dexterity: Attribute { base: dex, modifiers: 0, bonus: attr_bonus(dex) },
                    constitution: Attribute { base: con, modifiers: 0, bonus: attr_bonus(con) },
                    intelligence: Attribute { base: int, modifiers: 0, bonus: attr_bonus(int) },
                    wisdom: Attribute { base: wis, modifiers: 0, bonus: attr_bonus(wis) },
                    charisma: Attribute { base: cha, modifiers: 0, bonus: attr_bonus(cha) },
                },
            )
            .expect("Unable to insert attributes component");

        let mut pools = ecs.write_storage::<Pools>();
        pools
            .insert(
                player,
                Pools {
                    hit_points: Pool { current: 10 + attr_bonus(con), max: 10 + attr_bonus(con) },
                    mana: Pool { current: 2 + attr_bonus(int), max: 2 + attr_bonus(int) },
                    xp: 0,
                    level: 1,
                    bac: 10,
                    weight: 0.0,
                    god: false,
                },
            )
            .expect("Unable to insert pools component");
    }

    // TODO: use seeded RNG here
    let mut rng = RandomNumberGenerator::new();
    let starts_with = get_starting_inventory(class, &mut rng);
    for item in starts_with.0.iter() {
        raws::spawn_named_entity(&raws::RAWS.lock().unwrap(), ecs, item, raws::SpawnType::Equipped { by: player }, 0);
    }
    for item in starts_with.1.iter() {
        raws::spawn_named_entity(&raws::RAWS.lock().unwrap(), ecs, item, raws::SpawnType::Carried { by: player }, 0);
    }
}

fn get_starting_inventory(class: Classes, rng: &mut RandomNumberGenerator) -> (Vec<String>, Vec<String>) {
    let mut equipped: Vec<String> = Vec::new();
    let mut carried: Vec<String> = Vec::new();
    match class {
        Classes::Fighter => {
            equipped = vec![
                "equip_shortsword".to_string(),
                "equip_body_ringmail".to_string(),
                "equip_mediumshield".to_string(),
            ];
            carried = vec!["food_rations".to_string()];
        }
        Classes::Rogue => {
            equipped = vec!["equip_rapier".to_string(), "equip_body_leather".to_string()];
            carried = vec![
                "equip_dagger".to_string(),
                "equip_dagger".to_string(),
                "food_rations".to_string(),
                "food_apple".to_string(),
            ];
        }
        Classes::Wizard => {
            equipped = vec!["equip_dagger".to_string(), "equip_back_protection".to_string()];
            carried = vec!["food_rations".to_string()];
            for _i in 0..rng.roll_dice(1, 3) {
                carried.push(scroll_table(3).roll(rng));
            }
            for _i in 0..rng.roll_dice(1, 2) - 1 {
                carried.push(potion_table(3).roll(rng));
            }
        }
        Classes::Villager => {
            let rolled_weapon = raws::table_by_name(&raws::RAWS.lock().unwrap(), "villager_equipment", 1).roll(rng);
            equipped.push(rolled_weapon);
            carried = vec!["food_rations".to_string(), "food_apple".to_string(), "food_apple".to_string()];
        }
    }
    return (equipped, carried);
}
