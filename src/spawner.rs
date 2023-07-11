use super::{
    random_table::RandomTable, BlocksTile, CombatStats, Confusion, Consumable, Cursed, Destructible, InflictsDamage,
    Item, MagicMapper, Monster, Name, Player, Position, ProvidesHealing, Ranged, Rect, Renderable, SerializeMe,
    Viewshed, AOE, MAPWIDTH,
};
use rltk::{RandomNumberGenerator, RGB};
use specs::prelude::*;
use specs::saveload::{MarkedBuilder, SimpleMarker};
use std::collections::HashMap;

/// Spawns the player and returns his/her entity object.
pub fn player(ecs: &mut World, player_x: i32, player_y: i32, player_name: String) -> Entity {
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
        .with(Name { name: player_name })
        .with(CombatStats { max_hp: 8, hp: 8, defence: 0, power: 4 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build()
}

fn monster<S: ToString>(ecs: &mut World, x: i32, y: i32, glyph: rltk::FontCharType, name: S, hit_die: i32) {
    let rolled_hp = roll_hit_dice(ecs, 1, hit_die);

    ecs.create_entity()
        .with(Position { x, y })
        .with(Renderable { glyph: glyph, fg: RGB::named(rltk::GREEN), bg: RGB::named(rltk::BLACK), render_order: 1 })
        .with(Viewshed { visible_tiles: Vec::new(), range: 12, dirty: true })
        .with(Monster {})
        .with(Name { name: name.to_string() })
        .with(BlocksTile {})
        .with(CombatStats { max_hp: rolled_hp, hp: rolled_hp, defence: 0, power: 2 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn orc(ecs: &mut World, x: i32, y: i32) {
    monster(ecs, x, y, rltk::to_cp437('o'), "orc", 8);
}

fn goblin(ecs: &mut World, x: i32, y: i32) {
    monster(ecs, x, y, rltk::to_cp437('g'), "goblin", 6);
}

fn goblin_chieftain(ecs: &mut World, x: i32, y: i32) {
    monster(ecs, x, y, rltk::to_cp437('G'), "goblin chieftain", 8);
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

#[allow(clippy::map_entry)]
pub fn spawn_room(ecs: &mut World, room: &Rect, map_depth: i32) {
    let spawn_table = room_table(map_depth);
    let mut spawn_points: HashMap<usize, String> = HashMap::new();

    // Scope for borrow checker
    {
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();
        let num_spawns = rng.roll_dice(1, MAX_ENTITIES + 3) + (map_depth - 1) - 3;
        // With a MAX_ENTITIES of 4, this means each room has between:
        // d1:  -2 to 4
        // d2:  -1 to 5
        // d3:   0 to 6
        // etc.

        for _i in 0..num_spawns {
            let mut added = false;
            let mut tries = 0;
            while !added && tries < 20 {
                let x = (room.x1 + rng.roll_dice(1, i32::abs(room.x2 - room.x1))) as usize;
                let y = (room.y1 + rng.roll_dice(1, i32::abs(room.y2 - room.y1))) as usize;
                let idx = (y * MAPWIDTH) + x;
                if !spawn_points.contains_key(&idx) {
                    spawn_points.insert(idx, spawn_table.roll(&mut rng));
                    added = true;
                } else {
                    tries += 1;
                }
            }
        }
    }

    // Spawn
    for spawn in spawn_points.iter() {
        let x = (*spawn.0 % MAPWIDTH) as i32;
        let y = (*spawn.0 / MAPWIDTH) as i32;

        match spawn.1.as_ref() {
            // Monsters
            "goblin" => goblin(ecs, x, y),
            "goblin chieftain" => goblin_chieftain(ecs, x, y),
            "orc" => orc(ecs, x, y),
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
            _ => {}
        }
    }
}

fn room_table(map_depth: i32) -> RandomTable {
    return RandomTable::new()
        // Monsters
        .add("goblin", 15)
        .add("goblin chieftain", 2 + map_depth)
        .add("orc", 4 + map_depth)
        // Potions
        .add("weak health potion", 4)
        .add("health potion", 1 + (map_depth / 2))
        // Scrolls
        .add("fireball scroll", 1 + (map_depth / 3))
        .add("cursed fireball scroll", 1)
        .add("confusion scroll", 2)
        .add("magic missile scroll", 4)
        .add("magic map scroll", 2)
        .add("cursed magic map scroll", 1);
}

fn health_potion(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph: rltk::to_cp437('I'),
            fg: RGB::named(rltk::MAGENTA),
            bg: RGB::named(rltk::BLACK),
            render_order: 2,
        })
        .with(Name { name: "potion of health".to_string() })
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
            glyph: rltk::to_cp437('i'),
            fg: RGB::named(rltk::MAGENTA),
            bg: RGB::named(rltk::BLACK),
            render_order: 2,
        })
        .with(Name { name: "potion of lesser health".to_string() })
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
            glyph: rltk::to_cp437(')'),
            fg: RGB::named(rltk::BLUE),
            bg: RGB::named(rltk::BLACK),
            render_order: 2,
        })
        .with(Name { name: "scroll of magic missile".to_string() })
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
            glyph: rltk::to_cp437(')'),
            fg: RGB::named(rltk::ORANGE),
            bg: RGB::named(rltk::BLACK),
            render_order: 2,
        })
        .with(Name { name: "scroll of fireball".to_string() })
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
            glyph: rltk::to_cp437(')'),
            fg: RGB::named(rltk::ORANGE),
            bg: RGB::named(rltk::BLACK),
            render_order: 2,
        })
        .with(Name { name: "cursed scroll of fireball".to_string() })
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
            glyph: rltk::to_cp437(')'),
            fg: RGB::named(rltk::PURPLE),
            bg: RGB::named(rltk::BLACK),
            render_order: 2,
        })
        .with(Name { name: "scroll of confusion".to_string() })
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
            glyph: rltk::to_cp437(')'),
            fg: RGB::named(rltk::ROYALBLUE),
            bg: RGB::named(rltk::BLACK),
            render_order: 2,
        })
        .with(Name { name: "scroll of magic mapping".to_string() })
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
            glyph: rltk::to_cp437(')'),
            fg: RGB::named(rltk::ROYALBLUE),
            bg: RGB::named(rltk::BLACK),
            render_order: 2,
        })
        .with(Name { name: "cursed scroll of magic mapping".to_string() })
        .with(Item {})
        .with(Cursed {})
        .with(MagicMapper {})
        .with(Consumable {})
        .with(Destructible {})
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}
