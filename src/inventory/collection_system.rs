use crate::{
    gamelog,
    gui::obfuscate_name,
    gui::item_colour,
    Beatitude,
    Charges,
    EquipmentChanged,
    InBackpack,
    MagicItem,
    MasterDungeonMap,
    Name,
    ObfuscatedName,
    Position,
    WantsToPickupItem,
};
use specs::prelude::*;
use crate::consts::messages;
use bracket_lib::prelude::*;

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
        ReadStorage<'a, Beatitude>,
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
            beatitudes,
            dm,
            wands,
        ) = data;

        for pickup in wants_pickup.join() {
            positions.remove(pickup.item);
            backpack
                .insert(pickup.item, InBackpack { owner: pickup.collected_by })
                .expect("Unable to pickup item.");
            equipment_changed
                .insert(pickup.collected_by, EquipmentChanged {})
                .expect("Unable to insert EquipmentChanged.");

            if pickup.collected_by == *player_entity {
                gamelog::Logger
                    ::new()
                    .append(messages::YOU_PICKUP_ITEM)
                    .colour(item_colour(pickup.item, &beatitudes))
                    .append_n(
                        format!(
                            "{}",
                            obfuscate_name(
                                pickup.item,
                                &names,
                                &magic_items,
                                &obfuscated_names,
                                &beatitudes,
                                &dm,
                                Some(&wands)
                            ).0
                        )
                    )
                    .colour(WHITE)
                    .period()
                    .log();
            }
        }

        wants_pickup.clear();
    }
}
