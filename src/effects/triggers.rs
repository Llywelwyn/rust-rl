use super::{ add_effect, get_noncursed, particles, spatial, EffectType, Entity, Targets, World };
use crate::{
    gamelog,
    gui::item_colour_ecs,
    gui::obfuscate_name_ecs,
    gui::renderable_colour,
    Beatitude,
    Charges,
    Confusion,
    Consumable,
    Destructible,
    Equipped,
    Hidden,
    InBackpack,
    InflictsDamage,
    Item,
    MagicMapper,
    MasterDungeonMap,
    Name,
    ObfuscatedName,
    Player,
    Prop,
    ProvidesHealing,
    ProvidesIdentify,
    ProvidesNutrition,
    ProvidesRemoveCurse,
    RandomNumberGenerator,
    Renderable,
    RunState,
    SingleActivation,
    BUC,
    GrantsSpell,
    KnownSpells,
    Position,
    Viewshed,
    WantsToRemoveKey,
    WantsToDelete,
};
use crate::consts::messages::*;
use bracket_lib::prelude::*;
use specs::prelude::*;
pub fn item_trigger(source: Option<Entity>, item: Entity, target: &Targets, ecs: &mut World) {
    // Check if the item has charges, etc.
    if let Some(has_charges) = ecs.write_storage::<Charges>().get_mut(item) {
        if has_charges.uses <= 0 {
            let mut rng = ecs.write_resource::<RandomNumberGenerator>();
            if rng.roll_dice(1, 121) != 1 {
                gamelog::Logger::new().append(NOCHARGES_DIDNOTHING).log();
                return;
            }
            gamelog::Logger::new().colour(YELLOW).append(NOCHARGES_WREST);
            ecs.write_storage::<Consumable>()
                .insert(item, Consumable {})
                .expect("Could not insert consumable");
        }
        has_charges.uses -= 1;
    }
    // Use the item via the generic system
    let did_something = event_trigger(source, item, target, ecs);
    // If it's a consumable, delete it
    if did_something && ecs.read_storage::<Consumable>().get(item).is_some() {
        let mut removekey = ecs.write_storage::<WantsToRemoveKey>();
        removekey.insert(item, WantsToRemoveKey {}).expect("Unable to insert WantsToRemoveKey");
        let mut delete = ecs.write_storage::<WantsToDelete>();
        delete.insert(item, WantsToDelete {}).expect("Unable to insert WantsToDelete");
    }
}

pub fn trigger(source: Option<Entity>, trigger: Entity, target: &Targets, ecs: &mut World) {
    // Remove hidden from the trigger
    ecs.write_storage::<Hidden>().remove(trigger);
    // Use the trigger via the generic system
    let did_something = event_trigger(source, trigger, target, ecs);
    // If it was single-activation, delete it
    if did_something && ecs.read_storage::<SingleActivation>().get(trigger).is_some() {
        ecs.entities().delete(trigger).expect("Failed to delete entity with a SingleActivation");
    }
}

struct EventInfo {
    source: Option<Entity>,
    entity: Entity,
    target: Targets,
    buc: BUC,
    log: bool,
}

// TODO: Currently, items can only be used by the player, and so this system is only built for that.
//       It does almost no sanity-checking to make sure the logs only appear if the effect is taking
//       place on the player -- once monsters can use an item, their item usage will make logs for
//       the player saying they were the one who used the item. This will need refactoring then.
fn event_trigger(
    source: Option<Entity>,
    entity: Entity,
    target: &Targets,
    ecs: &mut World
) -> bool {
    let buc = if let Some(beatitude) = ecs.read_storage::<Beatitude>().get(entity) {
        beatitude.buc.clone()
    } else {
        BUC::Uncursed
    };
    let mut event = EventInfo { source, entity, target: target.clone(), buc, log: false };
    let logger = gamelog::Logger::new();

    let mut did_something = false;
    particles::handle_simple_particles(ecs, entity, target);
    particles::handle_burst_particles(ecs, entity, &target);
    particles::handle_line_particles(ecs, entity, &target);
    let (logger, restored_nutrition) = handle_restore_nutrition(ecs, &mut event, logger);
    let (logger, magic_mapped) = handle_magic_mapper(ecs, &mut event, logger);
    let (logger, granted_spell) = handle_grant_spell(ecs, &mut event, logger);
    let (logger, removed_curse) = handle_remove_curse(ecs, &mut event, logger);
    let (logger, identified) = handle_identify(ecs, &mut event, logger);
    let (logger, healed) = handle_healing(ecs, &mut event, logger);
    let (logger, damaged) = handle_damage(ecs, &mut event, logger);
    let (logger, confused) = handle_confusion(ecs, &mut event, logger);
    //let (logger, dug) = handle_dig(ecs, &mut event, logger); -- NYI i.e. Wand of Digging
    did_something |=
        restored_nutrition ||
        magic_mapped ||
        granted_spell ||
        healed ||
        damaged ||
        confused ||
        removed_curse ||
        identified;

    if event.log {
        logger.log();
    }

    return did_something;
}

