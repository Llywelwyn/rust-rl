use crate::{
    gamelog,
    gui::{ item_colour, obfuscate_name },
    Beatitude,
    Equipped,
    InBackpack,
    MagicItem,
    MasterDungeonMap,
    Name,
    ObfuscatedName,
    WantsToRemoveItem,
    BUC,
};
use bracket_lib::prelude::*;
use specs::prelude::*;
use crate::data::messages;

pub struct ItemRemoveSystem {}

impl<'a> System<'a> for ItemRemoveSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        Entities<'a>,
        ReadExpect<'a, Entity>,
        ReadStorage<'a, Name>,
        WriteStorage<'a, WantsToRemoveItem>,
        WriteStorage<'a, Equipped>,
        WriteStorage<'a, InBackpack>,
        ReadStorage<'a, MagicItem>,
        ReadStorage<'a, ObfuscatedName>,
        ReadStorage<'a, Beatitude>,
        ReadExpect<'a, MasterDungeonMap>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            entities,
            player_entity,
            names,
            mut wants_remove,
            mut equipped,
            mut backpack,
            magic_items,
            obfuscated_names,
            beatitudes,
            dm,
        ) = data;

        for (entity, to_remove) in (&entities, &wants_remove).join() {
            let mut can_remove = true;
            if let Some(beatitude) = beatitudes.get(to_remove.item) {
                // If cursed, can't remove!
                if beatitude.buc == BUC::Cursed {
                    can_remove = false;
                    gamelog::Logger
                        ::new()
                        .append(messages::YOU_REMOVE_ITEM_CURSED)
                        .colour(item_colour(to_remove.item, &beatitudes))
                        .append_n(
                            obfuscate_name(
                                to_remove.item,
                                &names,
                                &magic_items,
                                &obfuscated_names,
                                &beatitudes,
                                &dm,
                                None
                            ).0
                        )
                        .colour(WHITE)
                        .append("!")
                        .log();
                }
            }
            if !can_remove {
                continue;
            }
            // If not cursed, remove it
            equipped.remove(to_remove.item);
            if let Some(_) = names.get(to_remove.item) {
                if entity == *player_entity {
                    gamelog::Logger
                        ::new()
                        .append(messages::YOU_REMOVE_ITEM)
                        .colour(item_colour(to_remove.item, &beatitudes))
                        .append_n(
                            obfuscate_name(
                                to_remove.item,
                                &names,
                                &magic_items,
                                &obfuscated_names,
                                &beatitudes,
                                &dm,
                                None
                            ).0
                        )
                        .colour(WHITE)
                        .period()
                        .log();
                }
            }
            backpack
                .insert(to_remove.item, InBackpack { owner: entity })
                .expect("Unable to insert backpack");
        }

        wants_remove.clear();
    }
}
