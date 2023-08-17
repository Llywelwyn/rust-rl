use crate::{IdentifiedItem, Item, MasterDungeonMap, Name, ObfuscatedName, Player};
use specs::prelude::*;

pub struct ItemIdentificationSystem {}

impl<'a> System<'a> for ItemIdentificationSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        ReadStorage<'a, Player>,
        WriteStorage<'a, IdentifiedItem>,
        WriteExpect<'a, MasterDungeonMap>,
        ReadStorage<'a, Item>,
        ReadStorage<'a, Name>,
        WriteStorage<'a, ObfuscatedName>,
        Entities<'a>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (player, mut identified, mut dm, items, names, mut obfuscated_names, entities) = data;
        for (_p, id) in (&player, &identified).join() {
            rltk::console::log(id.name.clone());
            let tag = crate::raws::get_id_from_name(id.name.clone());
            if !dm.identified_items.contains(&id.name) && crate::raws::is_tag_magic(&tag) {
                dm.identified_items.insert(id.name.clone());

                for (entity, _item, name) in (&entities, &items, &names).join() {
                    if name.name == id.name {
                        obfuscated_names.remove(entity);
                    }
                }
            }
        }
        // Clean up
        identified.clear();
    }
}