fn handle_restore_nutrition(
    ecs: &mut World,
    event: &mut EventInfo,
    mut logger: gamelog::Logger
) -> (gamelog::Logger, bool) {
    if ecs.read_storage::<ProvidesNutrition>().get(event.entity).is_some() {
        let amount = match event.buc {
            BUC::Blessed => 600,
            BUC::Uncursed => 400,
            BUC::Cursed => 200,
        };
        add_effect(event.source, EffectType::ModifyNutrition { amount }, event.target.clone());
        logger = logger
            .append(NUTRITION)
            .colour(item_colour_ecs(ecs, event.entity))
            .append_n(obfuscate_name_ecs(ecs, event.entity).0)
            .colour(WHITE)
            .period()
            .buc(event.buc.clone(), Some(NUTRITION_CURSED), Some(NUTRITION_BLESSED));
        event.log = true;
        return (logger, true);
    }
    return (logger, false);
}

fn handle_magic_mapper(
    ecs: &mut World,
    event: &mut EventInfo,
    mut logger: gamelog::Logger
) -> (gamelog::Logger, bool) {
    if ecs.read_storage::<MagicMapper>().get(event.entity).is_some() {
        let mut runstate = ecs.fetch_mut::<RunState>();
        let cursed = if event.buc == BUC::Cursed { true } else { false };
        *runstate = RunState::MagicMapReveal { row: 0, cursed: cursed };
        logger = logger.append(MAGICMAP).buc(event.buc.clone(), Some(MAGICMAP_CURSED), None);
        event.log = true;
        return (logger, true);
    }
    return (logger, false);
}

fn handle_grant_spell(
    ecs: &mut World,
    event: &mut EventInfo,
    logger: gamelog::Logger
) -> (gamelog::Logger, bool) {
    if let Some(_granted_spell) = ecs.read_storage::<GrantsSpell>().get(event.entity) {
        if
            let Some(_known_spells) = ecs
                .write_storage::<KnownSpells>()
                .get_mut(event.source.unwrap())
        {
            // TODO: Check if the player knows *this* spell, and add it if not.
        } else {
            // TODO: Grant the KnownSpells component, and then add the spell.
        }
    }
    return (logger, false);
}

fn handle_healing(
    ecs: &mut World,
    event: &mut EventInfo,
    mut logger: gamelog::Logger
) -> (gamelog::Logger, bool) {
    if let Some(healing_item) = ecs.read_storage::<ProvidesHealing>().get(event.entity) {
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();
        let buc_mod = match event.buc {
            BUC::Blessed => 2,
            BUC::Cursed => -1,
            _ => 0,
        };
        let roll =
            rng.roll_dice(healing_item.n_dice + buc_mod, healing_item.sides) +
            healing_item.modifier;
        add_effect(
            event.source,
            EffectType::Healing { amount: roll, increment_max: get_noncursed(&event.buc) },
            event.target.clone()
        );
        for target in get_entity_targets(&event.target) {
            if
                ecs.read_storage::<Prop>().get(target).is_some() ||
                ecs.read_storage::<Item>().get(target).is_some()
            {
                continue;
            }
            let renderables = ecs.read_storage::<Renderable>();
            if ecs.read_storage::<Player>().get(target).is_some() {
                logger = logger
                    .colour(renderable_colour(&renderables, target))
                    .append("You")
                    .colour(WHITE)
                    .append(HEAL_PLAYER_HIT)
                    .buc(event.buc.clone(), None, Some(HEAL_PLAYER_HIT_BLESSED));
            } else {
                logger = logger
                    .append("The")
                    .colour(renderable_colour(&renderables, target))
                    .append(obfuscate_name_ecs(ecs, target).0)
                    .colour(WHITE)
                    .append(HEAL_OTHER_HIT);
            }
            event.log = true;
        }
        return (logger, true);
    }
    return (logger, false);
}

