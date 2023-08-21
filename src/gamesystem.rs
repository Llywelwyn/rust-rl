use super::{Skill, Skills};
use crate::gui::{Ancestry, Class};
use rltk::prelude::*;
use std::cmp::max;

/// Returns the attribute bonus for a given attribute score, where every 2 points above
/// or below 10 is an additional +1 or -1.
pub fn attr_bonus(value: i32) -> i32 {
    return (value - 10) / 2;
}

/// Returns the number of HP gained per level for a given constitution score.
pub fn hp_per_level(rng: &mut rltk::RandomNumberGenerator, constitution: i32) -> i32 {
    return max(rng.roll_dice(1, 8) + attr_bonus(constitution), 1);
}

#[allow(dead_code)]
/// Returns a total HP roll for a player, based on a given constitution score and level.
pub fn player_hp_at_level(rng: &mut rltk::RandomNumberGenerator, constitution: i32, level: i32) -> i32 {
    let mut total = 10 + attr_bonus(constitution);
    for _i in 0..level {
        total += hp_per_level(rng, constitution);
    }
    return total;
}

/// Returns a total HP roll for an NPC, based on a given constitution score and level.
pub fn npc_hp_at_level(rng: &mut rltk::RandomNumberGenerator, constitution: i32, level: i32) -> i32 {
    if level == 0 {
        return rng.roll_dice(1, 4);
    }
    let mut total = 1;
    for _i in 0..level {
        total += rng.roll_dice(1, 8) + attr_bonus(constitution);
    }
    return total;
}

/// Returns the number of mana gained per level for a given intelligence score.
pub fn mana_per_level(rng: &mut rltk::RandomNumberGenerator, intelligence: i32) -> i32 {
    return max(rng.roll_dice(1, 4) + attr_bonus(intelligence), 1);
}

/// Returns the number of mana gained per level for a given intelligence score.
pub fn mana_at_level(rng: &mut rltk::RandomNumberGenerator, intelligence: i32, level: i32) -> i32 {
    let mut total = 0;
    for _i in 0..level {
        total += mana_per_level(rng, intelligence);
    }
    return total;
}

/// Returns the skill bonus for a given skill, or -4 if the skill is not present.
pub fn skill_bonus(skill: Skill, skills: &Skills) -> i32 {
    if skills.skills.contains_key(&skill) {
        return skills.skills[&skill];
    } else {
        return -4;
    }
}

/// Roll 4d6 and drop the lowest, for rolling d20-style stats
#[allow(unused)]
pub fn roll_4d6(rng: &mut rltk::RandomNumberGenerator) -> i32 {
    let mut rolls: Vec<i32> = Vec::new();
    for _i in 0..4 {
        rolls.push(rng.roll_dice(1, 6));
    }
    rolls.sort_unstable();

    let mut roll = 0;
    for i in 1..rolls.len() {
        roll += rolls[i];
    }

    return roll;
}

/// Handles stat distribution for a player character.
pub fn get_attribute_rolls(
    rng: &mut RandomNumberGenerator,
    class: Class,
    ancestry: Ancestry,
) -> (i32, i32, i32, i32, i32, i32) {
    let (mut str, mut dex, mut con, mut int, mut wis, mut cha) = match class {
        Class::Fighter => (10, 8, 10, 6, 6, 8),
        Class::Rogue => (8, 10, 8, 6, 8, 10),
        Class::Wizard => (6, 8, 6, 10, 10, 8),
        Class::Villager => (6, 6, 6, 6, 6, 6),
    };
    let mut remaining_points = 75 - (str + dex + con + int + wis + cha);
    let improve_chance: [i32; 6] = match class {
        Class::Fighter => [30, 20, 30, 6, 7, 7],
        Class::Rogue => [18, 30, 20, 9, 8, 15],
        Class::Wizard => [10, 15, 20, 30, 15, 10],
        Class::Villager => [15, 15, 25, 15, 15, 15],
    };
    let ancestry_maximums: [i32; 6] = match ancestry {
        Ancestry::Human => [19, 19, 19, 19, 19, 19],   // 114
        Ancestry::Elf => [15, 18, 15, 20, 20, 18],     // 106
        Ancestry::Dwarf => [19, 17, 20, 16, 16, 16],   // 106
        Ancestry::Gnome => [16, 18, 16, 20, 18, 18],   // 106
        Ancestry::Catfolk => [16, 20, 16, 16, 18, 20], // 106
        _ => [18, 18, 18, 18, 18, 18],
    };
    let improve_table = crate::random_table::RandomTable::new()
        .add("Strength", improve_chance[0])
        .add("Dexterity", improve_chance[1])
        .add("Constitution", improve_chance[2])
        .add("Intelligence", improve_chance[3])
        .add("Wisdom", improve_chance[4])
        .add("Charisma", improve_chance[5]);
    let mut failed_attempts = 0;
    while remaining_points > 0 && failed_attempts < 100 {
        let roll = improve_table.roll(rng);
        match roll.as_str() {
            "Strength" => {
                if str < ancestry_maximums[0] {
                    str += 1;
                    remaining_points -= 1;
                } else {
                    failed_attempts += 1;
                }
            }
            "Dexterity" => {
                if dex < ancestry_maximums[1] {
                    dex += 1;
                    remaining_points -= 1;
                } else {
                    failed_attempts += 1;
                }
            }
            "Constitution" => {
                if con < ancestry_maximums[2] {
                    con += 1;
                    remaining_points -= 1;
                } else {
                    failed_attempts += 1;
                }
            }
            "Intelligence" => {
                if int < ancestry_maximums[3] {
                    int += 1;
                    remaining_points -= 1;
                } else {
                    failed_attempts += 1;
                }
            }
            "Wisdom" => {
                if wis < ancestry_maximums[4] {
                    wis += 1;
                    remaining_points -= 1;
                } else {
                    failed_attempts += 1;
                }
            }
            "Charisma" => {
                if cha < ancestry_maximums[5] {
                    cha += 1;
                    remaining_points -= 1;
                } else {
                    failed_attempts += 1;
                }
            }
            _ => {}
        }
    }
    console::log(format!("{}, {}", failed_attempts, remaining_points));
    return (str, dex, con, int, wis, cha);
}
