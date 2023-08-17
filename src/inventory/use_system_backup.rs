use super::{
    gamelog, Confusion, Consumable, Cursed, Destructible, Digger, HungerClock, HungerState, IdentifiedItem,
    InflictsDamage, MagicMapper, Map, Name, ParticleBuilder, Point, Pools, Position, ProvidesHealing,
    ProvidesNutrition, RandomNumberGenerator, RunState, SufferDamage, TileType, Viewshed, Wand, WantsToUseItem, AOE,
    DEFAULT_PARTICLE_LIFETIME, LONG_PARTICLE_LIFETIME,
};
use specs::prelude::*;

// Grouping together components because of type complexity issues - SystemData was too large.
// This is a temporary solution that'll be fixed once inventory use is refactored into separate
// systems.
type NameComponents<'a> = (ReadStorage<'a, Name>, WriteStorage<'a, IdentifiedItem>);

pub struct ItemUseSystem {}
impl<'a> System<'a> for ItemUseSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        ReadExpect<'a, Entity>,
        WriteExpect<'a, Map>,
        WriteExpect<'a, RandomNumberGenerator>,
        Entities<'a>,
        WriteStorage<'a, WantsToUseItem>,
        NameComponents<'a>,
        WriteStorage<'a, Consumable>,
        WriteStorage<'a, Wand>,
        ReadStorage<'a, Destructible>,
        ReadStorage<'a, Cursed>,
        ReadStorage<'a, ProvidesHealing>,
        ReadStorage<'a, ProvidesNutrition>,
        WriteStorage<'a, HungerClock>,
        WriteStorage<'a, Pools>,
        WriteStorage<'a, SufferDamage>,
        WriteExpect<'a, ParticleBuilder>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, InflictsDamage>,
        ReadStorage<'a, AOE>,
        ReadStorage<'a, Digger>,
        WriteStorage<'a, Confusion>,
        ReadStorage<'a, MagicMapper>,
        WriteExpect<'a, RunState>,
        WriteStorage<'a, Viewshed>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            player_entity,
            mut map,
            mut rng,
            entities,
            mut wants_to_use,
            (names, mut identified_items),
            mut consumables,
            mut wands,
            destructibles,
            cursed_items,
            provides_healing,
            provides_nutrition,
            mut hunger_clock,
            mut combat_stats,
            mut suffer_damage,
            mut particle_builder,
            positions,
            inflicts_damage,
            aoe,
            digger,
            mut confused,
            magic_mapper,
            mut runstate,
            mut viewsheds,
        ) = data;

        for (entity, wants_to_use) in (&entities, &wants_to_use).join() {
            let mut verb = "use";
            let mut used_item = true;
            let mut aoe_item = false;

            let mut logger = gamelog::Logger::new();

            let is_cursed = cursed_items.get(wants_to_use.item);
            let wand = wands.get_mut(wants_to_use.item);
            if let Some(wand) = wand {
                // If want has no uses, roll 1d121. On a 121, wrest the wand, then delete it.
                if wand.uses == 0 {
                    if rng.roll_dice(1, 121) != 121 {
                        gamelog::Logger::new().append("The wand does nothing.").log();
                        break;
                    }
                    logger = logger.colour(rltk::YELLOW).append("You wrest one last charge from the worn-out wand.");
                    consumables.insert(wants_to_use.item, Consumable {}).expect("Could not insert consumable");
                }
                verb = "zap";
                wand.uses -= 1;
            }

            let item_being_used = names.get(wants_to_use.item).unwrap();

            let is_edible = provides_nutrition.get(wants_to_use.item);
            if let Some(_) = is_edible {
                verb = "eat";
            }

            logger =
                logger.append(format!("You {} the", verb)).item_name_n(format!("{}", &item_being_used.name)).period();

            // TARGETING
            let mut targets: Vec<Entity> = Vec::new();
            let mut target_idxs: Vec<usize> = Vec::new();
            match wants_to_use.target {
                None => {
                    targets.push(*player_entity);
                    let pos = positions.get(*player_entity);
                    if let Some(pos) = pos {
                        target_idxs.push(map.xy_idx(pos.x, pos.y));
                    }
                }
                Some(mut target) => {
                    let area_effect = aoe.get(wants_to_use.item);
                    match area_effect {
                        None => {
                            // Single target in a tile
                            let idx = map.xy_idx(target.x, target.y);
                            target_idxs.push(idx);
                            crate::spatial::for_each_tile_content(idx, |mob| targets.push(mob));
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
                                    logger = logger
                                        .append("The")
                                        .item_name(&item_being_used.name)
                                        .colour(rltk::WHITE)
                                        .append("disobeys!");
                                }
                            }
                            // AOE
                            aoe_item = true;
                            let mut blast_tiles = rltk::field_of_view(target, area_effect.radius, &*map);
                            blast_tiles.retain(|p| p.x > 0 && p.x < map.width - 1 && p.y > 0 && p.y < map.height - 1);
                            for tile_idx in blast_tiles.iter() {
                                let idx = map.xy_idx(tile_idx.x, tile_idx.y);
                                target_idxs.push(idx);
                                crate::spatial::for_each_tile_content(idx, |mob| targets.push(mob));
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

            // EDIBLE
            match is_edible {
                None => {}
                Some(_) => {
                    let target = targets[0];
                    let hc = hunger_clock.get_mut(target);
                    if let Some(hc) = hc {
                        hc.state = HungerState::Satiated;
                        hc.duration = 200;
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
                            let roll = rng.roll_dice(heal.n_dice, heal.sides) + heal.modifier;
                            stats.hit_points.current = i32::min(stats.hit_points.max, stats.hit_points.current + roll);
                            if entity == *player_entity {
                                logger = logger.append("You recover some vigour.");
                            }
                            let pos = positions.get(entity);
                            if let Some(pos) = pos {
                                particle_builder.heal(pos.x, pos.y);
                            }
                        }
                    }
                }
            }

            let mut damage_logger = gamelog::Logger::new();
            let mut needs_damage_log = false;

            // DAMAGING ITEM
            let item_damages = inflicts_damage.get(wants_to_use.item);
            match item_damages {
                None => {}
                Some(damage) => {
                    let target_point = wants_to_use.target.unwrap();
                    let damage_roll = rng.roll_dice(damage.n_dice, damage.sides) + damage.modifier;
                    if !aoe_item {
                        particle_builder.request_rainbow_star(
                            target_point.x,
                            target_point.y,
                            rltk::to_cp437('*'),
                            DEFAULT_PARTICLE_LIFETIME,
                        );
                    }
                    for mob in targets.iter() {
                        let destructible = destructibles.get(*mob);
                        let entity_name = names.get(*mob).unwrap();
                        match destructible {
                            None => {
                                SufferDamage::new_damage(&mut suffer_damage, *mob, damage_roll, true);
                                if entity == *player_entity {
                                    damage_logger =
                                        damage_logger.append("The").npc_name(&entity_name.name).append("is hit!");
                                    needs_damage_log = true;
                                }
                            }
                            Some(_destructible) => {
                                damage_logger = damage_logger
                                    .append("The")
                                    .item_name(&entity_name.name)
                                    .colour(rltk::WHITE)
                                    .append("is destroyed!");
                                needs_damage_log = true;
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
                            logger = logger
                                .append("You feel")
                                .colour(rltk::GREEN)
                                .append("a sense of acuity towards your surroundings.");
                            *runstate = RunState::MagicMapReveal { row: 0, cursed: false };
                        }
                        Some(_) => {
                            logger = logger.append("You").colour(rltk::RED).append("forget where you last were.");
                            *runstate = RunState::MagicMapReveal { row: 0, cursed: true };
                        }
                    }
                }
            }

            let is_digger = digger.get(wants_to_use.item);
            match is_digger {
                None => {}
                Some(_) => {
                    used_item = true;
                    for idx in target_idxs {
                        if map.tiles[idx] == TileType::Wall {
                            map.tiles[idx] = TileType::Floor;
                        }
                        for viewshed in (&mut viewsheds).join() {
                            if viewshed
                                .visible_tiles
                                .contains(&Point::new(idx % map.width as usize, idx / map.width as usize))
                            {
                                viewshed.dirty = true;
                            }
                        }
                    }
                }
            }

            // ITEM DELETION AFTER USE
            if used_item {
                // Identify
                if entity == *player_entity {
                    identified_items
                        .insert(entity, IdentifiedItem { name: item_being_used.name.clone() })
                        .expect("Unable to insert");
                }
                let consumable = consumables.get(wants_to_use.item);
                match consumable {
                    None => {}
                    Some(_) => {
                        entities.delete(wants_to_use.item).expect("Delete failed");
                    }
                }
            }

            logger.log();
            if needs_damage_log {
                damage_logger.log();
            }
        }
        wants_to_use.clear();
    }
}
