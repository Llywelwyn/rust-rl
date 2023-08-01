use super::{Bystander, EntityMoved, Map, Position, TakingTurn, Viewshed};
use specs::prelude::*;

pub struct BystanderAI {}

impl<'a> System<'a> for BystanderAI {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        WriteExpect<'a, Map>,
        Entities<'a>,
        WriteStorage<'a, Viewshed>,
        ReadStorage<'a, Bystander>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, EntityMoved>,
        WriteExpect<'a, rltk::RandomNumberGenerator>,
        ReadStorage<'a, TakingTurn>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut map, entities, mut viewshed, bystander, mut position, mut entity_moved, mut rng, turns) = data;

        for (entity, mut viewshed, _bystander, mut pos, _turn) in
            (&entities, &mut viewshed, &bystander, &mut position, &turns).join()
        {
            if try_move_randomly(&mut pos, &mut rng, &mut map, &mut viewshed) {
                entity_moved.insert(entity, EntityMoved {}).expect("Unable to insert marker");
            }
        }
    }
}

pub fn try_move_randomly(
    pos: &mut Position,
    rng: &mut rltk::RandomNumberGenerator,
    map: &mut Map,
    viewshed: &mut Viewshed,
) -> bool {
    // Try to move randomly
    let mut x = pos.x;
    let mut y = pos.y;
    let move_roll = rng.roll_dice(1, 8);
    match move_roll {
        1 => x -= 1,
        2 => x += 1,
        3 => y -= 1,
        4 => y += 1,
        _ => {}
    }

    if x > 0 && x < map.width - 1 && y > 0 && y < map.height - 1 {
        let dest_idx = map.xy_idx(x, y);
        if !map.blocked[dest_idx] {
            let idx = map.xy_idx(pos.x, pos.y);
            map.blocked[idx] = false;
            pos.x = x;
            pos.y = y;
            map.blocked[dest_idx] = true;
            viewshed.dirty = true;
            return true;
        }
    }
    return false;
}
