use super::Raws;
use crate::components::*;
use crate::gamesystem::*;
use crate::random_table::RandomTable;
use regex::Regex;
use specs::prelude::*;
use specs::saveload::{MarkedBuilder, SimpleMarker};
use std::collections::{HashMap, HashSet};

const SPAWN_LOGGING: bool = true;

pub enum SpawnType {
    AtPosition { x: i32, y: i32 },
    Equipped { by: Entity },
    Carried { by: Entity },
}

pub struct RawMaster {
    raws: Raws,
    item_index: HashMap<String, usize>,
    mob_index: HashMap<String, usize>,
    prop_index: HashMap<String, usize>,
    table_index: HashMap<String, usize>,
}

impl RawMaster {
    pub fn empty() -> RawMaster {
        RawMaster {
            raws: Raws { items: Vec::new(), mobs: Vec::new(), props: Vec::new(), spawn_tables: Vec::new() },
            item_index: HashMap::new(),
            mob_index: HashMap::new(),
            prop_index: HashMap::new(),
            table_index: HashMap::new(),
        }
    }

    pub fn load(&mut self, raws: Raws) {
        self.raws = raws;
        let mut used_names: HashSet<String> = HashSet::new();
        for (i, item) in self.raws.items.iter().enumerate() {
            if used_names.contains(&item.id) {
                rltk::console::log(format!("DEBUGINFO: Duplicate Item ID found in raws [{}]", item.id));
            }
            self.item_index.insert(item.id.clone(), i);
            used_names.insert(item.id.clone());
        }
        for (i, mob) in self.raws.mobs.iter().enumerate() {
            if used_names.contains(&mob.id) {
                rltk::console::log(format!("DEBUGINFO: Duplicate Mob ID found in raws [{}]", mob.id));
            }
            self.mob_index.insert(mob.id.clone(), i);
            used_names.insert(mob.id.clone());
        }
        for (i, prop) in self.raws.props.iter().enumerate() {
            if used_names.contains(&prop.id) {
                rltk::console::log(format!("DEBUGINFO: Duplicate Prop ID found in raws [{}]", prop.id));
            }
            self.prop_index.insert(prop.id.clone(), i);
            used_names.insert(prop.id.clone());
        }
        for (i, table) in self.raws.spawn_tables.iter().enumerate() {
            if used_names.contains(&table.id) {
                rltk::console::log(format!("DEBUGINFO: Duplicate SpawnTable ID found in raws [{}]", table.id));
            }
            self.table_index.insert(table.id.clone(), i);
            used_names.insert(table.id.clone());

            for entry in table.table.iter() {
                if !used_names.contains(&entry.id) {
                    rltk::console::log(format!("DEBUGINFO: SpawnTables references unspecified entity [{}]", entry.id));
                }
            }
        }
    }
}

pub fn spawn_named_entity(
    raws: &RawMaster,
    ecs: &mut World,
    key: &str,
    pos: SpawnType,
    map_difficulty: i32,
) -> Option<Entity> {
    if raws.item_index.contains_key(key) {
        return spawn_named_item(raws, ecs, key, pos);
    } else if raws.mob_index.contains_key(key) {
        return spawn_named_mob(raws, ecs, key, pos, map_difficulty);
    } else if raws.prop_index.contains_key(key) {
        return spawn_named_prop(raws, ecs, key, pos);
    }
    None
}

