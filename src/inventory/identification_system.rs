use crate::{ Beatitude, IdentifiedBeatitude, IdentifiedItem, Item, MasterDungeonMap, Name, ObfuscatedName, Player };
use specs::prelude::*;
use crate::data::events::*;
use crate::gamelog;

pub struct ItemIdentificationSystem {}

impl<'a> System<'a> for ItemIdentificationSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        ReadStorage<'a, Player>,
        WriteStorage<'a, IdentifiedItem>,
        WriteStorage<'a, Beatitude>,
        WriteStorage<'a, IdentifiedBeatitude>,
        WriteExpect<'a, MasterDungeonMap>,
        ReadStorage<'a, Item>,
        ReadStorage<'a, Name>,
        WriteStorage<'a, ObfuscatedName>,
        Entities<'a>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            player,
            mut identified,
            mut beatitudes,
            mut identified_beatitudes,
            mut dm,
            items,
            names,
            mut obfuscated_names,
            entities,
        ) = data;
        for (_p, id) in (&player, &identified).join() {
            let tag = crate::raws::get_id_from_name(id.name.clone());
            if !dm.identified_items.contains(&id.name) && crate::raws::is_tag_magic(&tag) {
                if gamelog::get_event_count(EVENT::COUNT_TURN) != 1 {
                    gamelog::record_event(EVENT::IDENTIFIED(id.name.clone()));
                }
                dm.identified_items.insert(id.name.clone());
                for (entity, _item, name) in (&entities, &items, &names).join() {
                    if name.name == id.name {
                        obfuscated_names.remove(entity);
                    }
                }
            }
        }
        for (e, _id) in (&entities, &identified_beatitudes).join() {
            if let Some(beatitude) = beatitudes.get_mut(e) {
                beatitude.known = true;
            }
        }
        // Clean up
        identified.clear();
        identified_beatitudes.clear();
    }
}
