use super::{ State };
use bracket_lib::prelude::*;
use notan::prelude::*;
use notan::draw::DrawTextSection;
use std::collections::HashMap;
use crate::consts::TILESIZE;

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

pub fn show_cheat_menu(_gs: &mut State, ctx: &mut App) -> CheatMenuResult {
    let key = &ctx.keyboard;
    for keycode in key.pressed.iter() {
        match *keycode {
            KeyCode::A => {
                return CheatMenuResult::Ascend;
            }
            KeyCode::D => {
                return CheatMenuResult::Descend;
            }
            KeyCode::H => {
                return CheatMenuResult::Heal;
            }
            KeyCode::M => {
                return CheatMenuResult::MagicMap;
            }
            KeyCode::G => {
                return CheatMenuResult::GodMode;
            }
            KeyCode::Escape => {
                return CheatMenuResult::Cancel;
            }
            _ => {}
        };
    }
    return CheatMenuResult::NoResponse;
}

pub fn draw_cheat_menu(
    draw: &mut notan::draw::Draw,
    atlas: &HashMap<String, Texture>,
    font: &notan::draw::Font
) {
    let offsets = crate::camera::get_offset();
    const DEBUG_MENU: &str =
        r#"DEBUG MENU! [aA-zZ][Esc.]
    
    a - ASCEND A FLOOR
    d - DESCEND A FLOOR
    h - HEAL TO FULL
    m - MAGIC MAP REVEAL
    g - GOD MODE"#;
    draw.text(&font, DEBUG_MENU)
        .position(1.0 + (offsets.x as f32) * TILESIZE, 1.0 + (offsets.y as f32) * TILESIZE)
        .color(Color::RED);
    /*let (x_offset, y_offset) = (1, 10);
    ctx.print_color(
        1 + x_offset,
        1 + y_offset,
        RGB::named(RED),
        RGB::named(BLACK),
        "DEBUG MENU! [aA-zZ][Esc.]"
    );
    let x = 1 + x_offset;
    let mut y = 3 + y_offset;
    let count = 5;
    let width = 19;

    ctx.draw_box(x, y, width, (count + 1) as i32, RGB::named(RED), RGB::named(BLACK));
    y += 1;
    // Asc
    ctx.set(x_offset + 2, y, RGB::named(YELLOW), RGB::named(BLACK), to_cp437('a'));
    ctx.print(x_offset + 4, y, "ASCEND A FLOOR");
    y += 1;
    // Desc
    ctx.set(x_offset + 2, y, RGB::named(YELLOW), RGB::named(BLACK), to_cp437('d'));
    ctx.print(x_offset + 4, y, "DESCEND A FLOOR");
    y += 1;
    // Heal
    ctx.set(x_offset + 2, y, RGB::named(YELLOW), RGB::named(BLACK), to_cp437('h'));
    ctx.print(x_offset + 4, y, "HEAL TO FULL");
    y += 1;
    // Reveal map
    ctx.set(x_offset + 2, y, RGB::named(YELLOW), RGB::named(BLACK), to_cp437('m'));
    ctx.print(x_offset + 4, y, "MAGIC MAP REVEAL");
    y += 1;
    // Godmode
    ctx.set(x_offset + 2, y, RGB::named(YELLOW), RGB::named(BLACK), to_cp437('g'));
    ctx.print(x_offset + 4, y, "GOD MODE");*/
}
