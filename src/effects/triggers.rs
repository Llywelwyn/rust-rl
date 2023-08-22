use super::{add_effect, get_noncursed, particles, spatial, EffectType, Entity, Targets, World};
use crate::{
    gamelog, gui::item_colour_ecs, gui::obfuscate_name_ecs, gui::renderable_colour, Beatitude, Charges, Confusion,
    Consumable, Destructible, Equipped, Hidden, InBackpack, InflictsDamage, Item, MagicMapper, Player, Prop,
    ProvidesHealing, ProvidesNutrition, RandomNumberGenerator, RemovesCurse, Renderable, RunState, SingleActivation,
    BUC,
};
use rltk::prelude::*;
use specs::prelude::*;
pub fn item_trigger(source: Option<Entity>, item: Entity, target: &Targets, ecs: &mut World) {
    // Check if the item has charges, etc.
    if let Some(has_charges) = ecs.write_storage::<Charges>().get_mut(item) {
        if has_charges.uses <= 0 {
            let mut rng = ecs.write_resource::<RandomNumberGenerator>();
            if rng.roll_dice(1, 121) != 1 {
                gamelog::Logger::new().append("The wand does nothing.").log();
                return;
            }
            gamelog::Logger::new().colour(rltk::YELLOW).append("You wrest one last charge from the worn-out wand.");
            ecs.write_storage::<Consumable>().insert(item, Consumable {}).expect("Could not insert consumable");
        }
        has_charges.uses -= 1;
    }
    // Use the item via the generic system
    let did_something = event_trigger(source, item, target, ecs);
    // If it's a consumable, delete it
    if did_something && ecs.read_storage::<Consumable>().get(item).is_some() {
        ecs.entities().delete(item).expect("Failed to delete item");
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
fn event_trigger(source: Option<Entity>, entity: Entity, target: &Targets, ecs: &mut World) -> bool {
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
    let (logger, removed_curse) = handle_remove_curse(ecs, &mut event, logger);
    let (logger, healed) = handle_healing(ecs, &mut event, logger);
    let (logger, damaged) = handle_damage(ecs, &mut event, logger);
    let (logger, confused) = handle_confusion(ecs, &mut event, logger);
    //let (logger, dug) = handle_dig(ecs, &mut event, logger); -- NYI i.e. Wand of Digging
    did_something |= restored_nutrition || magic_mapped || healed || damaged || confused || removed_curse;

    if event.log {
        logger.log();
    }

    return did_something;
}

fn handle_restore_nutrition(
    ecs: &mut World,
    event: &mut EventInfo,
    mut logger: gamelog::Logger,
) -> (gamelog::Logger, bool) {
    if ecs.read_storage::<ProvidesNutrition>().get(event.entity).is_some() {
        let amount = match event.buc {
            BUC::Blessed => 600,
            BUC::Uncursed => 400,
            BUC::Cursed => 200,
        };
        add_effect(event.source, EffectType::ModifyNutrition { amount }, event.target.clone());
        logger = logger
            .append("You eat the")
            .colour(item_colour_ecs(ecs, event.entity))
            .append_n(obfuscate_name_ecs(ecs, event.entity).0)
            .colour(WHITE)
            .period()
            .buc(event.buc.clone(), Some("Blech! Rotten"), Some("Delicious"));
        event.log = true;
        return (logger, true);
    }
    return (logger, false);
}

fn handle_magic_mapper(ecs: &mut World, event: &mut EventInfo, mut logger: gamelog::Logger) -> (gamelog::Logger, bool) {
    if ecs.read_storage::<MagicMapper>().get(event.entity).is_some() {
        let mut runstate = ecs.fetch_mut::<RunState>();
        let cursed = if event.buc == BUC::Cursed { true } else { false };
        *runstate = RunState::MagicMapReveal { row: 0, cursed: cursed };
        logger = logger.append("You recall your surroundings!").buc(
            event.buc.clone(),
            Some("... but forget where you last were."),
            None,
        );
        event.log = true;
        return (logger, true);
    }
    return (logger, false);
}

fn handle_healing(ecs: &mut World, event: &mut EventInfo, mut logger: gamelog::Logger) -> (gamelog::Logger, bool) {
    if let Some(healing_item) = ecs.read_storage::<ProvidesHealing>().get(event.entity) {
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();
        let buc_mod = match event.buc {
            BUC::Blessed => 2,
            BUC::Cursed => -1,
            _ => 0,
        };
        let roll = rng.roll_dice(healing_item.n_dice + buc_mod, healing_item.sides) + healing_item.modifier;
        add_effect(
            event.source,
            EffectType::Healing { amount: roll, increment_max: get_noncursed(&event.buc) },
            event.target.clone(),
        );
        for target in get_entity_targets(&event.target) {
            if ecs.read_storage::<Prop>().get(target).is_some() || ecs.read_storage::<Item>().get(target).is_some() {
                continue;
            }
            let renderables = ecs.read_storage::<Renderable>();
            if ecs.read_storage::<Player>().get(target).is_some() {
                logger = logger
                    .colour(renderable_colour(&renderables, target))
                    .append("You")
                    .colour(WHITE)
                    .append("recover some vigour.")
                    .buc(event.buc.clone(), None, Some("You feel great!"));
            } else {
                logger = logger
                    .append("The")
                    .colour(renderable_colour(&renderables, target))
                    .append(obfuscate_name_ecs(ecs, target).0)
                    .colour(WHITE)
                    .append("is rejuvenated!");
            }
            event.log = true;
        }
        return (logger, true);
    }
    return (logger, false);
}

fn handle_damage(ecs: &mut World, event: &mut EventInfo, mut logger: gamelog::Logger) -> (gamelog::Logger, bool) {
    if let Some(damage_item) = ecs.read_storage::<InflictsDamage>().get(event.entity) {
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();
        let roll = rng.roll_dice(damage_item.n_dice, damage_item.sides) + damage_item.modifier;
        add_effect(event.source, EffectType::Damage { amount: roll }, event.target.clone());
        for target in get_entity_targets(&event.target) {
            if ecs.read_storage::<Prop>().get(target).is_some() {
                continue;
            }
            let renderables = ecs.read_storage::<Renderable>();
            if ecs.read_storage::<Player>().get(target).is_some() {
                logger = logger
                    .colour(renderable_colour(&renderables, target))
                    .append("You")
                    .colour(WHITE)
                    .append("are hit!");
            } else if ecs.read_storage::<Item>().get(target).is_some() {
                if ecs.read_storage::<Destructible>().get(target).is_some() {
                    logger = logger
                        .append("The")
                        .colour(renderable_colour(&renderables, target))
                        .append(obfuscate_name_ecs(ecs, target).0)
                        .colour(WHITE)
                        .append("is ruined!");
                }
            } else {
                logger = logger
                    .append("The")
                    .colour(renderable_colour(&renderables, target))
                    .append(obfuscate_name_ecs(ecs, target).0)
                    .colour(WHITE)
                    .append("is hit!");
            }
            event.log = true;
        }
        return (logger, true);
    }
    return (logger, false);
}

#[allow(unused_mut)]
fn handle_confusion(ecs: &mut World, event: &mut EventInfo, mut logger: gamelog::Logger) -> (gamelog::Logger, bool) {
    if let Some(confusion) = ecs.read_storage::<Confusion>().get(event.entity) {
        add_effect(event.source, EffectType::Confusion { turns: confusion.turns }, event.target.clone());
        return (logger, true);
    }
    return (logger, false);
}

fn select_single_remove_curse(ecs: &World) {
    let mut runstate = ecs.fetch_mut::<RunState>();
    *runstate = RunState::ShowRemoveCurse;
}

fn handle_remove_curse(ecs: &mut World, event: &mut EventInfo, mut logger: gamelog::Logger) -> (gamelog::Logger, bool) {
    if let Some(_r) = ecs.read_storage::<RemovesCurse>().get(event.entity) {
        let mut to_decurse: Vec<Entity> = Vec::new();
        let mut cursed_in_backpack = false;
        let mut cursed_in_inventory = false;
        match event.buc {
            // If cursed, show the prompt to select one item.
            BUC::Cursed => {
                select_single_remove_curse(ecs);
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
                    .filter(|(_e, _i, bp, b)| bp.owner == event.source.unwrap() && b.buc == BUC::Cursed)
                {
                    cursed_in_backpack = true;
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
            .filter(|(_e, _i, e, b)| e.owner == event.source.unwrap() && b.buc == BUC::Cursed)
        {
            cursed_in_inventory = true;
            to_decurse.push(entity);
        }
        if to_decurse.len() == 0 {
            match event.buc {
                BUC::Uncursed => select_single_remove_curse(ecs),
                BUC::Blessed => logger = logger.append("You feel righteous! ... but nothing happens."),
                _ => {}
            }
            return (logger, true);
        }
        let mut beatitudes = ecs.write_storage::<Beatitude>();
        for e in to_decurse {
            beatitudes.insert(e, Beatitude { buc: BUC::Uncursed, known: true }).expect("Unable to insert beatitude");
        }
        logger =
            logger.append("You feel a reassuring presence.").buc(event.buc.clone(), None, Some("You feel righteous!"));
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
