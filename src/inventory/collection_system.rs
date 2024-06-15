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
    WantsToAssignKey,
    Renderable,
    Stackable,
};
use specs::prelude::*;
use crate::data::messages;
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
        ReadStorage<'a, Renderable>,
        ReadStorage<'a, Beatitude>,
        ReadExpect<'a, MasterDungeonMap>,
        ReadStorage<'a, Charges>,
        ReadStorage<'a, WantsToAssignKey>,
        ReadStorage<'a, Stackable>,
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
            renderables,
            beatitudes,
            dm,
            wands,
            wants_key,
            stackable,
        ) = data;
        let mut to_remove: Vec<Entity> = Vec::new();
        // For every item that wants to be picked up that *isn't* waiting on a key assignment.
        for (pickup, _key) in (&wants_pickup, !&wants_key).join() {
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
            positions.remove(pickup.item);
            backpack
                .insert(pickup.item, InBackpack { owner: pickup.collected_by })
                .expect("Unable to pickup item");
            equipment_changed
                .insert(pickup.collected_by, EquipmentChanged {})
                .expect("Unable to insert EquipmentChanged");
            to_remove.push(pickup.collected_by);
        }
        for item in to_remove.iter() {
            wants_pickup.remove(*item);
        }
    }
}
