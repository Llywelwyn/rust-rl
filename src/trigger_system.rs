use super::{
    effects::{add_effect, EffectType, Targets},
    gamelog, Confusion, EntityMoved, EntryTrigger, Hidden, InflictsDamage, Map, Name, ParticleBuilder, Position,
    SingleActivation,
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
            crate::spatial::for_each_tile_content(idx, |entity_id| {
                if entity != entity_id {
                    let maybe_trigger = entry_trigger.get(entity_id);
                    match maybe_trigger {
                        None => {}
                        Some(_trigger) => {
                            // Something on this pos had a trigger
                            let name = names.get(entity_id);
                            hidden.remove(entity_id);
                            if let Some(name) = name {
                                particle_builder.trap_triggered(pos.x, pos.y);
                                gamelog::Logger::new().item_name(&name.name).append("triggers!").log();
                            }

                            let damage = inflicts_damage.get(entity_id);
                            if let Some(damage) = damage {
                                let damage_roll = rng.roll_dice(damage.n_dice, damage.sides) + damage.modifier;
                                particle_builder.damage_taken(pos.x, pos.y);
                                add_effect(
                                    None,
                                    EffectType::Damage { amount: damage_roll },
                                    Targets::Entity { target: entity },
                                );
                            }

                            let confuses = confusion.get(entity_id);
                            if let Some(confuses) = confuses {
                                confusion
                                    .insert(entity, Confusion { turns: confuses.turns })
                                    .expect("Unable to insert confusion");
                            }

                            let sa = single_activation.get(entity_id);
                            if let Some(_sa) = sa {
                                remove_entities.push(entity_id);
                            }
                        }
                    }
                }
            });
        }

        for trap in remove_entities.iter() {
            entities.delete(*trap).expect("Unable to delete trap");
        }

        entity_moved.clear();
    }
}
