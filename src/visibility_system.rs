use super::{
    gamelog,
    Blind,
    BlocksVisibility,
    Hidden,
    Map,
    Name,
    Player,
    Position,
    Telepath,
    Viewshed,
    Renderable,
    Prop,
    Item,
    gui::renderable_colour,
    tile_blocks_telepathy,
};
use bracket_lib::prelude::*;
use bracket_lib::pathfinding::FieldOfViewAlg::SymmetricShadowcasting;
use specs::prelude::*;

pub struct VisibilitySystem {}

const BLIND_TELEPATHY_RANGE_MULTIPLIER: i32 = 3;

impl<'a> System<'a> for VisibilitySystem {
    type SystemData = (
        WriteExpect<'a, Map>,
        WriteExpect<'a, RandomNumberGenerator>,
        Entities<'a>,
        WriteStorage<'a, Viewshed>,
        WriteStorage<'a, Telepath>,
        WriteStorage<'a, Position>,
        ReadStorage<'a, Player>,
        WriteStorage<'a, Hidden>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, Blind>,
        ReadStorage<'a, BlocksVisibility>,
        ReadStorage<'a, Renderable>,
        ReadStorage<'a, Prop>,
        ReadStorage<'a, Item>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            mut map,
            mut rng,
            entities,
            mut viewshed,
            mut telepath,
            pos,
            player,
            mut hidden,
            names,
            blind_entities,
            blocks_visibility,
            renderables,
            prop,
            item,
        ) = data;

        map.view_blocked.clear();
        for (block_pos, _block) in (&pos, &blocks_visibility).join() {
            let idx = map.xy_idx(block_pos.x, block_pos.y);
            map.view_blocked.insert(idx);
        }

        let mut player_was_dirty = false;
        for (ent, viewshed, pos) in (&entities, &mut viewshed, &pos).join() {
            if viewshed.dirty {
                viewshed.dirty = false;
                let range = if let Some(_is_blind) = blind_entities.get(ent) {
                    1
                } else {
                    if map.overmap { viewshed.range / 2 } else { viewshed.range }
                };
                let origin = Point::new(pos.x, pos.y);
                viewshed.visible_tiles = SymmetricShadowcasting.field_of_view(origin, range, &*map);
                viewshed.visible_tiles.retain(|p| {
                    p.x >= 0 &&
                        p.x < map.width &&
                        p.y >= 0 &&
                        p.y < map.height &&
                        (map.lit_tiles[map.xy_idx(p.x, p.y)] == true ||
                            DistanceAlg::Pythagoras.distance2d(Point::new(p.x, p.y), origin) < 1.5)
                });

                // If this is the player, reveal what they can see
                let _p: Option<&Player> = player.get(ent);
                if let Some(_p) = _p {
                    player_was_dirty = true;
                    for t in map.visible_tiles.iter_mut() {
                        *t = false;
                    }
                    for vis in viewshed.visible_tiles.iter() {
                        let idx = map.xy_idx(vis.x, vis.y);
                        map.revealed_tiles[idx] = true;
                        map.visible_tiles[idx] = true;

                        // Reveal hidden things
                        crate::spatial::for_each_tile_content(idx, |e| {
                            let is_hidden = hidden.get(e);
                            if let Some(_is_hidden) = is_hidden {
                                if rng.roll_dice(1, 12) == 1 {
                                    let name = names.get(e);
                                    if let Some(name) = name {
                                        gamelog::Logger
                                            ::new()
                                            .append("You spot a")
                                            .colour(renderable_colour(&renderables, e))
                                            .append_n(&name.name)
                                            .colour(WHITE)
                                            .period()
                                            .log();
                                    }
                                    hidden.remove(e);
                                }
                            }
                        });
                    }
                }
            }
        }

        for (ent, telepath, pos) in (&entities, &mut telepath, &pos).join() {
            if telepath.dirty {
                telepath.dirty = false;
                let mut range = telepath.range;
                if let Some(_is_blind) = blind_entities.get(ent) {
                    range *= BLIND_TELEPATHY_RANGE_MULTIPLIER;
                }
                telepath.telepath_tiles = fast_fov(pos.x, pos.y, range, &map);
                telepath.telepath_tiles.retain(
                    |p| p.x >= 0 && p.x < map.width && p.y >= 0 && p.y < map.height
                );

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

        if player_was_dirty {
            // Refresh the memory of our visible tiles, by removing whatever
            // was stored for every index we can currently see, and placing
            // back in updated data.
            let mut to_remove: Vec<usize> = Vec::new();
            for (i, &t) in map.visible_tiles.iter().enumerate() {
                if t {
                    to_remove.push(i);
                }
            }
            for idx in to_remove.iter() {
                map.memory.remove(idx);
            }
            for (e, r, p, _h) in (&entities, &renderables, &pos, !&hidden).join() {
                if prop.get(e).is_some() || item.get(e).is_some() {
                    let idx = map.xy_idx(p.x, p.y);
                    if map.visible_tiles[idx] {
                        if let Some(spriteinfo) = &r.sprite {
                            map.memory.entry(idx).or_insert(Vec::new()).push(crate::MapMemory {
                                sprite: spriteinfo.id.clone(),
                                fg: r.fg,
                                recolour: spriteinfo.recolour,
                                offset: spriteinfo.offset,
                                render_order: r.render_order,
                            });
                        }
                    }
                }
            }
        }
    }
}

pub fn fast_fov(p_x: i32, p_y: i32, r: i32, map: &WriteExpect<Map>) -> Vec<Point> {
    let mut visible_tiles: Vec<Point> = Vec::new();

    let mut i = 0;
    while i <= 360 {
        let x: f32 = f32::cos((i as f32) * (0.01745 as f32));
        let y: f32 = f32::sin((i as f32) * (0.01745 as f32));

        let mut ox: f32 = (p_x as f32) + (0.5 as f32);
        let mut oy: f32 = (p_y as f32) + (0.5 as f32);
        for _i in 0..r {
            let (ox_i32, oy_i32) = (ox as i32, oy as i32);
            visible_tiles.push(Point::new(ox_i32, oy_i32));
            if
                ox_i32 >= 0 &&
                ox_i32 < map.width &&
                oy_i32 >= 0 &&
                oy_i32 < map.height &&
                tile_blocks_telepathy(map.tiles[map.xy_idx(ox_i32, oy_i32)])
            {
                break;
            }
            ox += x;
            oy += y;
        }
        i += 4;
    }

    visible_tiles
}
