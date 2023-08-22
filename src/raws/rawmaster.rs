use super::{Raws, Reaction};
use crate::components::*;
use crate::gamesystem::*;
use crate::gui::Ancestry;
use crate::random_table::RandomTable;
use crate::LOG_SPAWNING;
use regex::Regex;
use rltk::prelude::*;
use specs::prelude::*;
use specs::saveload::{MarkedBuilder, SimpleMarker};
use std::collections::{HashMap, HashSet};

/// Applies effects to the entity - e.g. "healing: 1d4+1", where
///  effects are components on the entity with varying parameters.
macro_rules! apply_effects {
    ($effects:expr, $eb:expr) => {
        for effect in $effects.iter() {
            let effect_name = effect.0.as_str();
            match effect_name {
                "healing" => {
                    let (n_dice, sides, modifier) = parse_dice_string(effect.1.as_str());
                    $eb = $eb.with(ProvidesHealing { n_dice, sides, modifier })
                }
                "ranged" => $eb = $eb.with(Ranged { range: effect.1.parse::<i32>().unwrap() }),
                "damage" => {
                    let (n_dice, sides, modifier) = parse_dice_string(effect.1.as_str());
                    $eb = $eb.with(InflictsDamage { n_dice, sides, modifier })
                }
                "aoe" => $eb = $eb.with(AOE { radius: effect.1.parse::<i32>().unwrap() }),
                "confusion" => $eb = $eb.with(Confusion { turns: effect.1.parse::<i32>().unwrap() }),
                "ac" => $eb = $eb.with(ArmourClassBonus { amount: effect.1.parse::<i32>().unwrap() }),
                "to_hit" => $eb = $eb.with(ToHitBonus { amount: effect.1.parse::<i32>().unwrap() }),
                "particle_line" => $eb = $eb.with(parse_particle_line(&effect.1)),
                "particle_burst" => $eb = $eb.with(parse_particle_burst(&effect.1)),
                "particle" => $eb = $eb.with(parse_particle(&effect.1)),
                _ => console::log(format!("Warning: effect {} not implemented.", effect_name)),
            }
        }
    };
}

