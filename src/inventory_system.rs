use super::{
    gamelog, CombatStats, Confusion, Consumable, Cursed, Destructible, InBackpack, InflictsDamage, MagicMapper, Map,
    Name, ParticleBuilder, Point, Position, ProvidesHealing, RunState, SufferDamage, WantsToDropItem,
    WantsToPickupItem, WantsToUseItem, AOE, DEFAULT_PARTICLE_LIFETIME, LONG_PARTICLE_LIFETIME,
};
use specs::prelude::*;

pub struct ItemCollectionSystem {}

impl<'a> System<'a> for ItemCollectionSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        ReadExpect<'a, Entity>,
        WriteStorage<'a, WantsToPickupItem>,
        WriteStorage<'a, Position>,
        ReadStorage<'a, Name>,
        WriteStorage<'a, InBackpack>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (player_entity, mut wants_pickup, mut positions, names, mut backpack) = data;

        for pickup in wants_pickup.join() {
            positions.remove(pickup.item);
            backpack.insert(pickup.item, InBackpack { owner: pickup.collected_by }).expect("Unable to pickup item.");

            if pickup.collected_by == *player_entity {
                gamelog::Logger::new()
                    .append("You pick up the")
                    .item_name(format!("{}.", &names.get(pickup.item).unwrap().name))
                    .log();
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
        ReadExpect<'a, Map>,
        Entities<'a>,
        WriteStorage<'a, WantsToUseItem>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, Consumable>,
        ReadStorage<'a, Destructible>,
        ReadStorage<'a, Cursed>,
        ReadStorage<'a, ProvidesHealing>,
        WriteStorage<'a, CombatStats>,
        WriteStorage<'a, SufferDamage>,
        WriteExpect<'a, ParticleBuilder>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, InflictsDamage>,
        ReadStorage<'a, AOE>,
        WriteStorage<'a, Confusion>,
        ReadStorage<'a, MagicMapper>,
        WriteExpect<'a, RunState>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            player_entity,
            map,
            entities,
            mut wants_to_use,
            names,
            consumables,
            destructibles,
            cursed_items,
            provides_healing,
            mut combat_stats,
            mut suffer_damage,
            mut particle_builder,
            positions,
            inflicts_damage,
            aoe,
            mut confused,
            magic_mapper,
            mut runstate,
        ) = data;

        for (entity, wants_to_use) in (&entities, &wants_to_use).join() {
            let mut used_item = true;
            let mut aoe_item = false;
            let item_being_used = names.get(wants_to_use.item).unwrap();

            let is_cursed = cursed_items.get(wants_to_use.item);

            gamelog::Logger::new().append("You use the").item_name(format!("{}.", &item_being_used.name)).log();

            // TARGETING
            let mut targets: Vec<Entity> = Vec::new();
            match wants_to_use.target {
                None => {
                    targets.push(*player_entity);
                }
                Some(mut target) => {
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
                            // If item with a targeted AOE is cursed, get the position
                            // of the player and set them to be the new target.
                            match is_cursed {
                                None => {}
                                Some(_) => {
                                    let pos = positions.get(*player_entity);
                                    if let Some(pos) = pos {
                                        target = Point::new(pos.x, pos.y);
                                    }
                                    gamelog::Logger::new()
                                        .append("The")
                                        .item_name(&item_being_used.name)
                                        .colour(rltk::WHITE)
                                        .append("disobeys!")
                                        .log();
                                }
                            }
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
                                    LONG_PARTICLE_LIFETIME,
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
                                gamelog::Logger::new()
                                    .append("Quaffing, you heal")
                                    .colour(rltk::GREEN)
                                    .append(heal.amount)
                                    .colour(rltk::WHITE)
                                    .append("hit points.")
                                    .log();
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
                                    gamelog::Logger::new()
                                        .append("The")
                                        .npc_name(&entity_name.name)
                                        .colour(rltk::WHITE)
                                        .append("takes")
                                        .damage(damage.amount)
                                        .colour(rltk::WHITE)
                                        .append("damage from the")
                                        .item_name(format!("{}.", &item_being_used.name))
                                        .log();
                                }
                            }
                            Some(_destructible) => {
                                gamelog::Logger::new()
                                    .append("The")
                                    .item_name(&entity_name.name)
                                    .colour(rltk::WHITE)
                                    .append("is destroyed!")
                                    .log();
                                entities.delete(*mob).expect("Delete failed");
                            }
                        }

                        used_item = true;
                    }
                }
            }

            // CONFUSION
            let mut add_confusion = Vec::new();
            {
                let causes_confusion = confused.get(wants_to_use.item);
                match causes_confusion {
                    None => {}
                    Some(confusion) => {
                        for mob in targets.iter() {
                            add_confusion.push((*mob, confusion.turns));
                            // Gamelog entry for this is handled turn-by-turn in AI.
                        }
                    }
                }
            }
            for mob in add_confusion.iter() {
                confused.insert(mob.0, Confusion { turns: mob.1 }).expect("Unable to insert status");
            }

            // MAGIC MAPPERS
            let is_mapper = magic_mapper.get(wants_to_use.item);
            match is_mapper {
                None => {}
                Some(_) => {
                    used_item = true;
                    match is_cursed {
                        None => {
                            gamelog::Logger::new()
                                .append("You feel")
                                .colour(rltk::GREEN)
                                .append("a sense of acuity towards your surroundings.")
                                .log();
                            *runstate = RunState::MagicMapReveal { row: 0, cursed: false };
                        }
                        Some(_) => {
                            gamelog::Logger::new()
                                .append("You")
                                .colour(rltk::RED)
                                .append("forget where you last were.")
                                .log();
                            *runstate = RunState::MagicMapReveal { row: 0, cursed: true };
                        }
                    }
                }
            }

            // ITEM DELETION AFTER USE
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
        Entities<'a>,
        WriteStorage<'a, WantsToDropItem>,
        ReadStorage<'a, Name>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, InBackpack>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (player_entity, entities, mut wants_drop, names, mut positions, mut backpack) = data;

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
                gamelog::Logger::new()
                    .append("You drop the")
                    .item_name(format!("{}.", &names.get(to_drop.item).unwrap().name))
                    .log();
            }
        }

        wants_drop.clear();
    }
}
