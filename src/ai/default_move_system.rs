use crate::{EntityMoved, Map, MoveMode, Movement, Position, TakingTurn, Telepath, Viewshed};
use specs::prelude::*;

// Rolling a 1d8+x to decide where to move, where x are the number
// of dice rolls in which they will remian stationary. i.e. If this
// const is set to 8, there is a 50% chance of not wandering.
const CHANCE_OF_REMAINING_STATIONARY: i32 = 8;
pub struct DefaultAI {}

impl<'a> System<'a> for DefaultAI {
    type SystemData = (
        WriteStorage<'a, TakingTurn>,
        ReadStorage<'a, MoveMode>,
        WriteStorage<'a, Position>,
        WriteExpect<'a, Map>,
        WriteStorage<'a, Viewshed>,
        WriteStorage<'a, Telepath>,
        WriteStorage<'a, EntityMoved>,
        WriteExpect<'a, rltk::RandomNumberGenerator>,
        Entities<'a>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            mut turns,
            move_mode,
            mut positions,
            mut map,
            mut viewsheds,
            mut telepaths,
            mut entity_moved,
            mut rng,
            entities,
        ) = data;
        let mut turn_done: Vec<Entity> = Vec::new();
        for (entity, _turn, mut pos, move_mode, mut viewshed) in
            (&entities, &turns, &mut positions, &move_mode, &mut viewsheds).join()
        {
            turn_done.push(entity);
            match move_mode.mode {
                Movement::Static => {}
                Movement::Random => {
                    let mut x = pos.x;
                    let mut y = pos.y;
                    let move_roll = rng.roll_dice(1, 8 + CHANCE_OF_REMAINING_STATIONARY);
                    match move_roll {
                        1 => x -= 1,
                        2 => x += 1,
                        3 => y -= 1,
                        4 => y += 1,
                        5 => {
                            x -= 1;
                            y -= 1
                        }
                        6 => {
                            x += 1;
                            y -= 1
                        }
                        7 => {
                            x -= 1;
                            y += 1
                        }
                        8 => {
                            x += 1;
                            y += 1
                        }
                        _ => {}
                    }

                    if x > 0 && x < map.width - 1 && y > 0 && y < map.height - 1 {
                        let dest_idx = map.xy_idx(x, y);
                        if !map.blocked[dest_idx] {
                            let idx = map.xy_idx(pos.x, pos.y);
                            map.blocked[idx] = false;
                            pos.x = x;
                            pos.y = y;
                            entity_moved.insert(entity, EntityMoved {}).expect("Unable to insert EntityMoved");
                            map.blocked[dest_idx] = true;
                            viewshed.dirty = true;
                            if let Some(is_telepath) = telepaths.get_mut(entity) {
                                is_telepath.dirty = true;
                            }
                        }
                    }
                }
            }
        }
        for done in turn_done.iter() {
            turns.remove(*done);
        }
    }
}
