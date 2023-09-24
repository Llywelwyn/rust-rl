use super::{ State, RunState, tooltip::draw_tooltips, camera::get_offset, VIEWPORT_H, VIEWPORT_W };
use bracket_lib::prelude::*;

#[derive(PartialEq, Copy, Clone)]
pub enum FarlookResult {
    NoResponse {
        x: i32,
        y: i32,
    },
    Cancel,
}

pub fn show_farlook(gs: &mut State, ctx: &mut BTerm) -> FarlookResult {
    let runstate = gs.ecs.fetch::<RunState>();
    let offsets = get_offset();

    ctx.print_color(
        1 + offsets.x,
        1 + offsets.y,
        RGB::named(WHITE),
        RGB::named(BLACK),
        "Look at what? [move keys][Esc.]"
    );

    if let RunState::Farlook { x, y } = *runstate {
        let x = x.clamp(offsets.x, offsets.x - 1 + VIEWPORT_W);
        let y = y.clamp(offsets.y, offsets.y - 1 + VIEWPORT_H);

        ctx.set(x, y, RGB::named(WHITE), RGB::named(BLACK), to_cp437('X'));
        draw_tooltips(&gs.ecs, ctx, Some((x, y)));

        return match ctx.key {
            None => FarlookResult::NoResponse { x, y },
            Some(key) =>
                match key {
                    VirtualKeyCode::Escape | VirtualKeyCode::X => FarlookResult::Cancel,
                    VirtualKeyCode::Numpad9 => FarlookResult::NoResponse { x: x + 1, y: y - 1 },
                    VirtualKeyCode::Numpad8 => FarlookResult::NoResponse { x, y: y - 1 },
                    VirtualKeyCode::Numpad7 => FarlookResult::NoResponse { x: x - 1, y: y - 1 },
                    VirtualKeyCode::Numpad6 => FarlookResult::NoResponse { x: x + 1, y },
                    VirtualKeyCode::Numpad4 => FarlookResult::NoResponse { x: x - 1, y },
                    VirtualKeyCode::Numpad3 => FarlookResult::NoResponse { x: x + 1, y: y + 1 },
                    VirtualKeyCode::Numpad2 => FarlookResult::NoResponse { x, y: y + 1 },
                    VirtualKeyCode::Numpad1 => FarlookResult::NoResponse { x: x - 1, y: y + 1 },
                    _ => FarlookResult::NoResponse { x, y },
                }
        };
    } else {
        let ppos = gs.ecs.fetch::<Point>();
        // TODO: PPOS + offsets (should get these from screen_bounds())
        return FarlookResult::NoResponse { x: ppos.x + offsets.x, y: ppos.x + offsets.y };
    }
}