pub fn spawn_named_item(raws: &RawMaster, ecs: &mut World, key: &str, pos: SpawnType) -> Option<Entity> {
    if raws.item_index.contains_key(key) {
        let item_template = &raws.raws.items[raws.item_index[key]];
        let mut eb = ecs.create_entity().marked::<SimpleMarker<SerializeMe>>();

        eb = eb.with(Name { name: item_template.name.name.clone(), plural: item_template.name.plural.clone() });
        eb = eb.with(Item {});
        eb = spawn_position(pos, eb, key, raws);

        if let Some(renderable) = &item_template.renderable {
            eb = eb.with(get_renderable_component(renderable));
        }

        let mut weapon_type = -1;

        if let Some(flags) = &item_template.flags {
            for flag in flags.iter() {
                match flag.as_str() {
                    "CONSUMABLE" => eb = eb.with(Consumable {}),
                    "DESTRUCTIBLE" => eb = eb.with(Destructible {}),
                    "CURSED" => eb = eb.with(Cursed {}),
                    "EQUIP_MELEE" => eb = eb.with(Equippable { slot: EquipmentSlot::Melee }),
                    "EQUIP_SHIELD" => eb = eb.with(Equippable { slot: EquipmentSlot::Shield }),
                    "EQUIP_HEAD" => eb = eb.with(Equippable { slot: EquipmentSlot::Head }),
                    "EQUIP_BODY" => eb = eb.with(Equippable { slot: EquipmentSlot::Body }),
                    "EQUIP_FEET" => eb = eb.with(Equippable { slot: EquipmentSlot::Feet }),
                    "EQUIP_HANDS" => eb = eb.with(Equippable { slot: EquipmentSlot::Hands }),
                    "EQUIP_NECK" => eb = eb.with(Equippable { slot: EquipmentSlot::Neck }),
                    "EQUIP_BACK" => eb = eb.with(Equippable { slot: EquipmentSlot::Back }),
                    "WAND" => eb = eb.with(Wand { uses: 3, max_uses: 3 }),
                    "FOOD" => eb = eb.with(ProvidesNutrition {}),
                    "STRENGTH" => weapon_type = 0,
                    "DEXTERITY" => weapon_type = 2,
                    "FINESSE" => weapon_type = 3,
                    _ => rltk::console::log(format!("Unrecognised flag: {}", flag.as_str())),
                }
            }
        }

        let mut base_damage = "1d4";
        let mut hit_bonus = 0;

        if let Some(effects_list) = &item_template.effects {
            for effect in effects_list.iter() {
                let effect_name = effect.0.as_str();
                match effect_name {
                    "provides_healing" => eb = eb.with(ProvidesHealing { amount: effect.1.parse::<i32>().unwrap() }),
                    "ranged" => eb = eb.with(Ranged { range: effect.1.parse::<i32>().unwrap() }),
                    "damage" => eb = eb.with(InflictsDamage { amount: effect.1.parse::<i32>().unwrap() }),
                    "aoe" => eb = eb.with(AOE { radius: effect.1.parse::<i32>().unwrap() }),
                    "confusion" => eb = eb.with(Confusion { turns: effect.1.parse::<i32>().unwrap() }),
                    "base_damage" => base_damage = effect.1,
                    "hit_bonus" => hit_bonus = effect.1.parse::<i32>().unwrap(),
                    "ac" => eb = eb.with(ArmourClassBonus { amount: effect.1.parse::<i32>().unwrap() }),
                    "magicmapper" => eb = eb.with(MagicMapper {}),
                    "digger" => eb = eb.with(Digger {}),
                    _ => rltk::console::log(format!("Warning: effect {} not implemented.", effect_name)),
                }
            }
        }

        if weapon_type != -1 {
            let (n_dice, die_type, bonus) = parse_dice_string(base_damage);
            let mut wpn = MeleeWeapon {
                attribute: WeaponAttribute::Strength,
                damage_n_dice: n_dice,
                damage_die_type: die_type,
                damage_bonus: bonus,
                hit_bonus: hit_bonus,
            };
            match weapon_type {
                0 => wpn.attribute = WeaponAttribute::Strength,
                1 => wpn.attribute = WeaponAttribute::Dexterity,
                _ => wpn.attribute = WeaponAttribute::Finesse,
            }
            eb = eb.with(wpn);
        }

        return Some(eb.build());
    }
    None
}

