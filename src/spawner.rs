use super::{
    random_table::RandomTable, Attribute, Attributes, BlocksTile, CombatStats, Confusion, Consumable, Cursed,
    DefenceBonus, Destructible, EntryTrigger, EquipmentSlot, Equippable, Hidden, HungerClock, HungerState,
    InflictsDamage, Item, MagicMapper, Map, MeleePowerBonus, Mind, Monster, Name, Player, Position, ProvidesHealing,
    ProvidesNutrition, Ranged, Rect, Renderable, SerializeMe, SingleActivation, TileType, Viewshed, Wand, AOE,
    MAPWIDTH,
};
use rltk::{console, RandomNumberGenerator, RGB};
use specs::prelude::*;
use specs::saveload::{MarkedBuilder, SimpleMarker};
use std::collections::HashMap;

/// Spawns the player and returns his/her entity object.
pub fn player(ecs: &mut World, player_x: i32, player_y: i32) -> Entity {
    // d8 hit die - but always maxxed at level 1, so player doesn't have to roll.
    ecs.create_entity()
        .with(Position { x: player_x, y: player_y })
        .with(Renderable {
            glyph: rltk::to_cp437('@'),
            fg: RGB::named(rltk::YELLOW),
            bg: RGB::named(rltk::BLACK),
            render_order: 0,
        })
        .with(Player {})
        .with(Viewshed { visible_tiles: Vec::new(), range: 12, dirty: true })
        .with(Name { name: "wanderer".to_string(), plural: "wanderers".to_string() })
        .with(CombatStats { max_hp: 8, hp: 8, defence: 0, power: 4 })
        .with(HungerClock { state: HungerState::Satiated, duration: 50 })
        .with(Attributes {
            strength: Attribute { base: 10, modifiers: 0, bonus: 0 },
            dexterity: Attribute { base: 10, modifiers: 0, bonus: 0 },
            constitution: Attribute { base: 10, modifiers: 0, bonus: 0 },
            intelligence: Attribute { base: 10, modifiers: 0, bonus: 0 },
            wisdom: Attribute { base: 10, modifiers: 0, bonus: 0 },
            charisma: Attribute { base: 10, modifiers: 0, bonus: 0 },
        })
        .marked::<SimpleMarker<SerializeMe>>()
        .build()
}

