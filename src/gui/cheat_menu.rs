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
    _atlas: &HashMap<String, Texture>,
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
        .position(3.0 + (offsets.x as f32) * TILESIZE, 3.0 + (offsets.y as f32) * TILESIZE)
        .color(Color::RED)
        .size(TILESIZE);
}