fn handle_damage(
    ecs: &mut World,
    event: &mut EventInfo,
    mut logger: gamelog::Logger
) -> (gamelog::Logger, bool) {
    if let Some(damage_item) = ecs.read_storage::<InflictsDamage>().get(event.entity) {
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();
        let roll = rng.roll_dice(damage_item.n_dice, damage_item.sides) + damage_item.modifier;
        add_effect(
            event.source,
            EffectType::Damage { amount: roll, damage_type: damage_item.damage_type },
            event.target.clone()
        );
        for target in get_entity_targets(&event.target) {
            if ecs.read_storage::<Prop>().get(target).is_some() {
                continue;
            }
            let renderables = ecs.read_storage::<Renderable>();
            let positions = ecs.read_storage::<Position>();
            let target_pos = positions.get(target).unwrap_or(&(Position { x: 0, y: 0 }));
            let viewsheds = ecs.read_storage::<Viewshed>();
            let player_viewshed = viewsheds.get(*ecs.fetch::<Entity>()).unwrap();
            if ecs.read_storage::<Player>().get(target).is_some() {
                logger = logger
                    .colour(renderable_colour(&renderables, target))
                    .append("You")
                    .colour(WHITE)
                    .append(DAMAGE_PLAYER_HIT);
                event.log = true;
            } else if
                player_viewshed.visible_tiles.contains(&Point::new(target_pos.x, target_pos.y))
            {
                if ecs.read_storage::<Item>().get(target).is_some() {
                    if ecs.read_storage::<Destructible>().get(target).is_some() {
                        logger = logger
                            .append("The")
                            .colour(renderable_colour(&renderables, target))
                            .append(obfuscate_name_ecs(ecs, target).0)
                            .colour(WHITE)
                            .append(DAMAGE_ITEM_HIT);
                    }
                } else {
                    logger = logger
                        .append("The")
                        .colour(renderable_colour(&renderables, target))
                        .append(obfuscate_name_ecs(ecs, target).0)
                        .colour(WHITE)
                        .append(DAMAGE_OTHER_HIT);
                }
                event.log = true;
            }
        }
        return (logger, true);
    }
    return (logger, false);
}

#[allow(unused_mut)]
fn handle_confusion(
    ecs: &mut World,
    event: &mut EventInfo,
    mut logger: gamelog::Logger
) -> (gamelog::Logger, bool) {
    if let Some(confusion) = ecs.read_storage::<Confusion>().get(event.entity) {
        add_effect(
            event.source,
            EffectType::Confusion { turns: confusion.turns },
            event.target.clone()
        );
        return (logger, true);
    }
    return (logger, false);
}

fn select_single(ecs: &World, runstate: RunState) {
    let mut new_runstate = ecs.fetch_mut::<RunState>();
    *new_runstate = runstate;
}

