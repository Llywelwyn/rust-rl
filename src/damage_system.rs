use super::{gamelog, CombatStats, Entities, Item, Map, Name, Player, Position, SufferDamage};
use specs::prelude::*;

pub struct DamageSystem {}

impl<'a> System<'a> for DamageSystem {
    type SystemData = (
        WriteStorage<'a, CombatStats>,
        WriteStorage<'a, SufferDamage>,
        WriteExpect<'a, Map>,
        ReadStorage<'a, Position>,
        Entities<'a>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut stats, mut damage, mut map, positions, entities) = data;

        for (entity, mut stats, damage) in (&entities, &mut stats, &damage).join() {
            stats.hp -= damage.amount.iter().sum::<i32>();
            let pos = positions.get(entity);
            if let Some(pos) = pos {
                let idx = map.xy_idx(pos.x, pos.y);
                map.bloodstains.insert(idx);
            }
        }

        damage.clear();
    }
}

pub fn delete_the_dead(ecs: &mut World) {
    let mut dead: Vec<Entity> = Vec::new();
    // Using scope to make borrow checker happy
    {
        let combat_stats = ecs.read_storage::<CombatStats>();
        let players = ecs.read_storage::<Player>();
        let names = ecs.read_storage::<Name>();
        let items = ecs.read_storage::<Item>();
        let entities = ecs.entities();
        for (entity, stats) in (&entities, &combat_stats).join() {
            if stats.hp < 1 {
                let player = players.get(entity);
                match player {
                    None => {
                        let victim_name = names.get(entity);
                        if let Some(victim_name) = victim_name {
                            let item = items.get(entity);
                            if let Some(_item) = item {
                                gamelog::Logger::new()
                                    .append("The")
                                    .npc_name(&victim_name.name)
                                    .colour(rltk::WHITE)
                                    .append("was destroyed.")
                                    .log();
                            } else {
                                gamelog::Logger::new()
                                    .append("The")
                                    .npc_name(&victim_name.name)
                                    .colour(rltk::WHITE)
                                    .append("died.")
                                    .log();
                            }
                        }
                        dead.push(entity)
                    }
                    Some(_) => {
                        // This is where the GameOver state will go eventully. But currently
                        // it's easier to just keep the game going for the sake of testing.
                    }
                }
            }
        }
    }

    for victim in dead {
        ecs.delete_entity(victim).expect("Unable to delete.");
    }
}
