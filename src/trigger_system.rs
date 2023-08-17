use super::{
    effects::{add_effect, aoe_tiles, EffectType, Targets},
    gamelog,
    gui::renderable_colour,
    EntityMoved, EntryTrigger, Map, Name, Point, Position, Renderable, AOE,
};
use rltk::prelude::*;
use specs::prelude::*;

pub struct TriggerSystem {}

impl<'a> System<'a> for TriggerSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        ReadExpect<'a, Map>,
        WriteStorage<'a, EntityMoved>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, EntryTrigger>,
        ReadStorage<'a, Name>,
        Entities<'a>,
        ReadStorage<'a, AOE>,
        ReadStorage<'a, Renderable>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (map, mut entity_moved, position, entry_trigger, names, entities, aoes, renderables) = data;
        for (entity, mut _entity_moved, pos) in (&entities, &mut entity_moved, &position).join() {
            let idx = map.xy_idx(pos.x, pos.y);
            crate::spatial::for_each_tile_content(idx, |entity_id| {
                if entity != entity_id {
                    let maybe_trigger = entry_trigger.get(entity_id);
                    match maybe_trigger {
                        None => {}
                        Some(_trigger) => {
                            if map.visible_tiles[idx] == true {
                                if let Some(name) = names.get(entity_id) {
                                    gamelog::Logger::new()
                                        .append("The")
                                        .colour(renderable_colour(&renderables, entity_id))
                                        .append(&name.name)
                                        .colour(WHITE)
                                        .append("triggers!")
                                        .log();
                                }
                            }
                            add_effect(
                                Some(entity_id),
                                EffectType::TriggerFire { trigger: entity_id },
                                if let Some(aoe) = aoes.get(entity_id) {
                                    Targets::TileList {
                                        targets: aoe_tiles(&*map, Point::new(pos.x, pos.y), aoe.radius),
                                    }
                                } else {
                                    Targets::Tile { target: idx }
                                },
                            );
                        }
                    }
                }
            });
        }
        entity_moved.clear();
    }
}