fn monster<S: ToString>(ecs: &mut World, x: i32, y: i32, glyph: rltk::FontCharType, name: S, hit_die: i32, power: i32) {
    let rolled_hp = roll_hit_dice(ecs, 1, hit_die);

    ecs.create_entity()
        .with(Position { x, y })
        .with(Renderable { glyph: glyph, fg: RGB::named(rltk::GREEN), bg: RGB::named(rltk::BLACK), render_order: 1 })
        .with(Viewshed { visible_tiles: Vec::new(), range: 12, dirty: true })
        .with(Monster {})
        .with(Mind {})
        .with(Name { name: name.to_string(), plural: format!("{}s", name.to_string()) })
        .with(BlocksTile {})
        .with(CombatStats { max_hp: rolled_hp, hp: rolled_hp, defence: 0, power: power })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn orc(ecs: &mut World, x: i32, y: i32) {
    monster(ecs, x, y, rltk::to_cp437('o'), "orc", 8, 3);
}

fn goblin(ecs: &mut World, x: i32, y: i32) {
    monster(ecs, x, y, rltk::to_cp437('g'), "goblin", 6, 2);
}

fn goblin_chieftain(ecs: &mut World, x: i32, y: i32) {
    monster(ecs, x, y, rltk::to_cp437('G'), "goblin chieftain", 8, 3);
}

pub fn roll_hit_dice(ecs: &mut World, n: i32, d: i32) -> i32 {
    let mut rng = ecs.write_resource::<RandomNumberGenerator>();
    let mut rolled_hp: i32 = 0;

    for _i in 0..n {
        rolled_hp += rng.roll_dice(1, d);
    }

    return rolled_hp;
}

// Consts
const MAX_ENTITIES: i32 = 4;

/// Fills a room with stuff!
pub fn spawn_room(
    map: &Map,
    rng: &mut RandomNumberGenerator,
    room: &Rect,
    map_depth: i32,
    spawn_list: &mut Vec<(usize, String)>,
) {
    let mut possible_targets: Vec<usize> = Vec::new();
    {
        // Borrow scope - to keep access to the map separated
        for y in room.y1 + 1..room.y2 {
            for x in room.x1 + 1..room.x2 {
                let idx = map.xy_idx(x, y);
                if map.tiles[idx] == TileType::Floor {
                    possible_targets.push(idx);
                }
            }
        }
    }

    spawn_region(map, rng, &possible_targets, map_depth, spawn_list);
}

pub fn spawn_region(
    _map: &Map,
    rng: &mut RandomNumberGenerator,
    area: &[usize],
    map_depth: i32,
    spawn_list: &mut Vec<(usize, String)>,
) {
    let mut spawn_points: HashMap<usize, String> = HashMap::new();
    let mut areas: Vec<usize> = Vec::from(area);
    let category = category_table().roll(rng);
    let spawn_table;
    match category.as_ref() {
        "mob" => spawn_table = mob_table(map_depth),
        "item" => spawn_table = item_table(map_depth),
        "food" => spawn_table = food_table(map_depth),
        "trap" => spawn_table = trap_table(map_depth),
        _ => spawn_table = debug_table(),
    }

    let num_spawns = i32::min(areas.len() as i32, rng.roll_dice(1, MAX_ENTITIES + 2) - 2);
    if num_spawns <= 0 {
        return;
    }

    for _i in 0..num_spawns {
        let array_idx = if areas.len() == 1 { 0usize } else { (rng.roll_dice(1, areas.len() as i32) - 1) as usize };
        let map_idx = areas[array_idx];
        spawn_points.insert(map_idx, spawn_table.roll(rng));
        areas.remove(array_idx);
    }

    for spawn in spawn_points.iter() {
        spawn_list.push((*spawn.0, spawn.1.to_string()));
    }
}

pub fn spawn_entity(ecs: &mut World, spawn: &(&usize, &String)) {
    let x = (*spawn.0 % MAPWIDTH) as i32;
    let y = (*spawn.0 / MAPWIDTH) as i32;

    match spawn.1.as_ref() {
        // Monsters
        "goblin" => goblin(ecs, x, y),
        "goblin chieftain" => goblin_chieftain(ecs, x, y),
        "orc" => orc(ecs, x, y),
        // Equipment
        "dagger" => dagger(ecs, x, y),
        "shortsword" => shortsword(ecs, x, y),
        "buckler" => buckler(ecs, x, y),
        "shield" => shield(ecs, x, y),
        // Potions
        "weak health potion" => weak_health_potion(ecs, x, y),
        "health potion" => health_potion(ecs, x, y),
        // Scrolls
        "fireball scroll" => fireball_scroll(ecs, x, y),
        "cursed fireball scroll" => cursed_fireball_scroll(ecs, x, y),
        "confusion scroll" => confusion_scroll(ecs, x, y),
        "magic missile scroll" => magic_missile_scroll(ecs, x, y),
        "magic map scroll" => magic_map_scroll(ecs, x, y),
        "cursed magic map scroll" => cursed_magic_map_scroll(ecs, x, y),
        // Wands
        "magic missile wand" => magic_missile_wand(ecs, x, y),
        "fireball wand" => fireball_wand(ecs, x, y),
        "confusion wand" => confusion_wand(ecs, x, y),
        // Food
        "rations" => rations(ecs, x, y),
        // Traps
        "bear trap" => bear_trap(ecs, x, y),
        "confusion trap" => confusion_trap(ecs, x, y),
        _ => console::log(format!("Tried to spawn nothing ({}). Bugfix needed!", spawn.1)),
    }
}

// 20 mobs : 6 items : 2 food : 1 trap
fn category_table() -> RandomTable {
    return RandomTable::new().add("mob", 12).add("item", 6).add("food", 2).add("trap", 1);
}

fn debug_table() -> RandomTable {
    return RandomTable::new().add("debug", 1);
}

// 6 goblins : 1 goblin chief : 2 orcs
fn mob_table(map_depth: i32) -> RandomTable {
    return RandomTable::new()
        // Monsters
        .add("goblin", 6)
        .add("goblin chieftain", 1)
        .add("orc", 2 + map_depth);
}

// 6 equipment : 10 potions : 10 scrolls : 2 cursed scrolls
fn item_table(_map_depth: i32) -> RandomTable {
    return RandomTable::new()
        // Equipment
        .add("dagger", 4)
        .add("shortsword", 4)
        .add("buckler", 4)
        .add("shield", 2)
        // Potions
        .add("weak health potion", 14)
        .add("health potion", 6)
        // Scrolls
        .add("fireball scroll", 2)
        .add("cursed fireball scroll", 2)
        .add("confusion scroll", 4)
        .add("magic missile scroll", 10)
        .add("magic map scroll", 4)
        .add("cursed magic map scroll", 2)
        // Wands
        .add("magic missile wand", 1)
        .add("fireball wand", 1)
        .add("confusion wand", 1);
}

fn food_table(_map_depth: i32) -> RandomTable {
    return RandomTable::new().add("rations", 1);
}

fn trap_table(_map_depth: i32) -> RandomTable {
    return RandomTable::new().add("bear trap", 0).add("confusion trap", 1);
}

fn health_potion(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph: rltk::to_cp437('!'),
            fg: RGB::named(rltk::MAGENTA2),
            bg: RGB::named(rltk::BLACK),
            render_order: 2,
        })
        .with(Name { name: "potion of health".to_string(), plural: "potions of health".to_string() })
        .with(Item {})
        .with(Consumable {})
        .with(Destructible {})
        .with(ProvidesHealing { amount: 12 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn weak_health_potion(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph: rltk::to_cp437('!'),
            fg: RGB::named(rltk::MAGENTA),
            bg: RGB::named(rltk::BLACK),
            render_order: 2,
        })
        .with(Name { name: "potion of lesser health".to_string(), plural: "potions of lesser health".to_string() })
        .with(Item {})
        .with(Consumable {})
        .with(Destructible {})
        .with(ProvidesHealing { amount: 6 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

/*
fn poison_potion(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph: rltk::to_cp437('i'),
            fg: RGB::named(rltk::GREEN),
            bg: RGB::named(rltk::BLACK),
            render_order: 2,
        })
        .with(Name { name: "potion of ... health?".to_string() })
        .with(Item {})
        .with(Consumable {})
        .with(Destructible {})
        .with(ProvidesHealing { amount: -12 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}
*/

// Scrolls
// ~10 range should be considered average here.
fn magic_missile_scroll(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph: rltk::to_cp437('?'),
            fg: RGB::named(rltk::BLUE),
            bg: RGB::named(rltk::BLACK),
            render_order: 2,
        })
        .with(Name { name: "scroll of magic missile".to_string(), plural: "scrolls of magic missile".to_string() })
        .with(Item {})
        .with(Consumable {})
        .with(Destructible {})
        .with(Ranged { range: 12 }) // Long range - as far as default vision range
        .with(InflictsDamage { amount: 10 }) // Low~ damage
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn fireball_scroll(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph: rltk::to_cp437('?'),
            fg: RGB::named(rltk::ORANGE),
            bg: RGB::named(rltk::BLACK),
            render_order: 2,
        })
        .with(Name { name: "scroll of fireball".to_string(), plural: "scrolls of fireball".to_string() })
        .with(Item {})
        .with(Consumable {})
        .with(Destructible {})
        .with(Ranged { range: 10 })
        .with(InflictsDamage { amount: 20 })
        .with(AOE { radius: 3 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn cursed_fireball_scroll(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph: rltk::to_cp437('?'),
            fg: RGB::named(rltk::ORANGE),
            bg: RGB::named(rltk::BLACK),
            render_order: 2,
        })
        .with(Name { name: "cursed scroll of fireball".to_string(), plural: "cursed scrolls of fireball".to_string() })
        .with(Item {})
        .with(Cursed {})
        .with(Consumable {})
        .with(Destructible {})
        .with(Ranged { range: 10 })
        .with(InflictsDamage { amount: 20 })
        .with(AOE { radius: 3 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn confusion_scroll(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph: rltk::to_cp437('?'),
            fg: RGB::named(rltk::PURPLE),
            bg: RGB::named(rltk::BLACK),
            render_order: 2,
        })
        .with(Name { name: "scroll of confusion".to_string(), plural: "scrolls of confusion".to_string() })
        .with(Item {})
        .with(Consumable {})
        .with(Destructible {})
        .with(Ranged { range: 10 })
        .with(Confusion { turns: 4 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn magic_map_scroll(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph: rltk::to_cp437('?'),
            fg: RGB::named(rltk::ROYALBLUE),
            bg: RGB::named(rltk::BLACK),
            render_order: 2,
        })
        .with(Name { name: "scroll of magic mapping".to_string(), plural: "scrolls of magic mapping".to_string() })
        .with(Item {})
        .with(MagicMapper {})
        .with(Consumable {})
        .with(Destructible {})
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn cursed_magic_map_scroll(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph: rltk::to_cp437('?'),
            fg: RGB::named(rltk::ROYALBLUE),
            bg: RGB::named(rltk::BLACK),
            render_order: 2,
        })
        .with(Name {
            name: "cursed scroll of magic mapping".to_string(),
            plural: "cursed scrolls of magic mapping".to_string(),
        })
        .with(Item {})
        .with(Cursed {})
        .with(MagicMapper {})
        .with(Consumable {})
        .with(Destructible {})
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

// EQUIPMENT
fn dagger(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph: rltk::to_cp437(')'),
            fg: RGB::named(rltk::GREY),
            bg: RGB::named(rltk::BLACK),
            render_order: 2,
        })
        .with(Name { name: "dagger".to_string(), plural: "daggers".to_string() })
        .with(Item {})
        .with(Equippable { slot: EquipmentSlot::Melee })
        .with(MeleePowerBonus { amount: 1 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}
fn shortsword(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph: rltk::to_cp437(')'),
            fg: RGB::named(rltk::LIGHTGREY),
            bg: RGB::named(rltk::BLACK),
            render_order: 2,
        })
        .with(Name { name: "shortsword".to_string(), plural: "shortswords".to_string() })
        .with(Item {})
        .with(Equippable { slot: EquipmentSlot::Melee })
        .with(MeleePowerBonus { amount: 2 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn buckler(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph: rltk::to_cp437('['),
            fg: RGB::named(rltk::GREY),
            bg: RGB::named(rltk::BLACK),
            render_order: 2,
        })
        .with(Name { name: "buckler".to_string(), plural: "bucklers".to_string() })
        .with(Item {})
        .with(DefenceBonus { amount: 1 })
        .with(Equippable { slot: EquipmentSlot::Shield })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn shield(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph: rltk::to_cp437('['),
            fg: RGB::named(rltk::LIGHTGREY),
            bg: RGB::named(rltk::BLACK),
            render_order: 2,
        })
        .with(Name { name: "shield".to_string(), plural: "shields".to_string() })
        .with(Item {})
        .with(DefenceBonus { amount: 2 })
        .with(MeleePowerBonus { amount: -1 })
        .with(Equippable { slot: EquipmentSlot::Shield })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

// FOOD

fn rations(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph: rltk::to_cp437('%'),
            fg: RGB::named(rltk::LIGHT_SALMON),
            bg: RGB::named(rltk::BLACK),
            render_order: 2,
        })
        .with(Name { name: "rations".to_string(), plural: "rations".to_string() })
        .with(Item {})
        .with(ProvidesNutrition {})
        .with(Consumable {})
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

// WANDS

fn fireball_wand(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph: rltk::to_cp437('/'),
            fg: RGB::named(rltk::ORANGE),
            bg: RGB::named(rltk::BLACK),
            render_order: 2,
        })
        .with(Name { name: "wand of fireball".to_string(), plural: "wands of fireball".to_string() })
        .with(Item {})
        .with(Wand { uses: 3, max_uses: 3 })
        .with(Destructible {})
        .with(Ranged { range: 10 })
        .with(InflictsDamage { amount: 20 })
        .with(AOE { radius: 3 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn magic_missile_wand(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph: rltk::to_cp437('/'),
            fg: RGB::named(rltk::BLUE),
            bg: RGB::named(rltk::BLACK),
            render_order: 2,
        })
        .with(Name { name: "wand of magic missile".to_string(), plural: "wands of magic missile".to_string() })
        .with(Item {})
        .with(Wand { uses: 3, max_uses: 3 })
        .with(Destructible {})
        .with(Ranged { range: 12 }) // Long range - as far as default vision range
        .with(InflictsDamage { amount: 10 }) // Low~ damage
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn confusion_wand(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph: rltk::to_cp437('/'),
            fg: RGB::named(rltk::PURPLE),
            bg: RGB::named(rltk::BLACK),
            render_order: 2,
        })
        .with(Name { name: "wand of confusion".to_string(), plural: "wands of confusion".to_string() })
        .with(Item {})
        .with(Wand { uses: 3, max_uses: 3 })
        .with(Destructible {})
        .with(Ranged { range: 10 })
        .with(Confusion { turns: 4 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

// TRAPS
fn bear_trap(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph: rltk::to_cp437('^'),
            fg: RGB::named(rltk::GREY),
            bg: RGB::named(rltk::BLACK),
            render_order: 2,
        })
        .with(Name { name: "bear trap".to_string(), plural: "bear traps".to_string() })
        .with(Hidden {})
        .with(EntryTrigger {})
        .with(SingleActivation {})
        .with(InflictsDamage { amount: 6 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn confusion_trap(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph: rltk::to_cp437('^'),
            fg: RGB::named(rltk::PURPLE),
            bg: RGB::named(rltk::BLACK),
            render_order: 2,
        })
        .with(Name { name: "magic trap".to_string(), plural: "magic traps".to_string() })
        .with(Hidden {})
        .with(EntryTrigger {})
        .with(SingleActivation {})
        .with(Confusion { turns: 3 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}
