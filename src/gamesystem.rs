use super::{Skill, Skills};

pub fn attr_bonus(value: i32) -> i32 {
    return (value - 10) / 2;
}

pub fn player_hp_per_level(rng: &mut rltk::RandomNumberGenerator, constitution: i32) -> i32 {
    return rng.roll_dice(1, 8) + attr_bonus(constitution);
}

pub fn player_hp_at_level(rng: &mut rltk::RandomNumberGenerator, constitution: i32, level: i32) -> i32 {
    let mut total = 10 + attr_bonus(constitution);
    for _i in 0..level {
        total += player_hp_per_level(rng, constitution);
    }
    return total;
}

pub fn npc_hp(rng: &mut rltk::RandomNumberGenerator, constitution: i32, level: i32) -> i32 {
    if level == 0 {
        return rng.roll_dice(1, 4);
    }
    let mut total = 1;
    for _i in 0..level {
        total += rng.roll_dice(1, 8) + attr_bonus(constitution);
    }
    return total;
}

pub fn mana_per_level(rng: &mut rltk::RandomNumberGenerator, intelligence: i32) -> i32 {
    return rng.roll_dice(1, 4) + attr_bonus(intelligence);
}

pub fn mana_at_level(rng: &mut rltk::RandomNumberGenerator, intelligence: i32, level: i32) -> i32 {
    let mut total = 12;
    for _i in 0..level {
        total += mana_per_level(rng, intelligence);
    }
    return total;
}

pub fn skill_bonus(skill: Skill, skills: &Skills) -> i32 {
    if skills.skills.contains_key(&skill) {
        return skills.skills[&skill];
    } else {
        return -4;
    }
}

pub fn roll_4d6(rng: &mut rltk::RandomNumberGenerator) -> i32 {
    let mut rolls: Vec<i32> = Vec::new();
    for _i in 0..4 {
        rolls.push(rng.roll_dice(1, 6));
    }
    rolls.sort_unstable();

    let mut roll = 0;
    rltk::console::log(format!("roll 0"));
    for i in 1..rolls.len() {
        roll += rolls[i];
        rltk::console::log(format!("+ {}", rolls[i]));
    }

    return roll;
}
