use super::{ Skill, Skills };
use crate::gui::{ Ancestry, Class };
use crate::consts::entity;
use crate::consts::char_create::*;
use bracket_lib::prelude::*;
use std::cmp::max;

/// Returns the attribute bonus for a given attribute score, where every 2 points above
/// or below 10 is an additional +1 or -1.
pub fn attr_bonus(value: i32) -> i32 {
    return (value - entity::ATTR_BONUS_0) / entity::ATTR_NEEDED_PER_POINT;
}

/// Returns the number of HP gained per level for a given constitution score.
pub fn hp_per_level(rng: &mut RandomNumberGenerator, constitution: i32) -> i32 {
    return max(rng.roll_dice(1, entity::STANDARD_HIT_DIE) + attr_bonus(constitution), 1);
}

#[allow(dead_code)]
/// Returns a total HP roll for a player, based on a given constitution score and level.
pub fn player_hp_at_level(rng: &mut RandomNumberGenerator, constitution: i32, level: i32) -> i32 {
    let mut total = entity::STANDARD_HIT_DIE + attr_bonus(constitution);
    for _i in 0..level {
        total += hp_per_level(rng, constitution);
    }
    return total;
}

/// Returns a total HP roll for an NPC, based on a given constitution score and level.
pub fn npc_hp_at_level(rng: &mut RandomNumberGenerator, constitution: i32, level: i32) -> i32 {
    if level == 0 {
        return rng.roll_dice(1, entity::STANDARD_HIT_DIE_0);
    }
    let mut total = 1;
    for _i in 0..level {
        total += rng.roll_dice(1, entity::STANDARD_HIT_DIE) + attr_bonus(constitution);
    }
    return total;
}

/// Returns the number of mana gained per level for a given intelligence score.
pub fn mana_per_level(rng: &mut RandomNumberGenerator, intelligence: i32) -> i32 {
    return max(rng.roll_dice(1, entity::STANDARD_MANA_DIE) + attr_bonus(intelligence), 1);
}

/// Returns the number of mana gained per level for a given intelligence score.
pub fn mana_at_level(rng: &mut RandomNumberGenerator, intelligence: i32, level: i32) -> i32 {
    let mut total = entity::MINIMUM_MANA;
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
pub fn roll_4d6(rng: &mut RandomNumberGenerator) -> i32 {
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
    ancestry: Ancestry
) -> (i32, i32, i32, i32, i32, i32) {
    let (mut str, mut dex, mut con, mut int, mut wis, mut cha) = match class {
        Class::Unset => VILLAGER_MIN_ATTR,
        Class::Fighter => FIGHTER_MIN_ATTR,
        Class::Rogue => ROGUE_MIN_ATTR,
        Class::Wizard => WIZARD_MIN_ATTR,
        Class::Villager => VILLAGER_MIN_ATTR,
    };
    let mut remaining_points = TOTAL_ATTRIBUTE_POINTS_MAXIMUM - (str + dex + con + int + wis + cha);
    let improve_chance: [i32; 6] = match class {
        Class::Unset => VILLAGER_IMPR_CHANCE,
        Class::Fighter => FIGHTER_IMPR_CHANCE,
        Class::Rogue => ROGUE_IMPR_CHANCE,
        Class::Wizard => WIZARD_IMPR_CHANCE,
        Class::Villager => VILLAGER_IMPR_CHANCE,
    };
    let ancestry_maximums: [i32; 6] = match ancestry {
        Ancestry::Human => HUMAN_MAX_ATTR, // 114
        Ancestry::Elf => ELF_MAX_ATTR, // 106
        Ancestry::Dwarf => DWARF_MAX_ATTR, // 106
        Ancestry::Gnome => GNOME_MAX_ATTR, // 106
        Ancestry::Catfolk => CATFOLK_MAX_ATTR, // 106
        _ => UNKNOWN_MAX_ATTR,
    };
    let improve_table = crate::random_table::RandomTable
        ::new()
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
    return (str, dex, con, int, wis, cha);
}
