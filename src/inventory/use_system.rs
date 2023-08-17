use crate::{
    effects::{add_effect, aoe_tiles, EffectType, Targets},
    EquipmentChanged, IdentifiedItem, Map, Name, WantsToUseItem, AOE,
};
use specs::prelude::*;

pub struct ItemUseSystem {}

impl<'a> System<'a> for ItemUseSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        ReadExpect<'a, Entity>,
        WriteExpect<'a, Map>,
        Entities<'a>,
        WriteStorage<'a, WantsToUseItem>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, AOE>,
        WriteStorage<'a, EquipmentChanged>,
        WriteStorage<'a, IdentifiedItem>,
    );

    #[allow(clippy::cognitive_complexity)]
    fn run(&mut self, data: Self::SystemData) {
        let (player_entity, map, entities, mut wants_use, names, aoe, mut dirty, mut identified_item) = data;

        for (entity, useitem) in (&entities, &wants_use).join() {
            dirty.insert(entity, EquipmentChanged {}).expect("Unable to insert");
            // Identify
            if entity == *player_entity {
                identified_item
                    .insert(entity, IdentifiedItem { name: names.get(useitem.item).unwrap().name.clone() })
                    .expect("Unable to insert");
            }
            // Call the effects system
            add_effect(
                Some(entity),
                EffectType::ItemUse { item: useitem.item },
                match useitem.target {
                    None => Targets::Entity { target: *player_entity },
                    Some(target) => {
                        if let Some(aoe) = aoe.get(useitem.item) {
                            Targets::TileList { targets: aoe_tiles(&*map, target, aoe.radius) }
                        } else {
                            Targets::Tile { target: map.xy_idx(target.x, target.y) }
                        }
                    }
                },
            );
        }
        wants_use.clear();
    }
}
