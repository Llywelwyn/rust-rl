use super::State;
use rltk::prelude::*;

#[derive(PartialEq, Copy, Clone)]
pub enum CheatMenuResult {
    NoResponse,
    Cancel,
    Ascend,
    Descend,
    Heal,
    MagicMap,
    GodMode,
}

pub fn show_cheat_menu(_gs: &mut State, ctx: &mut Rltk) -> CheatMenuResult {
    let (x_offset, y_offset) = (1, 10);
    ctx.print_color(
        1 + x_offset,
        1 + y_offset,
        RGB::named(rltk::RED),
        RGB::named(rltk::BLACK),
        "DEBUG MENU! [aA-zZ][Esc.]",
    );
    let x = 1 + x_offset;
    let mut y = 3 + y_offset;
    let count = 5;
    let width = 19;

    ctx.draw_box(x, y, width, (count + 1) as i32, RGB::named(rltk::RED), RGB::named(rltk::BLACK));
    y += 1;
    // Asc
    ctx.set(x_offset + 2, y, RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK), rltk::to_cp437('a'));
    ctx.print(x_offset + 4, y, "ASCEND A FLOOR");
    y += 1;
    // Desc
    ctx.set(x_offset + 2, y, RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK), rltk::to_cp437('d'));
    ctx.print(x_offset + 4, y, "DESCEND A FLOOR");
    y += 1;
    // Heal
    ctx.set(x_offset + 2, y, RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK), rltk::to_cp437('h'));
    ctx.print(x_offset + 4, y, "HEAL TO FULL");
    y += 1;
    // Reveal map
    ctx.set(x_offset + 2, y, RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK), rltk::to_cp437('m'));
    ctx.print(x_offset + 4, y, "MAGIC MAP REVEAL");
    y += 1;
    // Godmode
    ctx.set(x_offset + 2, y, RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK), rltk::to_cp437('g'));
    ctx.print(x_offset + 4, y, "GOD MODE");
    y += 1;
    // Match keys
    match ctx.key {
        None => CheatMenuResult::NoResponse,
        Some(key) => match key {
            VirtualKeyCode::A => CheatMenuResult::Ascend,
            VirtualKeyCode::D => CheatMenuResult::Descend,
            VirtualKeyCode::H => CheatMenuResult::Heal,
            VirtualKeyCode::M => CheatMenuResult::MagicMap,
            VirtualKeyCode::G => CheatMenuResult::GodMode,
            VirtualKeyCode::Escape => CheatMenuResult::Cancel,
            _ => CheatMenuResult::NoResponse,
        },
    }
}
