use super::{
    effects::{add_effect, EffectType, Targets},
    gamelog, gamesystem,
    gui::renderable_colour,
    ArmourClassBonus, Attributes, Blind, EquipmentSlot, Equipped, HungerClock, HungerState, MeleeWeapon, MultiAttack,
    Name, NaturalAttacks, ParticleBuilder, Pools, Position, Renderable, Skill, Skills, ToHitBonus, WantsToMelee,
    WeaponAttribute,
};
use rltk::prelude::*;
use specs::prelude::*;

pub struct MeleeCombatSystem {}

impl<'a> System<'a> for MeleeCombatSystem {
    type SystemData = (
        Entities<'a>,
        ReadExpect<'a, Entity>,
        ReadStorage<'a, Renderable>,
        WriteStorage<'a, WantsToMelee>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, Attributes>,
        ReadStorage<'a, Skills>,
        ReadStorage<'a, Pools>,
        WriteExpect<'a, ParticleBuilder>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, Equipped>,
        ReadStorage<'a, MeleeWeapon>,
        ReadStorage<'a, NaturalAttacks>,
        ReadStorage<'a, ArmourClassBonus>,
        ReadStorage<'a, ToHitBonus>,
        ReadStorage<'a, HungerClock>,
        ReadStorage<'a, MultiAttack>,
        ReadStorage<'a, Blind>,
        WriteExpect<'a, rltk::RandomNumberGenerator>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            entities,
            player_entity,
            renderables,
            mut wants_melee,
            names,
            attributes,
            skills,
            pools,
            mut particle_builder,
            positions,
            equipped,
            melee_weapons,
            natural_attacks,
            ac,
            to_hit,
            hunger_clock,
            multi_attackers,
            blind_entities,
            mut rng,
        ) = data;

        //  Combat works with the older system of AC being a bonus to to-hit to the attacker. When an
        //  attacker tries to hit a victim, the attacker rolls a d20, and must roll *less than* the
        //  value of 10 + victim's AC + attacker's to-hit bonuses.
        //
        //  e.g. An attacker with +0 to-hit hitting a target with 10 AC:
        //       1d20 must be less than 20, 95% chance of a hit.
        //
        //  e.g. An attacker with +1 to-hit from being satiated hits a rat with 7 AC:
        //       1d20 must be less than 18 (10+7+1), 85% chance of a hit.
        //
        //  e.g. An attacker with +0 to-hit hitting a target with 0 AC:
        //       1d20 must be less than 10, 45% chance of a hit

        const COMBAT_LOGGING: bool = true;
        let mut logger = gamelog::Logger::new();
        let mut something_to_log = false;

