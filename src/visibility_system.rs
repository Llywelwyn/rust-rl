use super::{Map, Player, Position, Telepath, Viewshed};
use rltk::{FieldOfViewAlg::SymmetricShadowcasting, Point};
use specs::prelude::*;

pub struct VisibilitySystem {}

impl<'a> System<'a> for VisibilitySystem {
    type SystemData = (
        WriteExpect<'a, Map>,
        Entities<'a>,
        WriteStorage<'a, Viewshed>,
        WriteStorage<'a, Telepath>,
        WriteStorage<'a, Position>,
        ReadStorage<'a, Player>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut map, entities, mut viewshed, mut telepath, pos, player) = data;

        for (ent, viewshed, pos) in (&entities, &mut viewshed, &pos).join() {
            if viewshed.dirty {
                viewshed.dirty = false;
                // FIXME: SymmetricShadowcasting seems to be responsible for an infrequent crash --
                // but it could be unrelated to the FieldOfViewAlg being used. Needs some more testing!
                // Appeared first after switching, but can't seem to reproduce it.
                viewshed.visible_tiles =
                    SymmetricShadowcasting.field_of_view(Point::new(pos.x, pos.y), viewshed.range, &*map);
                viewshed.visible_tiles.retain(|p| p.x >= 0 && p.x < map.width && p.y >= 0 && p.y < map.height);

                // If this is the player, reveal what they can see
                let _p: Option<&Player> = player.get(ent);
                if let Some(_p) = _p {
                    for t in map.visible_tiles.iter_mut() {
                        *t = false;
                    }
                    for vis in viewshed.visible_tiles.iter() {
                        let idx = map.xy_idx(vis.x, vis.y);
                        map.revealed_tiles[idx] = true;
                        map.visible_tiles[idx] = true;
                    }
                }
            }
        }

        for (ent, telepath, pos) in (&entities, &mut telepath, &pos).join() {
            if telepath.dirty {
                telepath.dirty = false;

                telepath.telepath_tiles = fast_fov(pos.x, pos.y, telepath.range);
                telepath.telepath_tiles.retain(|p| p.x >= 0 && p.x < map.width && p.y >= 0 && p.y < map.height);

                // If this is the player, reveal what they can see
                let _p: Option<&Player> = player.get(ent);
                if let Some(_p) = _p {
                    for t in map.telepath_tiles.iter_mut() {
                        *t = false;
                    }
                    for vis in telepath.telepath_tiles.iter() {
                        let idx = map.xy_idx(vis.x, vis.y);
                        map.telepath_tiles[idx] = true;
                    }
                }
            }
        }
    }
}

pub fn fast_fov(p_x: i32, p_y: i32, r: i32) -> Vec<Point> {
    let mut visible_tiles: Vec<Point> = Vec::new();

    let mut i = 0;
    while i <= 360 {
        let x: f32 = f32::cos(i as f32 * 0.01745 as f32);
        let y: f32 = f32::sin(i as f32 * 0.01745 as f32);

        let mut ox: f32 = p_x as f32 + 0.5 as f32;
        let mut oy: f32 = p_y as f32 + 0.5 as f32;
        for _i in 0..r {
            visible_tiles.push(Point::new(ox as i32, oy as i32));
            ox += x;
            oy += y;
        }
        i += 4;
    }

    visible_tiles
}
