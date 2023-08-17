use crate::{
    gamelog, gui::obfuscate_name, Charges, EquipmentChanged, InBackpack, MagicItem, MasterDungeonMap, Name,
    ObfuscatedName, Position, WantsToPickupItem,
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
        WriteStorage<'a, EquipmentChanged>,
        ReadStorage<'a, MagicItem>,
        ReadStorage<'a, ObfuscatedName>,
        ReadExpect<'a, MasterDungeonMap>,
        ReadStorage<'a, Charges>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            player_entity,
            mut wants_pickup,
            mut positions,
            names,
            mut backpack,
            mut equipment_changed,
            magic_items,
            obfuscated_names,
            dm,
            wands,
        ) = data;

        for pickup in wants_pickup.join() {
            positions.remove(pickup.item);
            backpack.insert(pickup.item, InBackpack { owner: pickup.collected_by }).expect("Unable to pickup item.");
            equipment_changed
                .insert(pickup.collected_by, EquipmentChanged {})
                .expect("Unable to insert EquipmentChanged.");

            if pickup.collected_by == *player_entity {
                gamelog::Logger::new()
                    .append("You pick up the")
                    .item_name_n(format!(
                        "{}",
                        obfuscate_name(pickup.item, &names, &magic_items, &obfuscated_names, &dm, Some(&wands)).0
                    ))
                    .period()
                    .log();
            }
        }

        wants_pickup.clear();
    }
}