pub fn spawn_named_mob(
    raws: &RawMaster,
    ecs: &mut World,
    key: &str,
    pos: SpawnType,
    map_difficulty: i32,
) -> Option<Entity> {
    if raws.mob_index.contains_key(key) {
        let mob_template = &raws.raws.mobs[raws.mob_index[key]];
        let mut player_level = 1;
        {
            let pools = ecs.read_storage::<Pools>();
            let player_entity = ecs.fetch::<Entity>();
            let player_pool = pools.get(*player_entity);
            if let Some(pool) = player_pool {
                player_level = pool.level;
            }
        }

        let mut eb;
        // New entity with a position, name, combatstats, and viewshed
        eb = ecs.create_entity().marked::<SimpleMarker<SerializeMe>>();

        eb = spawn_position(pos, eb, key, raws);
        eb = eb.with(Name { name: mob_template.name.clone(), plural: mob_template.name.clone() });
        eb = eb.with(Viewshed { visible_tiles: Vec::new(), range: mob_template.vision_range, dirty: true });
        if let Some(renderable) = &mob_template.renderable {
            eb = eb.with(get_renderable_component(renderable));
        }
        if let Some(flags) = &mob_template.flags {
            for flag in flags.iter() {
                match flag.as_str() {
                    "BLOCKS_TILE" => eb = eb.with(BlocksTile {}),
                    "BYSTANDER" => eb = eb.with(Bystander {}),
                    "MONSTER" => eb = eb.with(Monster {}),
                    _ => rltk::console::log(format!("Unrecognised flag: {}", flag.as_str())),
                }
            }
        }
        if let Some(quips) = &mob_template.quips {
            eb = eb.with(Quips { available: quips.clone() });
        }

        // Setup combat stats
        let mut attr = Attributes {
            strength: Attribute { base: 10, modifiers: 0, bonus: 0 },
            dexterity: Attribute { base: 10, modifiers: 0, bonus: 0 },
            constitution: Attribute { base: 10, modifiers: 0, bonus: 0 },
            intelligence: Attribute { base: 10, modifiers: 0, bonus: 0 },
            wisdom: Attribute { base: 10, modifiers: 0, bonus: 0 },
            charisma: Attribute { base: 10, modifiers: 0, bonus: 0 },
        };
        let mut mob_con = 10;
        let mut mob_int = 10;
        if let Some(attributes) = &mob_template.attributes {
            if let Some(str) = attributes.str {
                attr.strength = Attribute { base: str, modifiers: 0, bonus: attr_bonus(str) };
            }
            if let Some(dex) = attributes.dex {
                attr.strength = Attribute { base: dex, modifiers: 0, bonus: attr_bonus(dex) };
            }
            if let Some(con) = attributes.con {
                attr.constitution = Attribute { base: con, modifiers: 0, bonus: attr_bonus(con) };
                mob_con = con;
            }
            if let Some(int) = attributes.int {
                attr.intelligence = Attribute { base: int, modifiers: 0, bonus: attr_bonus(int) };
                mob_int = int;
            }
            if let Some(wis) = attributes.wis {
                attr.wisdom = Attribute { base: wis, modifiers: 0, bonus: attr_bonus(wis) };
            }
            if let Some(cha) = attributes.cha {
                attr.charisma = Attribute { base: cha, modifiers: 0, bonus: attr_bonus(cha) };
            }
        }
        eb = eb.with(attr);

        let base_mob_level = if mob_template.level.is_some() { mob_template.level.unwrap() } else { 0 };
        let mut mob_level = base_mob_level;
        // If the level difficulty is smaller than the mob's base level, subtract 1;
        // else, if the level difficulty is larger, add one-fifth of the difference
        if base_mob_level > map_difficulty {
            mob_level -= 1;
        } else if base_mob_level < map_difficulty {
            mob_level += (map_difficulty - base_mob_level) / 5;
        }
        // If the player is a higher level than the mob, add one-fifth of the difference
        if base_mob_level < player_level {
            mob_level += (player_level - base_mob_level) / 4;
        }
        // If the resulting mob level is more than 1.5x the base, lower it to that number
        mob_level = i32::min(mob_level, (1.5 * base_mob_level as f32).trunc() as i32);

        // Should really use existing RNG here
        let mut rng = rltk::RandomNumberGenerator::new();

        let mob_hp = npc_hp(&mut rng, mob_con, mob_level);
        let mob_mana = mana_at_level(&mut rng, mob_int, mob_level);
        let mob_bac = if mob_template.bac.is_some() { mob_template.bac.unwrap() } else { 10 };

        if SPAWN_LOGGING {
            rltk::console::log(format!(
                "SPAWNLOG: {} ({}HP, {}MANA, {}BAC) spawned at level {} ({}[base], {}[map difficulty], {}[player level])",
                &mob_template.name, mob_hp, mob_mana, mob_bac, mob_level, base_mob_level, map_difficulty, player_level
            ));
        }

        let pools = Pools {
            level: mob_level,
            xp: 0,
            bac: mob_bac,
            hit_points: Pool { current: mob_hp, max: mob_hp },
            mana: Pool { current: mob_mana, max: mob_mana },
        };
        eb = eb.with(pools);

        let mut skills = Skills { skills: HashMap::new() };
        skills.skills.insert(Skill::Melee, 0);
        skills.skills.insert(Skill::Defence, 0);
        if let Some(mobskills) = &mob_template.skills {
            for sk in mobskills.iter() {
                match sk.0.as_str() {
                    "melee" => {
                        skills.skills.insert(Skill::Melee, *sk.1);
                    }
                    "defence" => {
                        skills.skills.insert(Skill::Defence, *sk.1);
                    }
                    "magic" => {
                        skills.skills.insert(Skill::Magic, *sk.1);
                    }
                    _ => {
                        rltk::console::log(format!("Unknown skill referenced: [{}]", sk.0));
                    }
                }
            }
        }
        eb = eb.with(skills);

        if let Some(natural_attacks) = &mob_template.attacks {
            let mut natural = NaturalAttacks { attacks: Vec::new() };
            for na in natural_attacks.iter() {
                let (n, d, b) = parse_dice_string(&na.damage);
                let attack = NaturalAttack {
                    name: na.name.clone(),
                    hit_bonus: na.hit_bonus,
                    damage_n_dice: n,
                    damage_die_type: d,
                    damage_bonus: b,
                };
                natural.attacks.push(attack);
            }
            eb = eb.with(natural);
        }

        let new_mob = eb.build();

        // Build entity, then check for anything they're wearing
        if let Some(wielding) = &mob_template.equipped {
            for tag in wielding.iter() {
                spawn_named_entity(raws, ecs, tag, SpawnType::Equipped { by: new_mob }, map_difficulty);
            }
        }
        return Some(new_mob);
    }
    None
}

