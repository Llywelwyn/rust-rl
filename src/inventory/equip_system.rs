use crate::{
    gamelog,
    gui::{item_colour, obfuscate_name},
    Beatitude, EquipmentChanged, Equippable, Equipped, InBackpack, MagicItem, MasterDungeonMap, Name, ObfuscatedName,
    WantsToUseItem,
};
use specs::prelude::*;

pub struct ItemEquipSystem {}

impl<'a> System<'a> for ItemEquipSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        ReadExpect<'a, Entity>,
        Entities<'a>,
        WriteStorage<'a, WantsToUseItem>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, Equippable>,
        WriteStorage<'a, Equipped>,
        WriteStorage<'a, InBackpack>,
        WriteStorage<'a, EquipmentChanged>,
        ReadStorage<'a, MagicItem>,
        ReadStorage<'a, ObfuscatedName>,
        ReadStorage<'a, Beatitude>,
        ReadExpect<'a, MasterDungeonMap>,
    );

    #[allow(clippy::cognitive_complexity)]
    fn run(&mut self, data: Self::SystemData) {
        let (
            player_entity,
            entities,
            mut wants_to_use_item,
            names,
            equippable,
            mut equipped,
            mut backpack,
            mut dirty,
            magic_items,
            obfuscated_names,
            beatitudes,
            dm,
        ) = data;
        let mut remove: Vec<Entity> = Vec::new();
        // For every item with a target, if the item is equippable, find the correct slot.
        for (target, wants_to_use_item) in (&entities, &wants_to_use_item).join() {
            if let Some(can_equip) = equippable.get(wants_to_use_item.item) {
                let target_slot = can_equip.slot;
                let mut logger = gamelog::Logger::new();

                // Remove any items target has in item's slot
                let mut to_unequip: Vec<Entity> = Vec::new();
                for (item_entity, already_equipped, _name) in (&entities, &equipped, &names).join() {
                    if already_equipped.owner == target && already_equipped.slot == target_slot {
                        to_unequip.push(item_entity);
                    }
                }
                for item in to_unequip.iter() {
                    equipped.remove(*item);
                    backpack.insert(*item, InBackpack { owner: target }).expect("Unable to insert backpack");
                    if target == *player_entity {
                        logger = logger
                            .append("You remove your")
                            .append_n(
                                obfuscate_name(*item, &names, &magic_items, &obfuscated_names, &beatitudes, &dm, None)
                                    .0,
                            )
                            .colour(item_colour(*item, &beatitudes, &dm))
                            .period();
                    }
                }

                // Wield the item
                equipped
                    .insert(wants_to_use_item.item, Equipped { owner: target, slot: target_slot })
                    .expect("Unable to insert equipped component");
                backpack.remove(wants_to_use_item.item);
                if target == *player_entity {
                    logger = logger
                        .append("You equip the")
                        .append_n(
                            obfuscate_name(
                                wants_to_use_item.item,
                                &names,
                                &magic_items,
                                &obfuscated_names,
                                &beatitudes,
                                &dm,
                                None,
                            )
                            .0,
                        )
                        .colour(item_colour(wants_to_use_item.item, &beatitudes, &dm))
                        .period();
                    logger.log();
                }
                remove.push(target);
            }
        }
        remove.iter().for_each(|e| {
            dirty.insert(*e, EquipmentChanged {}).expect("Unabble to insert EquipmentChanged");
            wants_to_use_item.remove(*e).expect("Unable to remove *e");
        })
    }
}
