use super::{ State, RunState, World, tooltip::draw_tooltips, camera::get_offset };
use bracket_lib::prelude::*;
use notan::prelude::*;
use notan::draw::{ Draw, DrawImages };
use std::collections::HashMap;
use crate::consts::visuals::{ TILES_IN_VIEWPORT_H, TILES_IN_VIEWPORT_W };
use crate::consts::TILESIZE;

#[derive(PartialEq, Copy, Clone)]
pub enum FarlookResult {
    NoResponse {
        x: i32,
        y: i32,
    },
    Cancel,
}

pub fn show_farlook(gs: &mut State, ctx: &mut App) -> FarlookResult {
    let runstate = gs.ecs.fetch::<RunState>();
    if let RunState::Farlook { x, y } = *runstate {
        let x = x.clamp(0, TILES_IN_VIEWPORT_W - 1);
        let y = y.clamp(0, TILES_IN_VIEWPORT_H - 1);
        let key = &ctx.keyboard;
        // Movement
        for keycode in key.pressed.iter() {
            match *keycode {
                KeyCode::Escape | KeyCode::X => {
                    return FarlookResult::Cancel;
                }
                KeyCode::Numpad1 => {
                    return FarlookResult::NoResponse { x: x - 1, y: y + 1 };
                }
                KeyCode::Numpad2 => {
                    return FarlookResult::NoResponse { x, y: y + 1 };
                }
                KeyCode::Numpad3 => {
                    return FarlookResult::NoResponse { x: x + 1, y: y + 1 };
                }
                KeyCode::Numpad4 => {
                    return FarlookResult::NoResponse { x: x - 1, y };
                }
                KeyCode::Numpad6 => {
                    return FarlookResult::NoResponse { x: x + 1, y };
                }
                KeyCode::Numpad7 => {
                    return FarlookResult::NoResponse { x: x - 1, y: y - 1 };
                }
                KeyCode::Numpad8 => {
                    return FarlookResult::NoResponse { x, y: y - 1 };
                }
                KeyCode::Numpad9 => {
                    return FarlookResult::NoResponse { x: x + 1, y: y - 1 };
                }
                _ => {}
            }
        }
        return FarlookResult::NoResponse { x, y };
    } else {
        return FarlookResult::NoResponse { x: TILES_IN_VIEWPORT_W / 2, y: TILES_IN_VIEWPORT_H / 2 };
    }
}

pub fn draw_farlook(
    ecs: &World,
    x: i32,
    y: i32,
    draw: &mut Draw,
    atlas: &HashMap<String, Texture>
) {
    let placement = super::viewport_to_px(x, y);
    draw.image(atlas.get("select1").unwrap())
        .position(placement.x, placement.y)
        .size(TILESIZE.sprite_x, TILESIZE.sprite_y);
    let _idx = super::viewport_to_map_idx(ecs, x, y);
    // Get tooltip for idx, etc.
}