pub fn spawn_named_prop(raws: &RawMaster, ecs: &mut World, key: &str, pos: SpawnType) -> Option<Entity> {
    if raws.prop_index.contains_key(key) {
        let prop_template = &raws.raws.props[raws.prop_index[key]];

        let mut eb = ecs.create_entity().marked::<SimpleMarker<SerializeMe>>();
        eb = spawn_position(pos, eb, key, raws);
        if let Some(renderable) = &prop_template.renderable {
            eb = eb.with(get_renderable_component(renderable));
        }
        eb = eb.with(Name { name: prop_template.name.clone(), plural: prop_template.name.clone() });

        if let Some(flags) = &prop_template.flags {
            for flag in flags.iter() {
                match flag.as_str() {
                    "HIDDEN" => eb = eb.with(Hidden {}),
                    "BLOCKS_TILE" => eb = eb.with(BlocksTile {}),
                    "BLOCKS_VISIBILITY" => eb = eb.with(BlocksVisibility {}),
                    "ENTRY_TRIGGER" => eb = eb.with(EntryTrigger {}),
                    "SINGLE_ACTIVATION" => eb = eb.with(SingleActivation {}),
                    "DOOR" => eb = eb.with(Door { open: false }),
                    "PROP" => eb = eb.with(Prop {}),
                    _ => rltk::console::log(format!("Unrecognised flag: {}", flag.as_str())),
                }
            }
        }

        if let Some(effects_list) = &prop_template.effects {
            for effect in effects_list.iter() {
                let effect_name = effect.0.as_str();
                match effect_name {
                    "damage" => eb = eb.with(InflictsDamage { amount: effect.1.parse::<i32>().unwrap() }),
                    "confusion" => eb = eb.with(Confusion { turns: effect.1.parse::<i32>().unwrap() }),
                    _ => rltk::console::log(format!("Warning: effect {} not implemented.", effect_name)),
                }
            }
        }

        return Some(eb.build());
    }
    None
}

