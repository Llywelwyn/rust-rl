use super::{
    gamelog, Confusion, EntityMoved, EntryTrigger, Hidden, InflictsDamage, Map, Name, ParticleBuilder, Position,
    SingleActivation, SufferDamage,
};
use specs::prelude::*;

pub struct TriggerSystem {}

impl<'a> System<'a> for TriggerSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        ReadExpect<'a, Map>,
        WriteStorage<'a, EntityMoved>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, EntryTrigger>,
        ReadStorage<'a, InflictsDamage>,
        WriteStorage<'a, Confusion>,
        WriteStorage<'a, SufferDamage>,
        WriteStorage<'a, Hidden>,
        ReadStorage<'a, SingleActivation>,
        ReadStorage<'a, Name>,
        WriteExpect<'a, ParticleBuilder>,
        Entities<'a>,
        WriteExpect<'a, rltk::RandomNumberGenerator>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            map,
            mut entity_moved,
            position,
            entry_trigger,
            inflicts_damage,
            mut confusion,
            mut inflict_damage,
            mut hidden,
            single_activation,
            names,
            mut particle_builder,
            entities,
            mut rng,
        ) = data;

        // Iterate entities that moved, and their final position
        let mut remove_entities: Vec<Entity> = Vec::new();
        for (entity, mut _entity_moved, pos) in (&entities, &mut entity_moved, &position).join() {
            let idx = map.xy_idx(pos.x, pos.y);
            for entity_id in map.tile_content[idx].iter() {
                if entity != *entity_id {
                    let maybe_trigger = entry_trigger.get(*entity_id);
                    match maybe_trigger {
                        None => {}
                        Some(_trigger) => {
                            // Something on this pos had a trigger
                            let name = names.get(*entity_id);
                            hidden.remove(*entity_id);
                            if let Some(name) = name {
                                particle_builder.trap_triggered(pos.x, pos.y);
                                gamelog::Logger::new().item_name(&name.name).append("triggers!").log();
                            }

                            let damage = inflicts_damage.get(*entity_id);
                            if let Some(damage) = damage {
                                particle_builder.damage_taken(pos.x, pos.y);
                                SufferDamage::new_damage(
                                    &mut inflict_damage,
                                    entity,
                                    rng.roll_dice(1, damage.amount),
                                    false,
                                );
                            }

                            let confuses = confusion.get(*entity_id);
                            if let Some(confuses) = confuses {
                                confusion
                                    .insert(entity, Confusion { turns: confuses.turns })
                                    .expect("Unable to insert confusion");
                            }

                            let sa = single_activation.get(*entity_id);
                            if let Some(_sa) = sa {
                                remove_entities.push(*entity_id);
                            }
                        }
                    }
                }
            }
        }

        for trap in remove_entities.iter() {
            entities.delete(*trap).expect("Unable to delete trap");
        }

        entity_moved.clear();
    }
}
