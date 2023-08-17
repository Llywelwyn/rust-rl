use crate::{
    gamelog, gui::obfuscate_name, EquipmentChanged, InBackpack, MagicItem, MasterDungeonMap, Name, ObfuscatedName,
    Position, Wand, WantsToDropItem,
};
use specs::prelude::*;

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
        ReadStorage<'a, ObfuscatedName>,
        ReadExpect<'a, MasterDungeonMap>,
        ReadStorage<'a, Wand>,
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
                gamelog::Logger::new()
                    .append("You drop the")
                    .item_name_n(format!(
                        "{}",
                        obfuscate_name(to_drop.item, &names, &magic_items, &obfuscated_names, &dm, Some(&wands)).0
                    ))
                    .period()
                    .log();
            }
        }

        wants_drop.clear();
    }
}
