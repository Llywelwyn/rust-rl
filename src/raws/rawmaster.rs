use super::Raws;
use crate::components::*;
use crate::random_table::RandomTable;
use specs::prelude::*;
use std::collections::HashMap;

pub enum SpawnType {
    AtPosition { x: i32, y: i32 },
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
        self.item_index = HashMap::new();
        for (i, item) in self.raws.items.iter().enumerate() {
            self.item_index.insert(item.id.clone(), i);
        }
        for (i, mob) in self.raws.mobs.iter().enumerate() {
            self.mob_index.insert(mob.id.clone(), i);
        }
        for (i, prop) in self.raws.props.iter().enumerate() {
            self.prop_index.insert(prop.id.clone(), i);
        }
        for (i, table) in self.raws.spawn_tables.iter().enumerate() {
            self.table_index.insert(table.id.clone(), i);
        }
    }
}

pub fn spawn_named_entity(
    raws: &RawMaster,
    new_entity: EntityBuilder,
    key: &str,
    pos: SpawnType,
    rng: &mut rltk::RandomNumberGenerator,
) -> Option<Entity> {
    if raws.item_index.contains_key(key) {
        return spawn_named_item(raws, new_entity, key, pos);
    } else if raws.mob_index.contains_key(key) {
        return spawn_named_mob(raws, new_entity, key, pos, rng);
    } else if raws.prop_index.contains_key(key) {
        return spawn_named_prop(raws, new_entity, key, pos);
    }
    None
}

pub fn spawn_named_item(raws: &RawMaster, new_entity: EntityBuilder, key: &str, pos: SpawnType) -> Option<Entity> {
    if raws.item_index.contains_key(key) {
        let item_template = &raws.raws.items[raws.item_index[key]];
        let mut eb = new_entity;

        eb = eb.with(Name { name: item_template.name.name.clone(), plural: item_template.name.plural.clone() });
        eb = eb.with(Item {});
        eb = spawn_position(pos, eb);

        if let Some(renderable) = &item_template.renderable {
            eb = eb.with(get_renderable_component(renderable));
        }

        if let Some(flags) = &item_template.flags {
            for flag in flags.iter() {
                match flag.as_str() {
                    "CONSUMABLE" => eb = eb.with(Consumable {}),
                    "DESTRUCTIBLE" => eb = eb.with(Destructible {}),
                    "CURSED" => eb = eb.with(Cursed {}),
                    "EQUIP_MELEE" => eb = eb.with(Equippable { slot: EquipmentSlot::Melee }),
                    "EQUIP_SHIELD" => eb = eb.with(Equippable { slot: EquipmentSlot::Shield }),
                    "WAND" => eb = eb.with(Wand { uses: 3, max_uses: 3 }),
                    "FOOD" => eb = eb.with(ProvidesNutrition {}),
                    _ => rltk::console::log(format!("Unrecognised flag: {}", flag.as_str())),
                }
            }
        }

        if let Some(effects_list) = &item_template.effects {
            for effect in effects_list.iter() {
                let effect_name = effect.0.as_str();
                match effect_name {
                    "provides_healing" => eb = eb.with(ProvidesHealing { amount: effect.1.parse::<i32>().unwrap() }),
                    "ranged" => eb = eb.with(Ranged { range: effect.1.parse::<i32>().unwrap() }),
                    "damage" => eb = eb.with(InflictsDamage { amount: effect.1.parse::<i32>().unwrap() }),
                    "aoe" => eb = eb.with(AOE { radius: effect.1.parse::<i32>().unwrap() }),
                    "confusion" => eb = eb.with(Confusion { turns: effect.1.parse::<i32>().unwrap() }),
                    "melee_power_bonus" => eb = eb.with(MeleePowerBonus { amount: effect.1.parse::<i32>().unwrap() }),
                    "defence_bonus" => eb = eb.with(DefenceBonus { amount: effect.1.parse::<i32>().unwrap() }),
                    "magicmapper" => eb = eb.with(MagicMapper {}),
                    "digger" => eb = eb.with(Digger {}),
                    _ => rltk::console::log(format!("Warning: effect {} not implemented.", effect_name)),
                }
            }
        }

        return Some(eb.build());
    }
    None
}

pub fn spawn_named_mob(
    raws: &RawMaster,
    new_entity: EntityBuilder,
    key: &str,
    pos: SpawnType,
    rng: &mut rltk::RandomNumberGenerator,
) -> Option<Entity> {
    if raws.mob_index.contains_key(key) {
        let mob_template = &raws.raws.mobs[raws.mob_index[key]];

        // New entity with a position, name, combatstats, and viewshed
        let mut eb = new_entity;
        eb = spawn_position(pos, eb);
        eb = eb.with(Name { name: mob_template.name.clone(), plural: mob_template.name.clone() });
        eb = eb.with(Monster {});
        let rolled_hp = roll_hit_dice(rng, 1, mob_template.stats.max_hp);
        eb = eb.with(CombatStats {
            max_hp: rolled_hp,
            hp: rolled_hp,
            power: mob_template.stats.power,
            defence: mob_template.stats.defence,
        });
        eb = eb.with(Viewshed { visible_tiles: Vec::new(), range: mob_template.vision_range, dirty: true });

        if let Some(renderable) = &mob_template.renderable {
            eb = eb.with(get_renderable_component(renderable));
        }

        if let Some(flags) = &mob_template.flags {
            for flag in flags.iter() {
                match flag.as_str() {
                    "BLOCKS_TILE" => eb = eb.with(BlocksTile {}),
                    _ => rltk::console::log(format!("Unrecognised flag: {}", flag.as_str())),
                }
            }
        }

        return Some(eb.build());
    }
    None
}

pub fn roll_hit_dice(rng: &mut rltk::RandomNumberGenerator, n: i32, d: i32) -> i32 {
    let mut rolled_hp: i32 = 0;

    for _i in 0..n {
        rolled_hp += rng.roll_dice(1, d);
    }

    return rolled_hp;
}

pub fn spawn_named_prop(raws: &RawMaster, new_entity: EntityBuilder, key: &str, pos: SpawnType) -> Option<Entity> {
    if raws.prop_index.contains_key(key) {
        let prop_template = &raws.raws.props[raws.prop_index[key]];

        let mut eb = new_entity;
        eb = spawn_position(pos, eb);
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

fn spawn_position(pos: SpawnType, new_entity: EntityBuilder) -> EntityBuilder {
    let mut eb = new_entity;

    match pos {
        SpawnType::AtPosition { x, y } => {
            eb = eb.with(Position { x, y });
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

pub fn table_by_name(raws: &RawMaster, key: &str, depth: i32) -> RandomTable {
    if raws.table_index.contains_key(key) {
        let spawn_table = &raws.raws.spawn_tables[raws.table_index[key]];

        use super::SpawnTableEntry;

        let available_options: Vec<&SpawnTableEntry> =
            spawn_table.table.iter().filter(|a| depth >= a.min && depth <= a.max).collect();

        let mut rt = RandomTable::new();
        for e in available_options.iter() {
            rt = rt.add(e.id.clone(), e.weight);
        }

        return rt;
    } else {
        return RandomTable::new().add("debug", 1);
    }
}
