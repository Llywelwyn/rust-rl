use super::{ Hidden, Map, Mind, Position, Prop, Renderable, Pools };
use bracket_lib::prelude::*;
use specs::prelude::*;
use std::ops::Mul;
use super::data::visuals::{ VIEWPORT_W, VIEWPORT_H };
use super::data::prelude::*;

const SHOW_BOUNDARIES: bool = false;

pub fn get_screen_bounds(ecs: &World) -> (i32, i32, i32, i32, i32, i32) {
    let player_pos = ecs.fetch::<Point>();
    let map = ecs.fetch::<Map>();
    let (x_chars, y_chars, mut x_offset, mut y_offset) = (VIEWPORT_W, VIEWPORT_H, 1, 10);

    let centre_x = (x_chars / 2) as i32;
    let centre_y = (y_chars / 2) as i32;

    let min_x = if map.width < (x_chars as i32) {
        x_offset += ((x_chars as i32) - map.width) / 2;
        0
    } else {
        (player_pos.x - centre_x).clamp(0, map.width - (x_chars as i32))
    };
    let min_y = if map.height < (y_chars as i32) {
        y_offset += ((y_chars as i32) - map.height) / 2;
        0
    } else {
        (player_pos.y - centre_y).clamp(0, map.height - (y_chars as i32))
    };
    let max_x = min_x + (x_chars as i32);
    let max_y = min_y + (y_chars as i32);

    (min_x, max_x, min_y, max_y, x_offset, y_offset)
}

pub fn render_camera(ecs: &World, ctx: &mut BTerm) {
    let map = ecs.fetch::<Map>();
    let (min_x, max_x, min_y, max_y, x_offset, y_offset) = get_screen_bounds(ecs);

    // Render map
    let mut y = 0;
    for t_y in min_y..max_y {
        let mut x = 0;
        for t_x in min_x..max_x {
            if t_x >= 0 && t_x < map.width && t_y >= 0 && t_y < map.height {
                let idx = map.xy_idx(t_x, t_y);
                if map.revealed_tiles[idx] {
                    if 1 == 2 {
                        let (glyph, fg, bg) = crate::map::themes::get_tile_renderables_for_id(
                            idx,
                            &*map,
                            Some(*ecs.fetch::<Point>()),
                            None
                        );
                        ctx.set(x + x_offset, y + y_offset, fg, bg, glyph);
                    } else {
                        ctx.set_active_console(0);
                        let (id, tint) = crate::map::themes::get_sprite_for_id(
                            idx,
                            &*map,
                            Some(*ecs.fetch::<Point>())
                        );
                        ctx.add_sprite(
                            Rect::with_size(x * 16 + x_offset * 16, y * 16 + y_offset * 16, 16, 16),
                            0,
                            tint,
                            id
                        );
                        ctx.set_active_console(TILE_LAYER);
                    }
                }
            } else if SHOW_BOUNDARIES {
                ctx.set(
                    x + x_offset,
                    y + y_offset,
                    RGB::named(DARKSLATEGRAY),
                    RGB::named(BLACK),
                    to_cp437('#')
                );
            }
            x += 1;
        }
        y += 1;
    }

    // Render entities
    {
        ctx.set_active_console(ENTITY_LAYER);

        let positions = ecs.read_storage::<Position>();
        let renderables = ecs.read_storage::<Renderable>();
        let pools = ecs.read_storage::<Pools>();
        let minds = ecs.read_storage::<Mind>();
        let hidden = ecs.read_storage::<Hidden>();
        let props = ecs.write_storage::<Prop>();
        let map = ecs.fetch::<Map>();
        let entities = ecs.entities();

        let mut data = (&positions, &renderables, &entities, !&hidden).join().collect::<Vec<_>>();
        data.sort_by(|&a, &b| b.1.render_order.cmp(&a.1.render_order));
        for (pos, render, ent, _hidden) in data.iter() {
            let idx = map.xy_idx(pos.x, pos.y);
            let entity_offset_x = pos.x - min_x;
            let entity_offset_y = pos.y - min_y;
            if pos.x < max_x && pos.y < max_y && pos.x >= min_x && pos.y >= min_y {
                let mut draw = false;
                let mut fg = render.fg;
                let bg = BLACK;
                // Draw entities on visible tiles
                if map.visible_tiles[idx] {
                    draw = true;
                } else {
                    fg = fg.mul(crate::data::visuals::NON_VISIBLE_MULTIPLIER);
                    // We don't darken BG, because get_tile_renderables_for_id handles this.
                }

                // Draw entities with minds within telepath range
                if !draw {
                    if map.telepath_tiles[idx] {
                        let has_mind = minds.get(*ent);
                        if let Some(_) = has_mind {
                            draw = true;
                        }
                    }
                }
                // Draw all props
                let is_prop = props.get(*ent);
                if let Some(_) = is_prop {
                    if map.revealed_tiles[idx] {
                        draw = true;
                    }
                }
                if draw {
                    if let Some(sprite) = render.sprite {
                        ctx.set_active_console(0);
                        ctx.add_sprite(
                            Rect::with_size(
                                entity_offset_x * 16 + x_offset * 16,
                                entity_offset_y * 16 + y_offset * 16,
                                16,
                                16
                            ),
                            render.render_order,
                            RGBA::named(WHITE),
                            sprite
                        );
                        ctx.set_active_console(ENTITY_LAYER);
                    } else {
                        ctx.set(
                            entity_offset_x + x_offset,
                            entity_offset_y + y_offset,
                            fg,
                            bg,
                            render.glyph
                        );
                    }
                    if let Some(pool) = pools.get(*ent) {
                        if pool.hit_points.current < pool.hit_points.max {
                            ctx.set_active_console(HP_BAR_LAYER);
                            crate::gui::draw_lerping_bar(
                                ctx,
                                (entity_offset_x + x_offset) * 16 + 2,
                                (entity_offset_y + y_offset) * 16 - 1,
                                14,
                                pool.hit_points.current,
                                pool.hit_points.max,
                                RGB::named(GREEN),
                                RGB::named(RED),
                                false,
                                false
                            );
                            ctx.set_active_console(ENTITY_LAYER);
                        }
                    }
                }
            }
        }
        ctx.set_active_console(TILE_LAYER);
    }
}

pub fn render_debug_map(map: &Map, ctx: &mut BTerm) {
    let player_pos = Point::new(map.width / 2, map.height / 2);
    let (x_chars, y_chars) = ctx.get_char_size();

    let center_x = (x_chars / 2) as i32;
    let center_y = (y_chars / 2) as i32;

    let min_x = player_pos.x - center_x;
    let max_x = min_x + (x_chars as i32);
    let min_y = player_pos.y - center_y;
    let max_y = min_y + (y_chars as i32);

    let map_width = map.width;
    let map_height = map.height;

    let mut y = 0;
    for ty in min_y..max_y {
        let mut x = 0;
        for tx in min_x..max_x {
            if tx >= 0 && tx < map_width && ty >= 0 && ty < map_height {
                let idx = map.xy_idx(tx, ty);
                if map.revealed_tiles[idx] {
                    let (glyph, fg, bg) = crate::map::themes::get_tile_renderables_for_id(
                        idx,
                        &*map,
                        None,
                        None
                    );
                    ctx.set(x, y, fg, bg, glyph);
                }
            } else if SHOW_BOUNDARIES {
                ctx.set(x, y, RGB::named(GRAY), RGB::named(BLACK), to_cp437('·'));
            }
            x += 1;
        }
        y += 1;
    }
}
