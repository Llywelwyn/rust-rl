use super::{
    gamelog::GameLog, CombatStats, Consumable, Destructible, InBackpack, InflictsDamage, Map, Name, ParticleBuilder,
    Position, ProvidesHealing, SufferDamage, WantsToDropItem, WantsToPickupItem, WantsToUseItem, AOE,
    DEFAULT_PARTICLE_LIFETIME,
};
use specs::prelude::*;

pub struct ItemCollectionSystem {}

impl<'a> System<'a> for ItemCollectionSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        ReadExpect<'a, Entity>,
        WriteExpect<'a, GameLog>,
        WriteStorage<'a, WantsToPickupItem>,
        WriteStorage<'a, Position>,
        ReadStorage<'a, Name>,
        WriteStorage<'a, InBackpack>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (player_entity, mut gamelog, mut wants_pickup, mut positions, names, mut backpack) = data;

        for pickup in wants_pickup.join() {
            positions.remove(pickup.item);
            backpack.insert(pickup.item, InBackpack { owner: pickup.collected_by }).expect("Unable to pickup item.");

            if pickup.collected_by == *player_entity {
                gamelog.entries.push(format!("You pick up the {}.", names.get(pickup.item).unwrap().name));
            }
        }

        wants_pickup.clear();
    }
}

pub struct ItemUseSystem {}
impl<'a> System<'a> for ItemUseSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        ReadExpect<'a, Entity>,
        WriteExpect<'a, GameLog>,
        ReadExpect<'a, Map>,
        Entities<'a>,
        WriteStorage<'a, WantsToUseItem>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, Consumable>,
        ReadStorage<'a, Destructible>,
        ReadStorage<'a, ProvidesHealing>,
        WriteStorage<'a, CombatStats>,
        WriteStorage<'a, SufferDamage>,
        WriteExpect<'a, ParticleBuilder>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, InflictsDamage>,
        ReadStorage<'a, AOE>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            player_entity,
            mut gamelog,
            map,
            entities,
            mut wants_to_use,
            names,
            consumables,
            destructibles,
            provides_healing,
            mut combat_stats,
            mut suffer_damage,
            mut particle_builder,
            positions,
            inflicts_damage,
            aoe,
        ) = data;

        for (entity, wants_to_use) in (&entities, &wants_to_use).join() {
            let mut used_item = true;
            let mut aoe_item = false;
            let item_being_used = names.get(wants_to_use.item).unwrap();

            // TARGETING
            let mut targets: Vec<Entity> = Vec::new();
            match wants_to_use.target {
                None => {
                    targets.push(*player_entity);
                }
                Some(target) => {
                    let area_effect = aoe.get(wants_to_use.item);
                    match area_effect {
                        None => {
                            // Single target in a tile
                            let idx = map.xy_idx(target.x, target.y);
                            for mob in map.tile_content[idx].iter() {
                                targets.push(*mob);
                            }
                        }
                        Some(area_effect) => {
                            // AOE
                            aoe_item = true;
                            let mut blast_tiles = rltk::field_of_view(target, area_effect.radius, &*map);
                            blast_tiles.retain(|p| p.x > 0 && p.x < map.width - 1 && p.y > 0 && p.y < map.height - 1);
                            for tile_idx in blast_tiles.iter() {
                                let idx = map.xy_idx(tile_idx.x, tile_idx.y);
                                for mob in map.tile_content[idx].iter() {
                                    targets.push(*mob);
                                }
                                particle_builder.request(
                                    tile_idx.x,
                                    tile_idx.y,
                                    rltk::RGB::named(rltk::ORANGE),
                                    rltk::RGB::named(rltk::BLACK),
                                    rltk::to_cp437('░'),
                                    200.0,
                                );
                            }
                        }
                    }
                }
            }

            // HEALING ITEM
            let item_heals = provides_healing.get(wants_to_use.item);
            match item_heals {
                None => {}
                Some(heal) => {
                    for target in targets.iter() {
                        let stats = combat_stats.get_mut(*target);
                        if let Some(stats) = stats {
                            stats.hp = i32::min(stats.max_hp, stats.hp + heal.amount);
                            if entity == *player_entity {
                                gamelog.entries.push(format!(
                                    "You quaff the {}, and heal {} hp.",
                                    item_being_used.name, heal.amount
                                ));
                            }
                            let pos = positions.get(entity);
                            if let Some(pos) = pos {
                                particle_builder.request(
                                    pos.x,
                                    pos.y,
                                    rltk::RGB::named(rltk::GREEN),
                                    rltk::RGB::named(rltk::BLACK),
                                    rltk::to_cp437('♥'),
                                    DEFAULT_PARTICLE_LIFETIME,
                                );
                            }
                        }
                    }
                }
            }

            // DAMAGING ITEM
            let item_damages = inflicts_damage.get(wants_to_use.item);
            match item_damages {
                None => {}
                Some(damage) => {
                    let target_point = wants_to_use.target.unwrap();
                    gamelog.entries.push(format!("You use the {}!", item_being_used.name));
                    if !aoe_item {
                        particle_builder.request_star(
                            target_point.x,
                            target_point.y,
                            rltk::RGB::named(rltk::CYAN),
                            rltk::RGB::named(rltk::BLACK),
                            rltk::to_cp437('*'),
                            DEFAULT_PARTICLE_LIFETIME,
                        );
                    }
                    for mob in targets.iter() {
                        let destructible = destructibles.get(*mob);
                        let entity_name = names.get(*mob).unwrap();
                        match destructible {
                            None => {
                                SufferDamage::new_damage(&mut suffer_damage, *mob, damage.amount);
                                if entity == *player_entity {
                                    gamelog.entries.push(format!(
                                        "{} takes {} damage from the {}!",
                                        entity_name.name, damage.amount, item_being_used.name
                                    ));
                                }
                            }
                            Some(_destructible) => {
                                gamelog.entries.push(format!("{} is destroyed!", entity_name.name));
                                entities.delete(*mob).expect("Delete failed");
                            }
                        }

                        used_item = true;
                    }
                }
            }
            if used_item {
                let consumable = consumables.get(wants_to_use.item);
                match consumable {
                    None => {}
                    Some(_) => {
                        entities.delete(wants_to_use.item).expect("Delete failed");
                    }
                }
            }
        }
        wants_to_use.clear();
    }
}

pub struct ItemDropSystem {}

impl<'a> System<'a> for ItemDropSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        ReadExpect<'a, Entity>,
        WriteExpect<'a, GameLog>,
        Entities<'a>,
        WriteStorage<'a, WantsToDropItem>,
        ReadStorage<'a, Name>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, InBackpack>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (player_entity, mut gamelog, entities, mut wants_drop, names, mut positions, mut backpack) = data;

        for (entity, to_drop) in (&entities, &wants_drop).join() {
            let mut dropper_pos: Position = Position { x: 0, y: 0 };
            {
                let dropped_pos = positions.get(entity).unwrap();
                dropper_pos.x = dropped_pos.x;
                dropper_pos.y = dropped_pos.y;
            }
            positions
                .insert(to_drop.item, Position { x: dropper_pos.x, y: dropper_pos.y })
                .expect("Failed to insert position.");
            backpack.remove(to_drop.item);

            if entity == *player_entity {
                gamelog.entries.push(format!("You drop the {}.", names.get(to_drop.item).unwrap().name));
            }
        }

        wants_drop.clear();
    }
}
