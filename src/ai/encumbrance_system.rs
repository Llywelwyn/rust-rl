use crate::{gamelog, Attributes, Burden, EquipmentChanged, Equipped, InBackpack, Item, Pools};
use specs::prelude::*;
use std::collections::HashMap;

pub const CARRY_CAPACITY_PER_STRENGTH: i32 = 8;

pub struct EncumbranceSystem {}

impl<'a> System<'a> for EncumbranceSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        WriteStorage<'a, EquipmentChanged>,
        Entities<'a>,
        ReadStorage<'a, Item>,
        ReadStorage<'a, InBackpack>,
        ReadStorage<'a, Equipped>,
        WriteStorage<'a, Pools>,
        ReadStorage<'a, Attributes>,
        ReadExpect<'a, Entity>,
        WriteStorage<'a, Burden>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut equip_dirty, entities, items, backpacks, wielded, mut pools, attributes, player, mut burdened) = data;
        if equip_dirty.is_empty() {
            return;
        }
        // Build update map
        let mut to_update: HashMap<Entity, f32> = HashMap::new();
        for (entity, _dirty) in (&entities, &equip_dirty).join() {
            to_update.insert(entity, 0.0);
        }
        equip_dirty.clear();
        // Total up equipped items
        for (item, equipped) in (&items, &wielded).join() {
            if to_update.contains_key(&equipped.owner) {
                let totals = to_update.get_mut(&equipped.owner).unwrap();
                *totals += item.weight;
            }
        }
        // Total carried items
        for (item, carried) in (&items, &backpacks).join() {
            if to_update.contains_key(&carried.owner) {
                let totals = to_update.get_mut(&carried.owner).unwrap();
                *totals += item.weight;
            }
        }
        // Apply to pools
        for (entity, weight) in to_update.iter() {
            if let Some(pool) = pools.get_mut(*entity) {
                pool.weight = *weight;
                if let Some(attr) = attributes.get(*entity) {
                    let carry_capacity_lbs =
                        (attr.strength.base + attr.strength.modifiers) * CARRY_CAPACITY_PER_STRENGTH;
                    if pool.weight as i32 > 3 * carry_capacity_lbs {
                        // Overloaded
                        burdened
                            .insert(*entity, Burden { level: crate::BurdenLevel::Overloaded })
                            .expect("Failed to insert Burden");
                        if *entity == *player {
                            gamelog::Logger::new().append("You're overloaded!").log();
                        }
                    } else if pool.weight as i32 > 2 * carry_capacity_lbs {
                        // Strained
                        burdened
                            .insert(*entity, Burden { level: crate::BurdenLevel::Strained })
                            .expect("Failed to insert Burden");
                        if *entity == *player {
                            gamelog::Logger::new().append("You're strained.").log();
                        }
                    } else if pool.weight as i32 > carry_capacity_lbs {
                        // Burdened
                        burdened
                            .insert(*entity, Burden { level: crate::BurdenLevel::Burdened })
                            .expect("Failed to insert Burden");
                        if *entity == *player {
                            gamelog::Logger::new().append("You're burdened.").log();
                        }
                    } else {
                        burdened.remove(*entity);
                    }
                }
            }
        }
    }
}