/// Applies flags to the entity - e.g. "blocks_tile", where
/// flags are components that have no parameters to modify.
macro_rules! apply_flags {
    ($flags:expr, $eb:expr) => {
        for flag in $flags.iter() {
            match flag.as_str() {
                // --- PROP FLAGS BEGIN HERE ---
                "HIDDEN" => $eb = $eb.with(Hidden {}),
                "BLOCKS_TILE" => $eb = $eb.with(BlocksTile {}),
                "BLOCKS_VISIBILITY" => $eb = $eb.with(BlocksVisibility {}),
                "ENTRY_TRIGGER" => $eb = $eb.with(EntryTrigger {}),
                "SINGLE_ACTIVATION" => $eb = $eb.with(SingleActivation {}),
                "DOOR" => {
                    $eb = $eb.with(Door { open: false });
                    $eb = $eb.with(BlocksVisibility {});
                    $eb = $eb.with(BlocksTile {});
                }
                // --- EFFECT FLAGS ---
                "FOOD" => $eb = $eb.with(ProvidesNutrition {}),
                "CONSUMABLE" => $eb = $eb.with(Consumable {}),
                "CHARGES" => $eb = $eb.with(Charges { uses: 3, max_uses: 3 }),
                "REMOVE_CURSE" => $eb = $eb.with(RemovesCurse {}),
                "DIGGER" => $eb = $eb.with(Digger {}),
                "MAGICMAP" => $eb = $eb.with(MagicMapper {}),
                // CAN BE DESTROYED BY DAMAGE
                "DESTRUCTIBLE" => $eb = $eb.with(Destructible {}),
                // --- EQUIP SLOTS ---
                "EQUIP_MELEE" => $eb = $eb.with(Equippable { slot: EquipmentSlot::Melee }),
                "EQUIP_SHIELD" => $eb = $eb.with(Equippable { slot: EquipmentSlot::Shield }),
                "EQUIP_HEAD" => $eb = $eb.with(Equippable { slot: EquipmentSlot::Head }),
                "EQUIP_BODY" => $eb = $eb.with(Equippable { slot: EquipmentSlot::Body }),
                "EQUIP_FEET" => $eb = $eb.with(Equippable { slot: EquipmentSlot::Feet }),
                "EQUIP_HANDS" => $eb = $eb.with(Equippable { slot: EquipmentSlot::Hands }),
                "EQUIP_NECK" => $eb = $eb.with(Equippable { slot: EquipmentSlot::Neck }),
                "EQUIP_BACK" => $eb = $eb.with(Equippable { slot: EquipmentSlot::Back }),
                // --- MOB ANCESTRIES ---
                "IS_PLAYER" => $eb = $eb.with(Player {}),
                "IS_HUMAN" => $eb = $eb.with(HasAncestry { name: Ancestry::Human }),
                "IS_DWARF" => $eb = $eb.with(HasAncestry { name: Ancestry::Dwarf }),
                "IS_ELF" => $eb = $eb.with(HasAncestry { name: Ancestry::Elf }),
                "IS_CATFOLK" => $eb = $eb.with(HasAncestry { name: Ancestry::Catfolk }),
                "IS_GNOME" => $eb = $eb.with(HasAncestry { name: Ancestry::Gnome }),
                // --- MOB FACTIONS ---
                "MINDLESS" => $eb = $eb.with(Faction { name: "mindless".to_string() }),
                "NEUTRAL" => $eb = $eb.with(Faction { name: "neutral".to_string() }),
                "HERBIVORE" => $eb = $eb.with(Faction { name: "herbivore".to_string() }),
                "CARNIVORE" => $eb = $eb.with(Faction { name: "carnivore".to_string() }),
                // --- MOVEMENT MODES --- ( defaults to WANDER )
                "STATIC" => $eb = $eb.with(MoveMode { mode: Movement::Static }),
                "RANDOM_PATH" => $eb = $eb.with(MoveMode { mode: Movement::RandomWaypoint { path: None } }),
                // --- RANDOM MOB ATTRIBUTES ---
                "SMALL_GROUP" => {} // These flags are for region spawning,
                "LARGE_GROUP" => {} // and don't need to apply a component.
                "MULTIATTACK" => $eb = $eb.with(MultiAttack {}),
                "BLIND" => $eb = $eb.with(Blind {}),
                _ => rltk::console::log(format!("Unrecognised flag: {}", flag.as_str())),
            }
        }
    };
}

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
    loot_index: HashMap<String, usize>,
    faction_index: HashMap<String, HashMap<String, Reaction>>,
    ancestry_index: HashMap<String, HashSet<String>>,
}

impl RawMaster {
    pub fn empty() -> RawMaster {
        RawMaster {
            raws: Raws {
                items: Vec::new(),
                mobs: Vec::new(),
                props: Vec::new(),
                spawn_tables: Vec::new(),
                loot_tables: Vec::new(),
                factions: Vec::new(),
                ancestries: Vec::new(),
            },
            item_index: HashMap::new(),
            mob_index: HashMap::new(),
            prop_index: HashMap::new(),
            table_index: HashMap::new(),
            loot_index: HashMap::new(),
            faction_index: HashMap::new(),
            ancestry_index: HashMap::new(),
        }
    }

    pub fn load(&mut self, raws: Raws) {
        self.raws = raws;
        let mut used_names: HashSet<String> = HashSet::new();
        for (i, item) in self.raws.items.iter().enumerate() {
            check_for_duplicate_entries(&used_names, &item.id);
            self.item_index.insert(item.id.clone(), i);
            used_names.insert(item.id.clone());
        }
        for (i, mob) in self.raws.mobs.iter().enumerate() {
            check_for_duplicate_entries(&used_names, &mob.id);
            self.mob_index.insert(mob.id.clone(), i);
            used_names.insert(mob.id.clone());
        }
        for (i, prop) in self.raws.props.iter().enumerate() {
            check_for_duplicate_entries(&used_names, &prop.id);
            self.prop_index.insert(prop.id.clone(), i);
            used_names.insert(prop.id.clone());
        }
        for (i, table) in self.raws.spawn_tables.iter().enumerate() {
            check_for_duplicate_entries(&used_names, &table.id);
            self.table_index.insert(table.id.clone(), i);
            used_names.insert(table.id.clone());
            for entry in table.table.iter() {
                check_for_unspecified_entity(&used_names, &entry.id)
            }
        }
        for (i, loot_table) in self.raws.loot_tables.iter().enumerate() {
            check_for_duplicate_entries(&used_names, &loot_table.id);
            self.loot_index.insert(loot_table.id.clone(), i);
            for entry in loot_table.table.iter() {
                check_for_unspecified_entity(&used_names, &entry.id)
            }
        }
        for faction in self.raws.factions.iter() {
            let mut reactions: HashMap<String, Reaction> = HashMap::new();
            for other in faction.responses.iter() {
                reactions.insert(
                    other.0.clone(),
                    match other.1.as_str() {
                        "flee" => Reaction::Flee,
                        "attack" => Reaction::Attack,
                        _ => Reaction::Ignore,
                    },
                );
            }
            self.faction_index.insert(faction.id.clone(), reactions);
        }
        for ancestry in self.raws.ancestries.iter() {
            self.ancestry_index.insert(ancestry.id.clone(), ancestry.allies.clone());
        }
    }
}

