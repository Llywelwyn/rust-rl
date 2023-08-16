use super::{gamelog, Equipped, InBackpack, Name, WantsToRemoveItem};
use specs::prelude::*;

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
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, player_entity, names, mut wants_remove, mut equipped, mut backpack) = data;

        for (entity, to_remove) in (&entities, &wants_remove).join() {
            equipped.remove(to_remove.item);
            if let Some(name) = names.get(to_remove.item) {
                if entity == *player_entity {
                    gamelog::Logger::new().append("You unequip the").item_name_n(&name.name).period().log();
                }
            }
            backpack.insert(to_remove.item, InBackpack { owner: entity }).expect("Unable to insert backpack");
        }

        wants_remove.clear();
    }
}