        for (entity, wants_melee, name, attacker_attributes, attacker_skills, attacker_pools) in
            (&entities, &wants_melee, &names, &attributes, &skills, &pools).join()
        {
            // Create blank vector of attacks being attempted.
            let mut attacks: Vec<(MeleeWeapon, String)> = Vec::new();
            let mut multi_attack = false;
            // Check if attacker can multi-attack.
            if let Some(_) = multi_attackers.get(entity) {
                multi_attack = true;
            }
            // Check if attacker is using a weapon.
            let mut using_weapon = false;
            for (wielded, melee) in (&equipped, &melee_weapons).join() {
                if wielded.owner == entity && wielded.slot == EquipmentSlot::Melee {
                    using_weapon = get_weapon_attack(wielded, melee, entity, &mut attacks);
                }
            }
            // If not using a weapon, get natural attacks. If we
            // are a multiattacker, get every natural attack. If
            // not, just pick one at random.
            if !using_weapon {
                if let Some(nat) = natural_attacks.get(entity) {
                    get_natural_attacks(&mut rng, nat.clone(), multi_attack, &mut attacks);
                } else {
                    attacks.push((
                        MeleeWeapon {
                            attribute: WeaponAttribute::Strength,
                            damage_n_dice: 1,
                            damage_die_type: 4,
                            damage_bonus: 0,
                            hit_bonus: 0,
                        },
                        "punches".to_string(),
                    ));
                }
            }
            // For every attack, do combat calcs. Break if someone dies.
            for attack in attacks {
                let target_pools = pools.get(wants_melee.target).unwrap();
                let target_attributes = attributes.get(wants_melee.target).unwrap();
                let target_skills = skills.get(wants_melee.target).unwrap();
                if attacker_pools.hit_points.current <= 0 {
                    break;
                }
                if target_pools.hit_points.current <= 0 {
                    break;
                }
                let weapon_info = attack.0;
                let attack_verb = attack.1;
                // Get all offensive bonuses
                let d20 = rng.roll_dice(1, 20);
                let attribute_hit_bonus = attacker_attributes.dexterity.bonus;
                let skill_hit_bonus = gamesystem::skill_bonus(Skill::Melee, &*attacker_skills);
                let mut equipment_hit_bonus = weapon_info.hit_bonus;
                for (wielded, to_hit) in (&equipped, &to_hit).join() {
                    if wielded.owner == entity {
                        equipment_hit_bonus += to_hit.amount;
                    }
                }
                let mut status_hit_bonus = 0;
                if let Some(_) = blind_entities.get(entity) {
                    status_hit_bonus -= 4;
                };
                let hc = hunger_clock.get(entity);
                if let Some(hc) = hc {
                    match hc.state {
                        HungerState::Satiated => {
                            status_hit_bonus += 1;
                        }
                        HungerState::Weak => {
                            status_hit_bonus -= 1;
                        }
                        HungerState::Fainting => {
                            status_hit_bonus -= 2;
                        }
                        _ => {}
                    }
                }
                // Total to-hit bonus
                let attacker_bonuses = 1    // +1 for being in melee combat
                    + attacker_pools.level  // + level
                    + attribute_hit_bonus   // +- str/dex bonus depending on weapon used
                    + skill_hit_bonus       // +- relevant skill modifier
                    + equipment_hit_bonus   // +- any other to-hit modifiers from equipment
                    + status_hit_bonus; //     +- any to-hit modifiers from status effects

                // Get armour class
                let bac = target_pools.bac;
                let attribute_ac_bonus = target_attributes.dexterity.bonus / 2;
                let skill_ac_bonus = gamesystem::skill_bonus(Skill::Defence, &*target_skills);
                let mut armour_ac_bonus = 0;
                for (wielded, ac) in (&equipped, &ac).join() {
                    if wielded.owner == wants_melee.target {
                        armour_ac_bonus += ac.amount;
                    }
                }
                let actual_armour_class = bac - attribute_ac_bonus - skill_ac_bonus - armour_ac_bonus;
                let mut armour_class_roll = actual_armour_class;

                if actual_armour_class < 0 {
                    // Invert armour class so we can roll 1d(AC)
                    armour_class_roll = rng.roll_dice(1, -actual_armour_class);
                    // Invert result so it's a negative again
                    armour_class_roll = -armour_class_roll;
                }

                // Monster attacks receive a +10 to-hit bonus against the player.
                let monster_v_player_bonus = if wants_melee.target == *player_entity { 10 } else { 0 };

                let target_number = monster_v_player_bonus + armour_class_roll + attacker_bonuses;

                let target_name = names.get(wants_melee.target).unwrap();
                if COMBAT_LOGGING {
                    rltk::console::log(format!(
                        "ATTACKLOG: {} *{}* {}: rolled ({}) 1d20 vs. {} ({} + {}AC + {}to-hit)",
                        &name.name,
                        attack_verb,
                        &target_name.name,
                        d20,
                        target_number,
                        monster_v_player_bonus,
                        armour_class_roll,
                        attacker_bonuses
                    ));
                }

                if d20 < target_number {
                    // Target hit!
                    let base_damage = rng.roll_dice(weapon_info.damage_n_dice, weapon_info.damage_die_type);
                    let skill_damage_bonus = gamesystem::skill_bonus(Skill::Melee, &*attacker_skills);
                    let mut attribute_damage_bonus = weapon_info.damage_bonus;
                    match weapon_info.attribute {
                        WeaponAttribute::Dexterity => attribute_damage_bonus += attacker_attributes.dexterity.bonus,
                        WeaponAttribute::Strength => attribute_damage_bonus += attacker_attributes.strength.bonus,
                        WeaponAttribute::Finesse => {
                            if attacker_attributes.dexterity.bonus > attacker_attributes.strength.bonus {
                                attribute_damage_bonus += attacker_attributes.dexterity.bonus;
                            } else {
                                attribute_damage_bonus += attacker_attributes.strength.bonus;
                            }
                        }
                    }
                    let mut damage = i32::max(0, base_damage + skill_damage_bonus + attribute_damage_bonus);

                    if COMBAT_LOGGING {
                        rltk::console::log(format!(
                            "ATTACKLOG: {} HIT for {} ({}[{}d{}]+{}[skill]+{}[attr])",
                            &name.name,
                            damage,
                            base_damage,
                            weapon_info.damage_n_dice,
                            weapon_info.damage_die_type,
                            skill_damage_bonus,
                            attribute_damage_bonus
                        ));
                    }

                    if actual_armour_class < 0 {
                        let ac_damage_reduction = rng.roll_dice(1, -actual_armour_class);
                        damage = i32::min(1, damage - ac_damage_reduction);
                        if COMBAT_LOGGING {
                            rltk::console::log(format!(
                                "ATTACKLOG: {} reduced their damage taken by {} (1dAC), and took {} hp damage.",
                                &target_name.name, ac_damage_reduction, damage
                            ));
                        }
                    }

                    let pos = positions.get(wants_melee.target);
                    if let Some(pos) = pos {
                        particle_builder.damage_taken(pos.x, pos.y)
                    }
                    add_effect(
                        Some(entity),
                        EffectType::Damage { amount: damage },
                        Targets::Entity { target: wants_melee.target },
                    );
                    if entity == *player_entity {
                        something_to_log = true;
                        logger = logger // You hit the <name>.
                            .append("You hit the")
                            .colour(renderable_colour(&renderables, wants_melee.target))
                            .append_n(&target_name.name)
                            .colour(WHITE)
                            .period();
                    } else if wants_melee.target == *player_entity {
                        something_to_log = true;
                        logger = logger // <name> hits you!
                            .append("The")
                            .colour(renderable_colour(&renderables, entity))
                            .append(&name.name)
                            .colour(WHITE)
                            .append(attack_verb)
                            .append("you!");
                    } else {
                        gamelog::Logger::new() // <name> misses the <target>.
                            .append("The")
                            .colour(renderable_colour(&renderables, entity))
                            .append(&name.name)
                            .colour(WHITE)
                            .append(attack_verb)
                            .append("the")
                            .colour(renderable_colour(&renderables, wants_melee.target))
                            .append_n(&target_name.name)
                            .colour(WHITE)
                            .period()
                            .log();
                    }
                } else {
                    if COMBAT_LOGGING {
                        rltk::console::log(format!("ATTACKLOG: {} *MISSED*", &name.name));
                    }

                    let pos = positions.get(wants_melee.target);
                    if let Some(pos) = pos {
                        particle_builder.attack_miss(pos.x, pos.y)
                    }
                    if entity == *player_entity {
                        something_to_log = true;
                        logger = logger // You miss.
                            .append("You miss.");
                    } else if wants_melee.target == *player_entity {
                        something_to_log = true;
                        logger = logger // <name> misses!
                            .append("The")
                            .colour(renderable_colour(&renderables, entity))
                            .append(&name.name)
                            .colour(WHITE)
                            .append("misses!");
                    } else {
                        gamelog::Logger::new() // <name> misses the <target>.
                            .append("The")
                            .colour(renderable_colour(&renderables, entity))
                            .append(&name.name)
                            .colour(WHITE)
                            .append("misses the")
                            .colour(renderable_colour(&renderables, wants_melee.target))
                            .append_n(&target_name.name)
                            .colour(WHITE)
                            .period()
                            .log();
                    }
                }
            }
        }
        wants_melee.clear();
        if something_to_log {
            logger.log();
        }
    }
}

