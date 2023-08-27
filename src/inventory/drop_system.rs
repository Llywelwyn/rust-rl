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
    WantsToDropItem,
};
use specs::prelude::*;
use crate::data::messages;

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
        WriteStorage<'a, EquipmentChanged>,
        ReadStorage<'a, MagicItem>,
        ReadStorage<'a, Beatitude>,
        ReadStorage<'a, ObfuscatedName>,
        ReadExpect<'a, MasterDungeonMap>,
        ReadStorage<'a, Charges>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            player_entity,
            entities,
            mut wants_drop,
            names,
            mut positions,
            mut backpack,
            mut equipment_changed,
            magic_items,
            beatitudes,
            obfuscated_names,
            dm,
            wands,
        ) = data;

        for (entity, to_drop) in (&entities, &wants_drop).join() {
            equipment_changed.insert(entity, EquipmentChanged {}).expect("Unable to insert EquipmentChanged.");
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
                gamelog::Logger
                    ::new()
                    .append(messages::YOU_DROP_ITEM)
                    .colour(item_colour(to_drop.item, &beatitudes))
                    .append_n(
                        format!(
                            "{}",
                            obfuscate_name(
                                to_drop.item,
                                &names,
                                &magic_items,
                                &obfuscated_names,
                                &beatitudes,
                                &dm,
                                Some(&wands)
                            ).0
                        )
                    )
                    .colour(rltk::WHITE)
                    .period()
                    .log();
            }
        }

        wants_drop.clear();
    }
}
