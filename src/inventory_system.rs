use super::{
    gamelog::GameLog, CombatStats, Consumable, InBackpack, Name, Position, ProvidesHealing, WantsToDropItem,
    WantsToPickupItem, WantsToUseItem,
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
        Entities<'a>,
        WriteStorage<'a, WantsToUseItem>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, Consumable>,
        ReadStorage<'a, ProvidesHealing>,
        WriteStorage<'a, CombatStats>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (player_entity, mut gamelog, entities, mut wants_use, names, consumables, healing, mut combat_stats) = data;

        for (entity, use_item, stats) in (&entities, &wants_use, &mut combat_stats).join() {
            let item_heals = healing.get(use_item.item);
            match item_heals {
                None => {}
                Some(healer) => {
                    stats.hp = i32::min(stats.max_hp, stats.hp + healer.heal_amount);
                    if entity == *player_entity {
                        gamelog.entries.push(format!(
                            "You quaff the {}, and heal {} hp.",
                            names.get(use_item.item).unwrap().name,
                            healer.heal_amount
                        ));
                    }
                    let consumable = consumables.get(use_item.item);
                    match consumable {
                        None => {}
                        Some(_) => {
                            entities.delete(use_item.item).expect("Delete failed");
                        }
                    }
                }
            }
        }
        wants_use.clear();
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