fn spawn_position<'a>(pos: SpawnType, new_entity: EntityBuilder<'a>, tag: &str, raws: &RawMaster) -> EntityBuilder<'a> {
    let mut eb = new_entity;

    match pos {
        SpawnType::AtPosition { x, y } => eb = eb.with(Position { x, y }),
        SpawnType::Carried { by } => eb = eb.with(InBackpack { owner: by }),
        SpawnType::Equipped { by } => {
            let slot = find_slot_for_equippable_item(tag, raws);
            eb = eb.with(Equipped { owner: by, slot })
        }
    }

    eb
}

fn get_renderable_component(renderable: &super::item_structs::Renderable) -> crate::components::Renderable {
    crate::components::Renderable {
        glyph: rltk::to_cp437(renderable.glyph.chars().next().unwrap()),
        fg: rltk::RGB::from_hex(&renderable.fg).expect("Invalid RGB"),
        bg: rltk::RGB::from_hex(&renderable.bg).expect("Invalid RGB"),
        render_order: renderable.order,
    }
}

pub fn table_by_name(raws: &RawMaster, key: &str, difficulty: i32) -> RandomTable {
    if raws.table_index.contains_key(key) {
        let spawn_table = &raws.raws.spawn_tables[raws.table_index[key]];

        use super::SpawnTableEntry;

        let upper_bound = difficulty;
        let lower_bound = difficulty / 6;

        let available_options: Vec<&SpawnTableEntry> = spawn_table
            .table
            .iter()
            .filter(|entry| entry.difficulty >= lower_bound && entry.difficulty <= upper_bound)
            .collect();

        let mut rt = RandomTable::new();
        if !available_options.is_empty() {
            for e in available_options.iter() {
                rt = rt.add(e.id.clone(), e.weight);
            }

            return rt;
        }
    }
    if SPAWN_LOGGING {
        rltk::console::log(format!(
            "SPAWNLOG: Something went wrong when trying to spawn {} @ map difficulty {}. Returned debug entry.",
            key, difficulty
        ));
    }
    return RandomTable::new().add("debug", 1);
}

pub fn parse_dice_string(dice: &str) -> (i32, i32, i32) {
    lazy_static! {
        static ref DICE_RE: Regex = Regex::new(r"(\d+)d(\d+)([\+\-]\d+)?").unwrap();
    }
    let mut n_dice = 1;
    let mut die_type = 4;
    let mut die_bonus = 0;
    for cap in DICE_RE.captures_iter(dice) {
        if let Some(group) = cap.get(1) {
            n_dice = group.as_str().parse::<i32>().expect("Not a digit");
        }
        if let Some(group) = cap.get(2) {
            die_type = group.as_str().parse::<i32>().expect("Not a digit");
        }
        if let Some(group) = cap.get(3) {
            die_bonus = group.as_str().parse::<i32>().expect("Not a digit");
        }
    }
    (n_dice, die_type, die_bonus)
}

fn find_slot_for_equippable_item(tag: &str, raws: &RawMaster) -> EquipmentSlot {
    if !raws.item_index.contains_key(tag) {
        panic!("Trying to equip an unknown item: {}", tag);
    }
    let item_index = raws.item_index[tag];
    let item = &raws.raws.items[item_index];
    if let Some(flags) = &item.flags {
        for flag in flags {
            match flag.as_str() {
                "EQUIP_MELEE" => return EquipmentSlot::Melee,
                "EQUIP_SHIELD" => return EquipmentSlot::Shield,
                "EQUIP_BODY" => return EquipmentSlot::Body,
                "EQUIP_HEAD" => return EquipmentSlot::Head,
                "EQUIP_FEET" => return EquipmentSlot::Feet,
                "EQUIP_NECK" => return EquipmentSlot::Neck,
                "EQUIP_BACK" => return EquipmentSlot::Back,
                "EQUIP_HANDS" => return EquipmentSlot::Hands,
                _ => {}
            }
        }
    }
    panic!("Trying to equip {}, but it has no slot tag.", tag);
}
