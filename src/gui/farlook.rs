use super::{ State, RunState, tooltip::draw_tooltips, camera::get_offset, VIEWPORT_H, VIEWPORT_W };
use bracket_lib::prelude::*;
use notan::prelude::*;
use notan::draw::{ Draw, DrawImages };
use std::collections::HashMap;
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
    let offsets = get_offset();
    if let RunState::Farlook { x, y } = *runstate {
        let x = x.clamp(offsets.x, offsets.x - 1 + VIEWPORT_W);
        let y = y.clamp(offsets.y, offsets.y - 1 + VIEWPORT_H);
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
        let ppos = gs.ecs.fetch::<Point>();
        return FarlookResult::NoResponse { x: ppos.x + offsets.x, y: ppos.x + offsets.y };
    }
}

pub fn draw_farlook(x: i32, y: i32, draw: &mut Draw, atlas: &HashMap<String, Texture>) {
    draw.image(atlas.get("ui_select_c1").unwrap()).position(
        (x as f32) * TILESIZE.x,
        (y as f32) * TILESIZE.x
    );
}
