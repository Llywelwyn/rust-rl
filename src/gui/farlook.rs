use super::{ State, RunState, tooltip::draw_tooltips, camera::get_screen_bounds };
use rltk::prelude::*;

#[derive(PartialEq, Copy, Clone)]
pub enum FarlookResult {
    NoResponse {
        x: i32,
        y: i32,
    },
    Cancel,
}

pub fn show_farlook(gs: &mut State, ctx: &mut Rltk) -> FarlookResult {
    let runstate = gs.ecs.fetch::<RunState>();
    let (_min_x, _max_x, _min_y, _max_y, x_offset, y_offset) = get_screen_bounds(&gs.ecs, ctx);

    ctx.print_color(
        1 + x_offset,
        1 + y_offset,
        RGB::named(WHITE),
        RGB::named(BLACK),
        "Look at what? [move keys][Esc.]"
    );

    if let RunState::Farlook { x, y } = *runstate {
        let (screen_x, screen_y) = (69, 41);
        let x = x.clamp(x_offset, x_offset - 1 + (screen_x as i32));
        let y = y.clamp(y_offset, y_offset - 1 + (screen_y as i32));

        ctx.set(x, y, RGB::named(WHITE), RGB::named(BLACK), rltk::to_cp437('X'));
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
        return FarlookResult::NoResponse { x: ppos.x + x_offset, y: ppos.x + y_offset };
    }
}