/// Checks a string against a HashSet, logging if a duplicate is found.
fn check_for_duplicate_entries(used_names: &HashSet<String>, id: &String) {
    if used_names.contains(id) {
        rltk::console::log(format!("DEBUGINFO: Duplicate ID found in raws [{}]", id));
    }
}
/// Checks a string against a HashSet, logging if the string isn't found.
fn check_for_unspecified_entity(used_names: &HashSet<String>, id: &String) {
    if !used_names.contains(id) {
        rltk::console::log(format!("DEBUGINFO: Table references unspecified entity [{}]", id));
    }
}

pub fn spawn_named_entity(
    raws: &RawMaster,
    ecs: &mut World,
    key: &str,
    buc: Option<BUC>,
    pos: SpawnType,
    map_difficulty: i32,
) -> Option<Entity> {
    if raws.item_index.contains_key(key) {
        return spawn_named_item(raws, ecs, key, buc, pos);
    } else if raws.mob_index.contains_key(key) {
        return spawn_named_mob(raws, ecs, key, pos, map_difficulty);
    } else if raws.prop_index.contains_key(key) {
        return spawn_named_prop(raws, ecs, key, pos);
    }
    None
}

pub fn spawn_named_item(
    raws: &RawMaster,
    ecs: &mut World,
    key: &str,
    buc: Option<BUC>,
    pos: SpawnType,
) -> Option<Entity> {
    if raws.item_index.contains_key(key) {
        let item_template = &raws.raws.items[raws.item_index[key]];
        let mut dm = ecs.fetch_mut::<crate::map::MasterDungeonMap>();
        let scroll_names = dm.scroll_map.clone();
        let potion_names = dm.potion_map.clone();
        let wand_names = dm.wand_map.clone();
        let identified_items = dm.identified_items.clone();
        // --- WE GET ALL THE VALUES FROM THE ECS WE NEED HERE, BEFORE ---
        // --- WE CREATE THE EB, TO AVOID BORROW CHECKER COMPLAINTS    ---
        let roll = ecs.write_resource::<RandomNumberGenerator>().roll_dice(1, 6);
        let player_entity = ecs.fetch::<Entity>();
        let known_beatitude = match pos {
            SpawnType::Equipped { by } => {
                if by == *player_entity {
                    true
                } else {
                    false
                }
            }
            SpawnType::Carried { by } => {
                if by == *player_entity {
                    true
                } else {
                    false
                }
            }
            _ => false,
        };
        if known_beatitude && !identified_items.contains(&item_template.name.name) {
            dm.identified_items.insert(item_template.name.name.clone());
        }
        std::mem::drop(player_entity);
        std::mem::drop(dm);
        // -- DROP EVERYTHING THAT INVOLVES THE ECS BEFORE THIS POINT ---
        let mut eb = ecs.create_entity().marked::<SimpleMarker<SerializeMe>>();

        eb = eb.with(Name { name: item_template.name.name.clone(), plural: item_template.name.plural.clone() });
        eb = eb.with(Item { weight: item_template.weight.unwrap_or(0.0), value: item_template.value.unwrap_or(0.0) });
        eb = spawn_position(pos, eb, key, raws);

        if let Some(renderable) = &item_template.renderable {
            eb = eb.with(get_renderable_component(renderable));
        }
        // BEATITUDE
        let buc = if let Some(buc_status) = buc {
            buc_status
        } else {
            match roll {
                1 => BUC::Cursed,
                2 => BUC::Blessed,
                _ => BUC::Uncursed,
            }
        };
        eb = eb.with(Beatitude { buc, known: known_beatitude });

        if let Some(flags) = &item_template.flags {
            apply_flags!(flags, eb);
        }
        if let Some(effects_list) = &item_template.effects {
            apply_effects!(effects_list, eb);
        }

        if let Some(magic_item) = &item_template.magic {
            let item_class = match magic_item.class.as_str() {
                "common" => MagicItemClass::Common,
                "uncommon" => MagicItemClass::Uncommon,
                "rare" => MagicItemClass::Rare,
                "veryrare" => MagicItemClass::VeryRare,
                _ => MagicItemClass::Legendary,
            };
            eb = eb.with(MagicItem { class: item_class });
            if !identified_items.contains(&item_template.name.name) {
                #[allow(clippy::single_match)]
                match magic_item.naming.as_str() {
                    "scroll" => {
                        let singular = scroll_names[&item_template.name.name].clone();
                        let mut plural = singular.clone();
                        plural += "s";
                        eb = eb.with(ObfuscatedName { name: singular, plural: plural })
                    }
                    "potion" => {
                        let singular = potion_names[&item_template.name.name].clone();
                        let mut plural = singular.clone();
                        plural += "s";
                        eb = eb.with(ObfuscatedName { name: singular, plural: plural })
                    }
                    "wand" => {
                        let singular = wand_names[&item_template.name.name].clone();
                        let mut plural = singular.clone();
                        plural += "s";
                        eb = eb.with(ObfuscatedName { name: singular, plural: plural })
                    }
                    _ => {
                        let singular = magic_item.naming.clone();
                        let mut plural = singular.clone();
                        plural += "s";
                        eb = eb.with(ObfuscatedName { name: singular, plural: plural })
                    }
                }
            }
        }

        if let Some(weapon) = &item_template.equip {
            let (n_dice, die_type, bonus) = parse_dice_string(weapon.damage.as_str());
            let weapon_attribute = match weapon.flag.as_str() {
                "DEXTERITY" => WeaponAttribute::Dexterity,
                "FINESSE" => WeaponAttribute::Finesse,
                _ => WeaponAttribute::Strength,
            };
            let wpn = MeleeWeapon {
                attribute: weapon_attribute,
                damage_n_dice: n_dice,
                damage_die_type: die_type,
                damage_bonus: bonus,
                hit_bonus: weapon.to_hit.unwrap_or(0),
            };
            eb = eb.with(wpn);
        }

        return Some(eb.build());
    }
    console::log(format!("DEBUGINFO: Tried to spawn named item [{}] but failed", key));
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
        if let Some(telepath) = &mob_template.telepathy_range {
            eb = eb.with(Telepath { telepath_tiles: Vec::new(), range: *telepath, dirty: true });
        }
        if let Some(renderable) = &mob_template.renderable {
            eb = eb.with(get_renderable_component(renderable));
        }
        eb = eb.with(BlocksTile {});
        eb = eb.with(Faction { name: "hostile".to_string() });
        eb = eb.with(MoveMode { mode: Movement::Random });
        let mut xp_value = 1;
        let mut has_mind = true;
        if let Some(flags) = &mob_template.flags {
            apply_flags!(flags, eb);
            if flags.contains(&"MULTIATTACK".to_string()) {
                xp_value += 3;
            }
            if flags.contains(&"MINDLESS".to_string()) {
                has_mind = false;
            }
        }
        if has_mind {
            eb = eb.with(Mind {});
        }
        // Add quips, if we have some listed.
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

        let speed = if mob_template.speed.is_some() { mob_template.speed.unwrap() } else { 12 };
        eb = eb.with(Energy { current: 0, speed: speed });

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
        let mob_hp = npc_hp_at_level(&mut rng, mob_con, mob_level);
        let mob_mana = mana_at_level(&mut rng, mob_int, mob_level);
        let mob_bac = if mob_template.bac.is_some() { mob_template.bac.unwrap() } else { 10 };

        let pools = Pools {
            level: mob_level,
            xp: 0,
            bac: mob_bac,
            hit_points: Pool { current: mob_hp, max: mob_hp },
            mana: Pool { current: mob_mana, max: mob_mana },
            weight: 0.0,
            god: false,
        };
        eb = eb.with(pools);
        eb = eb.with(EquipmentChanged {});

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

        xp_value += mob_level * mob_level;
        if speed > 18 {
            xp_value += 5;
        } else if speed > 12 {
            xp_value += 3;
        }
        if mob_bac < 0 {
            xp_value += 14 + 2 * mob_bac;
        } else if mob_bac == 0 {
            xp_value += 7;
        } else if mob_bac == 1 {
            xp_value += 6;
        } else if mob_bac == 2 {
            xp_value += 5;
        }
        if mob_level > 9 {
            xp_value += 50;
        }
        // Final xp value = 1 + level^2 + bonus for low ac + bonus for speed + bonus for multiattack + bonus for level>9

        eb = eb.with(GrantsXP { amount: xp_value });

        // Setup loot table
        if let Some(loot) = &mob_template.loot {
            eb = eb.with(LootTable { table: loot.table.clone(), chance: loot.chance });
        }

        if LOG_SPAWNING {
            rltk::console::log(format!(
                "SPAWNLOG: {} ({}HP, {}MANA, {}BAC) spawned at level {} ({}[base], {}[map difficulty], {}[player level]), worth {} XP",
                &mob_template.name, mob_hp, mob_mana, mob_bac, mob_level, base_mob_level, map_difficulty, player_level, xp_value
            ));
        }

        let new_mob = eb.build();

        // Build entity, then check for anything they're wearing
        if let Some(wielding) = &mob_template.equipped {
            for tag in wielding.iter() {
                spawn_named_entity(raws, ecs, tag, None, SpawnType::Equipped { by: new_mob }, map_difficulty);
            }
        }

        return Some(new_mob);
    }
    None
}

