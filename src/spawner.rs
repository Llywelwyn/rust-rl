use super::{
    BlocksTile, CombatStats, Confusion, Consumable, Destructible, InflictsDamage, Item, Monster, Name, Player,
    Position, ProvidesHealing, Ranged, Rect, Renderable, SerializeMe, Viewshed, AOE, MAPWIDTH,
};
use rltk::{RandomNumberGenerator, RGB};
use specs::prelude::*;
use specs::saveload::{MarkedBuilder, SimpleMarker};

/// Spawns the player and returns his/her entity object.
pub fn player(ecs: &mut World, player_x: i32, player_y: i32) -> Entity {
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
        .with(Name { name: "hero (you)".to_string() })
        .with(CombatStats { max_hp: 30, hp: 30, defence: 2, power: 5 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build()
}

/// Spawns a random monster at a given loc
pub fn random_monster(ecs: &mut World, x: i32, y: i32) {
    let roll: i32;
    {
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();
        roll = rng.roll_dice(1, 3);
    }
    match roll {
        1 => orc(ecs, x, y),
        2 => goblin_chieftain(ecs, x, y),
        _ => goblin(ecs, x, y),
    }
}
/// Spawns a random item at a given loc
pub fn random_item(ecs: &mut World, x: i32, y: i32) {
    let roll: i32;
    {
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();
        roll = rng.roll_dice(1, 6);
    }
    match roll {
        1 => health_potion(ecs, x, y),
        2 => poison_potion(ecs, x, y),
        3 => magic_missile_scroll(ecs, x, y),
        4 => fireball_scroll(ecs, x, y),
        5 => confusion_scroll(ecs, x, y),
        _ => weak_health_potion(ecs, x, y),
    }
}

const MAX_MONSTERS: i32 = 4;
const MAX_ITEMS: i32 = 3;

fn monster<S: ToString>(ecs: &mut World, x: i32, y: i32, glyph: rltk::FontCharType, name: S) {
    ecs.create_entity()
        .with(Position { x, y })
        .with(Renderable { glyph: glyph, fg: RGB::named(rltk::GREEN), bg: RGB::named(rltk::BLACK), render_order: 1 })
        .with(Viewshed { visible_tiles: Vec::new(), range: 12, dirty: true })
        .with(Monster {})
        .with(Name { name: name.to_string() })
        .with(BlocksTile {})
        .with(CombatStats { max_hp: 16, hp: 16, defence: 1, power: 4 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn orc(ecs: &mut World, x: i32, y: i32) {
    monster(ecs, x, y, rltk::to_cp437('o'), "orc");
}

fn goblin(ecs: &mut World, x: i32, y: i32) {
    monster(ecs, x, y, rltk::to_cp437('g'), "goblin");
}

fn goblin_chieftain(ecs: &mut World, x: i32, y: i32) {
    monster(ecs, x, y, rltk::to_cp437('G'), "goblin chieftain");
}

pub fn spawn_room(ecs: &mut World, room: &Rect) {
    let mut monster_spawn_points: Vec<usize> = Vec::new();
    let mut item_spawn_points: Vec<usize> = Vec::new();

    // Scope for borrow checker
    {
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();
        let num_monsters = rng.roll_dice(1, MAX_MONSTERS + 2) - 3;
        let num_items = rng.roll_dice(1, MAX_ITEMS + 2) - 3;

        for _i in 0..num_monsters {
            let mut added = false;
            while !added {
                let x = (room.x1 + rng.roll_dice(1, i32::abs(room.x2 - room.x1))) as usize;
                let y = (room.y1 + rng.roll_dice(1, i32::abs(room.y2 - room.y1))) as usize;
                let idx = (y * MAPWIDTH) + x;
                if !monster_spawn_points.contains(&idx) {
                    monster_spawn_points.push(idx);
                    added = true;
                }
            }
        }

        for _i in 0..num_items {
            let mut added = false;
            while !added {
                let x = (room.x1 + rng.roll_dice(1, i32::abs(room.x2 - room.x1))) as usize;
                let y = (room.y1 + rng.roll_dice(1, i32::abs(room.y2 - room.y1))) as usize;
                let idx = (y * MAPWIDTH) + x;
                if !item_spawn_points.contains(&idx) {
                    item_spawn_points.push(idx);
                    added = true;
                }
            }
        }
    }

    // Spawn
    for idx in monster_spawn_points.iter() {
        let x = *idx % MAPWIDTH;
        let y = *idx / MAPWIDTH;
        random_monster(ecs, x as i32, y as i32);
    }
    for idx in item_spawn_points.iter() {
        let x = *idx % MAPWIDTH;
        let y = *idx / MAPWIDTH;
        random_item(ecs, x as i32, y as i32);
    }
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