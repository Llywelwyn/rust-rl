use super::{
    gamelog, gamesystem, ArmourClassBonus, Attributes, EquipmentSlot, Equipped, HungerClock, HungerState, MeleeWeapon,
    Name, NaturalAttacks, ParticleBuilder, Pools, Position, Skill, Skills, SufferDamage, WantsToMelee, WeaponAttribute,
};
use specs::prelude::*;

pub struct MeleeCombatSystem {}

impl<'a> System<'a> for MeleeCombatSystem {
    type SystemData = (
        Entities<'a>,
        ReadExpect<'a, Entity>,
        WriteStorage<'a, WantsToMelee>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, Attributes>,
        ReadStorage<'a, Skills>,
        ReadStorage<'a, Pools>,
        WriteStorage<'a, SufferDamage>,
        WriteExpect<'a, ParticleBuilder>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, Equipped>,
        ReadStorage<'a, MeleeWeapon>,
        ReadStorage<'a, NaturalAttacks>,
        ReadStorage<'a, ArmourClassBonus>,
        ReadStorage<'a, HungerClock>,
        WriteExpect<'a, rltk::RandomNumberGenerator>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            entities,
            player_entity,
            mut wants_melee,
            names,
            attributes,
            skills,
            pools,
            mut inflict_damage,
            mut particle_builder,
            positions,
            equipped,
            melee_weapons,
            natural_attacks,
            ac,
            hunger_clock,
            mut rng,
        ) = data;

        for (entity, wants_melee, name, attacker_attributes, attacker_skills, attacker_pools) in
            (&entities, &wants_melee, &names, &attributes, &skills, &pools).join()
        {
            let target_pools = pools.get(wants_melee.target).unwrap();
            let target_attributes = attributes.get(wants_melee.target).unwrap();
            let target_skills = skills.get(wants_melee.target).unwrap();

            if attacker_pools.hit_points.current <= 0 {
                break;
            }
            if target_pools.hit_points.current <= 0 {
                break;
            }

            let target_name = names.get(wants_melee.target).unwrap();

            let mut weapon_info = MeleeWeapon {
                attribute: WeaponAttribute::Strength,
                hit_bonus: 0,
                damage_n_dice: 1,
                damage_die_type: 4,
                damage_bonus: 0,
            };
            let mut attack_verb = "hits";

            if let Some(nat) = natural_attacks.get(entity) {
                rltk::console::log("Natural attack found");
                if !nat.attacks.is_empty() {
                    let attack_index = if nat.attacks.len() == 1 {
                        0
                    } else {
                        rng.roll_dice(1, nat.attacks.len() as i32) as usize - 1
                    };
                    weapon_info.hit_bonus = nat.attacks[attack_index].hit_bonus;
                    weapon_info.damage_n_dice = nat.attacks[attack_index].damage_n_dice;
                    weapon_info.damage_die_type = nat.attacks[attack_index].damage_die_type;
                    weapon_info.damage_bonus = nat.attacks[attack_index].damage_bonus;
                    attack_verb = &nat.attacks[attack_index].name;
                }
            }

            for (wielded, melee) in (&equipped, &melee_weapons).join() {
                if wielded.owner == entity && wielded.slot == EquipmentSlot::Melee {
                    weapon_info = melee.clone();
                }
            }

            // Get all offensive bonuses
            let d20 = rng.roll_dice(1, 20);
            let attribute_hit_bonus = attacker_attributes.strength.bonus;
            let skill_hit_bonus = gamesystem::skill_bonus(Skill::Melee, &*attacker_skills);
            let weapon_hit_bonus = weapon_info.hit_bonus;
            let mut status_hit_bonus = 0;
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
            let modified_hit_roll = d20 - attribute_hit_bonus - skill_hit_bonus - weapon_hit_bonus - status_hit_bonus;

            // Get armour class
            let bac = target_pools.bac;
            let attribute_ac_bonus = target_attributes.dexterity.bonus;
            let skill_ac_bonus = gamesystem::skill_bonus(Skill::Defence, &*target_skills);
            let mut armour_ac_bonus = 0;
            for (wielded, ac) in (&equipped, &ac).join() {
                if wielded.owner == wants_melee.target {
                    armour_ac_bonus += ac.amount;
                }
            }
            let armour_class = bac - attribute_ac_bonus - skill_ac_bonus - armour_ac_bonus;

            let target_number = 10 + armour_class + attacker_pools.level;

            if d20 != 1 && (d20 == 20 || modified_hit_roll < target_number) {
                // Target hit!
                let base_damage = rng.roll_dice(weapon_info.damage_n_dice, weapon_info.damage_die_type);
                let attribute_damage_bonus = attacker_attributes.strength.bonus;
                let skill_damage_bonus = gamesystem::skill_bonus(Skill::Melee, &*attacker_skills);
                let weapon_damage_bonus = weapon_info.damage_bonus;
                let damage =
                    i32::max(0, base_damage + attribute_damage_bonus + skill_damage_bonus + weapon_damage_bonus);

                let pos = positions.get(wants_melee.target);
                if let Some(pos) = pos {
                    particle_builder.damage_taken(pos.x, pos.y)
                }
                SufferDamage::new_damage(&mut inflict_damage, wants_melee.target, damage);
                if entity == *player_entity {
                    gamelog::Logger::new() // You hit the <name>.
                        .append("You hit the")
                        .npc_name_n(&target_name.name)
                        .period()
                        .log();
                } else if wants_melee.target == *player_entity {
                    gamelog::Logger::new() // <name> hits you!
                        .append("The")
                        .npc_name(&name.name)
                        .append(attack_verb)
                        .append("you!")
                        .log();
                } else {
                    gamelog::Logger::new() // <name> misses the <target>.
                        .append("The")
                        .npc_name(&name.name)
                        .append(attack_verb)
                        .append("the")
                        .npc_name_n(&target_name.name)
                        .period()
                        .log();
                }
            } else {
                let pos = positions.get(wants_melee.target);
                if let Some(pos) = pos {
                    particle_builder.attack_miss(pos.x, pos.y)
                }
                if entity == *player_entity {
                    gamelog::Logger::new() // You miss.
                        .append("You miss.")
                        .log();
                } else if wants_melee.target == *player_entity {
                    gamelog::Logger::new() // <name> misses!
                        .append("The")
                        .npc_name(&name.name)
                        .colour(rltk::WHITE)
                        .append("misses!")
                        .log();
                } else {
                    gamelog::Logger::new() // <name> misses the <target>.
                        .append("The")
                        .npc_name(&name.name)
                        .colour(rltk::WHITE)
                        .append("misses the")
                        .npc_name_n(&target_name.name)
                        .period()
                        .log();
                }
            }
        }

        wants_melee.clear();
    }
}
