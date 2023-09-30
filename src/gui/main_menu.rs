use notan::prelude::*;
use notan::draw::{ Draw, CreateDraw, DrawTextSection, Font };
use specs::prelude::*;
use std::collections::HashMap;
use super::{ FONTSIZE, RunState, DISPLAYWIDTH, TILESIZE, MainMenuSelection };
use crate::consts::DISPLAYHEIGHT;

pub fn draw_mainmenu(ecs: &World, draw: &mut Draw, atlas: &HashMap<String, Texture>, font: &Font) {
    let runstate = ecs.read_resource::<RunState>();
    let selected = match *runstate {
        RunState::MainMenu { menu_selection } => menu_selection,
        _ => unreachable!("draw_mainmenu() called outside of MainMenu runstate."),
    };
    let save_exists = crate::saveload_system::does_save_exist();
    const MID_X: f32 = ((DISPLAYWIDTH as f32) * TILESIZE) / 2.0;

    let (x, mut y) = (MID_X, ((DISPLAYHEIGHT as f32) * TILESIZE) / 4.0);
    draw.text(font, "RUST-RL")
        .size(FONTSIZE * 2.0)
        .position(x, y)
        .h_align_center();
    y = draw.last_text_bounds().max_y();
    draw.text(font, "New Game")
        .size(FONTSIZE)
        .position(x, y)
        .h_align_center()
        .color(get_colour(selected, MainMenuSelection::NewGame));
    if save_exists {
        y = draw.last_text_bounds().max_y();
        draw.text(font, "Load Game")
            .size(FONTSIZE)
            .position(x, y)
            .h_align_center()
            .color(get_colour(selected, MainMenuSelection::LoadGame));
    }
    y = draw.last_text_bounds().max_y();
    draw.text(font, "Quit")
        .size(FONTSIZE)
        .position(x, y)
        .h_align_center()
        .color(get_colour(selected, MainMenuSelection::Quit));
}

fn get_colour(selected: MainMenuSelection, desired: MainMenuSelection) -> Color {
    if selected == desired { Color::from_rgb(0.0, 1.0, 0.0) } else { Color::WHITE }
}