fn handle_identify(
    ecs: &mut World,
    event: &mut EventInfo,
    mut logger: gamelog::Logger
) -> (gamelog::Logger, bool) {
    if let Some(_i) = ecs.read_storage::<ProvidesIdentify>().get(event.entity) {
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();
        let mut dm = ecs.fetch_mut::<MasterDungeonMap>();
        let identify_all = match event.buc {
            BUC::Blessed => rng.roll_dice(1, 5) == 1,
            BUC::Uncursed => rng.roll_dice(1, 25) == 1,
            _ => false,
        };
        if !identify_all {
            select_single(ecs, RunState::ShowIdentify);
            return (logger, true);
        }
        let mut to_identify: Vec<(Entity, String)> = Vec::new();
        let mut beatitudes = ecs.write_storage::<Beatitude>();
        let obfuscated = ecs.read_storage::<ObfuscatedName>();
        for (e, _i, _bp, name) in (
            &ecs.entities(),
            &ecs.read_storage::<Item>(),
            &ecs.read_storage::<InBackpack>(),
            &ecs.read_storage::<Name>(),
        )
            .join()
            .filter(|(e, _i, bp, name)| {
                let in_this_backpack = bp.owner == event.source.unwrap();
                let has_obfuscated_name = obfuscated.get(*e).is_some();
                let already_identified = dm.identified_items.contains(&name.name.clone());
                let known_beatitude = beatitudes
                    .get(event.source.unwrap())
                    .map(|b| b.known)
                    .unwrap_or(true);
                let result =
                    in_this_backpack &&
                    (has_obfuscated_name || !already_identified || !known_beatitude);
                return result;
            }) {
            to_identify.push((e, name.name.clone()));
        }
        for item in to_identify {
            dm.identified_items.insert(item.1);
            if let Some(beatitude) = beatitudes.get_mut(item.0) {
                beatitude.known = true;
            }
        }
        logger = logger
            .append(IDENTIFY_ALL)
            .buc(event.buc.clone(), None, Some(IDENTIFY_ALL_BLESSED));
        event.log = true;
        return (logger, true);
    }
    return (logger, false);
}

fn handle_remove_curse(
    ecs: &mut World,
    event: &mut EventInfo,
    mut logger: gamelog::Logger
) -> (gamelog::Logger, bool) {
    if let Some(_r) = ecs.read_storage::<ProvidesRemoveCurse>().get(event.entity) {
        let mut to_decurse: Vec<Entity> = Vec::new();
        match event.buc {
            // If cursed, show the prompt to select one item.
            BUC::Cursed => {
                select_single(ecs, RunState::ShowRemoveCurse);
                return (logger, true);
            }
            // If blessed, decurse everything in our backpack.
            BUC::Blessed => {
                for (entity, _i, _bp, _b) in (
                    &ecs.entities(),
                    &ecs.read_storage::<Item>(),
                    &ecs.read_storage::<InBackpack>(),
                    &ecs.read_storage::<Beatitude>(),
                )
                    .join()
                    .filter(
                        |(_e, _i, bp, b)| bp.owner == event.source.unwrap() && b.buc == BUC::Cursed
                    ) {
                    to_decurse.push(entity);
                }
            }
            _ => {}
        }
        // If noncursed, decurse everything we have equipped.
        for (entity, _i, _e, _b) in (
            &ecs.entities(),
            &ecs.read_storage::<Item>(),
            &ecs.read_storage::<Equipped>(),
            &ecs.read_storage::<Beatitude>(),
        )
            .join()
            .filter(|(_e, _i, e, b)| e.owner == event.source.unwrap() && b.buc == BUC::Cursed) {
            to_decurse.push(entity);
        }
        if to_decurse.len() == 0 {
            match event.buc {
                BUC::Uncursed => select_single(ecs, RunState::ShowRemoveCurse),
                BUC::Blessed => {
                    logger = logger.append(REMOVECURSE_BLESSED_FAILED);
                }
                _ => {}
            }
            return (logger, true);
        }
        let mut beatitudes = ecs.write_storage::<Beatitude>();
        for e in to_decurse {
            beatitudes
                .insert(e, Beatitude { buc: BUC::Uncursed, known: true })
                .expect("Unable to insert beatitude");
        }
        logger = logger.append(REMOVECURSE).buc(event.buc.clone(), None, Some(REMOVECURSE_BLESSED));
        event.log = true;
        return (logger, true);
    }
    return (logger, false);
}

fn get_entity_targets(target: &Targets) -> Vec<Entity> {
    let mut entities: Vec<Entity> = Vec::new();
    match target {
        Targets::Entity { target } => entities.push(*target),
        Targets::EntityList { targets } => targets.iter().for_each(|target| entities.push(*target)),
        Targets::Tile { target } => {
            spatial::for_each_tile_content(*target, |entity| {
                entities.push(entity);
            });
        }
        Targets::TileList { targets } => {
            targets.iter().for_each(|target| {
                spatial::for_each_tile_content(*target, |entity| {
                    entities.push(entity);
                });
            });
        }
    }
    return entities;
}