pub fn spawn_named_prop(raws: &RawMaster, ecs: &mut World, key: &str, pos: SpawnType) -> Option<Entity> {
    if raws.prop_index.contains_key(key) {
        // ENTITY BUILDER PREP
        let prop_template = &raws.raws.props[raws.prop_index[key]];
        let mut eb = ecs.create_entity().marked::<SimpleMarker<SerializeMe>>();
        eb = spawn_position(pos, eb, key, raws);
        // APPLY MANDATORY COMPONENTS FOR A PROP:
        //  - Name
        //  - Prop {}
        eb = eb.with(Name { name: prop_template.name.clone(), plural: prop_template.name.clone() });
        eb = eb.with(Prop {});
        // APPLY OPTIONAL COMPONENTS FOR A PROP:
        // - Renderable
        // - Flags
        // - Effects
        if let Some(renderable) = &prop_template.renderable {
            eb = eb.with(get_renderable_component(renderable));
        }

        if let Some(flags) = &prop_template.flags {
            apply_flags!(flags, eb);
        }

        if let Some(effects_list) = &prop_template.effects {
            apply_effects!(effects_list, eb);
        }
        // BUILD THE ENTITY
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

pub fn table_by_name(raws: &RawMaster, key: &str, optional_difficulty: Option<i32>) -> RandomTable {
    let (difficulty, upper_bound, lower_bound);
    if optional_difficulty.is_some() {
        difficulty = optional_difficulty.unwrap();
        upper_bound = difficulty;
        lower_bound = if key != "mobs" { 0 } else { difficulty / 6 };
    } else {
        difficulty = -1;
        upper_bound = 100;
        lower_bound = 0;
    }
    if raws.table_index.contains_key(key) {
        let spawn_table = &raws.raws.spawn_tables[raws.table_index[key]];

        use super::SpawnTableEntry;

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
    rltk::console::log(format!(
        "DEBUGINFO: Something went wrong when trying to spawn {} @ map difficulty {} [upper bound: {}, lower bound: {}]. Returned debug entry.",
        key, difficulty, upper_bound, lower_bound
    ));
    return RandomTable::new().add("debug", 1);
}

pub fn parse_dice_string(dice: &str) -> (i32, i32, i32) {
    lazy_static! {
        static ref DICE_RE: Regex = Regex::new(r"(\d+)d(\d+)([\+\-]\d+)?").unwrap();
    }
    let mut n_dice = 1;
    let mut die_type = 1;
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

pub fn roll_on_loot_table(raws: &RawMaster, rng: &mut RandomNumberGenerator, key: &str) -> Option<String> {
    if raws.loot_index.contains_key(key) {
        console::log(format!("DEBUGINFO: Rolling on loot table: {}", key));
        let mut rt = RandomTable::new();
        let available_options = &raws.raws.loot_tables[raws.loot_index[key]];
        for item in available_options.table.iter() {
            rt = rt.add(item.id.clone(), item.weight);
        }
        return Some(rt.roll(rng));
    } else if raws.table_index.contains_key(key) {
        console::log(format!("DEBUGINFO: No loot table found, so using spawn table: {}", key));
        let mut rt = RandomTable::new();
        let available_options = &raws.raws.spawn_tables[raws.table_index[key]];
        for item in available_options.table.iter() {
            rt = rt.add(item.id.clone(), item.weight);
        }
        return Some(rt.roll(rng));
    }
    console::log(format!("DEBUGINFO: Unknown loot table {}", key));
    return None;
}

#[derive(PartialEq, Copy, Clone)]
pub enum SpawnsAs {
    Single,
    SmallGroup,
    LargeGroup,
}

/// Looks up a key in the mob index, and if a corresponding mob is found, searches the
/// entry for a group spawn flag. i.e. SMALL_GROUP/LARGE_GROUP.
pub fn get_mob_spawn_type(raws: &RawMaster, key: &str) -> SpawnsAs {
    if raws.mob_index.contains_key(key) {
        let mob_template = &raws.raws.mobs[raws.mob_index[key]];
        if let Some(flags) = &mob_template.flags {
            for flag in flags {
                match flag.as_str() {
                    "SMALL_GROUP" => return SpawnsAs::SmallGroup,
                    "LARGE_GROUP" => return SpawnsAs::LargeGroup,
                    _ => {}
                }
            }
        }
    }
    return SpawnsAs::Single;
}

pub fn get_mob_spawn_amount(rng: &mut RandomNumberGenerator, spawn_type: &SpawnsAs, player_level: i32) -> i32 {
    let n = match spawn_type {
        // Single mobs always spawn alone.
        SpawnsAs::Single => 1,
        // Small groups either spawn alone or as a small group (2-4).
        SpawnsAs::SmallGroup => {
            if rng.roll_dice(1, 2) == 1 {
                1
            } else {
                4
            }
        }
        // Large groups either spawn in a small group or as a large group (2-11).
        SpawnsAs::LargeGroup => {
            if rng.roll_dice(1, 2) == 1 {
                4
            } else {
                11
            }
        }
    };
    let roll = if n == 1 { 1 } else { rng.roll_dice(2, n) };
    // We want to constrain group sizes depending on player's level, so
    // we don't get large groups of mobs when the player is unequipped.
    match player_level {
        0..=2 => return i32::max(1, roll / 4),
        3..=4 => return i32::max(1, roll / 2),
        _ => return roll,
    };
}

pub fn get_scroll_tags() -> Vec<String> {
    let raws = &super::RAWS.lock().unwrap();
    let mut result = Vec::new();
    for item in raws.raws.items.iter() {
        if let Some(magic) = &item.magic {
            if &magic.naming == "scroll" {
                result.push(item.name.name.clone());
            }
        }
    }
    return result;
}

pub fn get_potion_tags() -> Vec<String> {
    let raws = &super::RAWS.lock().unwrap();
    let mut result = Vec::new();
    for item in raws.raws.items.iter() {
        if let Some(magic) = &item.magic {
            if &magic.naming == "potion" {
                result.push(item.name.name.clone());
            }
        }
    }
    return result;
}

pub fn get_wand_tags() -> Vec<String> {
    let raws = &super::RAWS.lock().unwrap();
    let mut result = Vec::new();
    for item in raws.raws.items.iter() {
        if let Some(magic) = &item.magic {
            if &magic.naming == "wand" {
                result.push(item.name.name.clone());
            }
        }
    }
    return result;
}

pub fn get_id_from_name(name: String) -> String {
    let raws = &super::RAWS.lock().unwrap();
    for item in &raws.raws.items {
        if item.name.name == name {
            return item.id.clone();
        }
    }
    return "null".to_string();
}

pub fn is_tag_magic(tag: &str) -> bool {
    let raws = &super::RAWS.lock().unwrap();
    if raws.item_index.contains_key(tag) {
        let item_template = &raws.raws.items[raws.item_index[tag]];
        return item_template.magic.is_some();
    } else {
        return false;
    }
}

/// Queries the faction index to obtain one faction's reaction to another faction.
pub fn faction_reaction(this_faction: &str, other_faction: &str, raws: &RawMaster) -> Reaction {
    if raws.faction_index.contains_key(this_faction) {
        let mine = &raws.faction_index[this_faction];
        if mine.contains_key(other_faction) {
            return mine[other_faction];
        } else if mine.contains_key("default") {
            return mine["default"];
        }
    }
    return Reaction::Ignore;
}

pub fn ancestry_reaction(this_ancestry: Ancestry, other_ancestry: Ancestry, raws: &RawMaster) -> Option<Reaction> {
    if this_ancestry == other_ancestry {
        return Some(Reaction::Ignore);
    } else {
        let this_ancestry = get_ancestry_string(this_ancestry);
        let other_ancestry = get_ancestry_string(other_ancestry);
        if raws.ancestry_index.contains_key(this_ancestry) {
            let mine = &raws.ancestry_index[this_ancestry];
            if mine.contains(other_ancestry) {
                return Some(Reaction::Ignore);
            }
        }
    }
    return None;
}

pub fn get_reactions(
    this_entity: Entity,
    other_entity: Entity,
    factions: &ReadStorage<Faction>,
    ancestries: &ReadStorage<HasAncestry>,
    raws: &RawMaster,
) -> Reaction {
    if let Some(this_ancestry) = ancestries.get(this_entity) {
        if let Some(other_ancestry) = ancestries.get(other_entity) {
            let result = ancestry_reaction(this_ancestry.name, other_ancestry.name, raws);
            if result.is_some() {
                return result.unwrap();
            }
        }
    }
    if let Some(this_faction) = factions.get(this_entity) {
        if let Some(other_faction) = factions.get(other_entity) {
            return faction_reaction(&this_faction.name, &other_faction.name, raws);
        }
    }
    return Reaction::Ignore;
}

fn get_ancestry_string(ancestry: Ancestry) -> &'static str {
    match ancestry {
        Ancestry::Human => return "human",
        Ancestry::Elf => return "elf",
        Ancestry::Dwarf => return "dwarf",
        Ancestry::Catfolk => return "catfolk",
        Ancestry::Gnome => return "gnome",
        Ancestry::NULL => return "NULL",
    }
}

fn parse_particle_line(n: &str) -> SpawnParticleLine {
    let tokens: Vec<_> = n.split(';').collect();
    SpawnParticleLine {
        glyph: to_cp437(tokens[0].chars().next().unwrap()),
        tail_glyph: to_cp437(tokens[1].chars().next().unwrap()),
        colour: RGB::from_hex(tokens[2]).expect("Invalid RGB"),
        lifetime_ms: tokens[3].parse::<f32>().unwrap(),
        trail_colour: RGB::from_hex(tokens[4]).expect("Invalid trail RGB"),
        trail_lifetime_ms: tokens[5].parse::<f32>().unwrap(),
    }
}

fn parse_particle(n: &str) -> SpawnParticleSimple {
    let tokens: Vec<_> = n.split(';').collect();
    SpawnParticleSimple {
        glyph: to_cp437(tokens[0].chars().next().unwrap()),
        colour: RGB::from_hex(tokens[1]).expect("Invalid RGB"),
        lifetime_ms: tokens[2].parse::<f32>().unwrap(),
    }
}

fn parse_particle_burst(n: &str) -> SpawnParticleBurst {
    let tokens: Vec<_> = n.split(';').collect();
    SpawnParticleBurst {
        glyph: to_cp437(tokens[0].chars().next().unwrap()),
        head_glyph: to_cp437(tokens[1].chars().next().unwrap()),
        tail_glyph: to_cp437(tokens[2].chars().next().unwrap()),
        colour: RGB::from_hex(tokens[3]).expect("Invalid RGB"),
        lerp: RGB::from_hex(tokens[4]).expect("Invalid LERP RGB"),
        lifetime_ms: tokens[5].parse::<f32>().unwrap(),
        trail_colour: RGB::from_hex(tokens[6]).expect("Invalid trail RGB"),
        trail_lifetime_ms: tokens[7].parse::<f32>().unwrap(),
    }
}