fn get_natural_attacks(
    rng: &mut rltk::RandomNumberGenerator,
    nat: NaturalAttacks,
    multi_attack: bool,
    attacks: &mut Vec<(MeleeWeapon, String)>,
) {
    if !nat.attacks.is_empty() {
        if multi_attack {
            for a in nat.attacks.iter() {
                attacks.push((
                    MeleeWeapon {
                        attribute: WeaponAttribute::Strength,
                        hit_bonus: a.hit_bonus,
                        damage_n_dice: a.damage_n_dice,
                        damage_die_type: a.damage_die_type,
                        damage_bonus: a.damage_bonus,
                    },
                    a.name.to_string(),
                ));
            }
        } else {
            let attack_index =
                if nat.attacks.len() == 1 { 0 } else { rng.roll_dice(1, nat.attacks.len() as i32) as usize - 1 };
            attacks.push((
                MeleeWeapon {
                    attribute: WeaponAttribute::Strength,
                    hit_bonus: nat.attacks[attack_index].hit_bonus,
                    damage_n_dice: nat.attacks[attack_index].damage_n_dice,
                    damage_die_type: nat.attacks[attack_index].damage_die_type,
                    damage_bonus: nat.attacks[attack_index].damage_bonus,
                },
                nat.attacks[attack_index].name.to_string(),
            ));
        }
    }
}

fn get_weapon_attack(
    wielded: &Equipped,
    melee: &MeleeWeapon,
    entity: Entity,
    attacks: &mut Vec<(MeleeWeapon, String)>,
) -> bool {
    if wielded.owner == entity && wielded.slot == EquipmentSlot::Melee {
        attacks.push((melee.clone(), "hits".to_string()));
        return true;
    }
    return false;
}
