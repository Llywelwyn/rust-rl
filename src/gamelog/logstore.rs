use super::{ events, LogFragment, Logger };
use bracket_lib::prelude::*;
use std::sync::Mutex;
use std::collections::BTreeMap;
use notan::prelude::*;
use notan::text::CreateText;
use crate::consts::{ TILESIZE, FONTSIZE };
use crate::consts::visuals::VIEWPORT_W;
use crate::Fonts;

lazy_static! {
    pub static ref LOG: Mutex<BTreeMap<i32, Vec<LogFragment>>> = Mutex::new(BTreeMap::new());
}

/// Render with defaults, to avoid having to pass so many args.
pub fn render(draw: bool, gfx: &mut Graphics, font: &Fonts) {
    if draw {
        render_log(
            gfx,
            &font,
            &(TILESIZE, TILESIZE * 8.0 + 4.0),
            (VIEWPORT_W as f32) * TILESIZE,
            7
        );
    }
}

/// Render with specificied params.
pub fn render_log(gfx: &mut Graphics, font: &Fonts, pos: &(f32, f32), width: f32, entries: usize) {
    let mut text = gfx.create_text();
    let log = LOG.lock().unwrap();
    let latest: Vec<_> = log.iter().rev().take(entries).collect();
    let mut initialised = false;
    let mut y = pos.1;
    for (_, entries) in latest {
        let mut written_on_line = false;
        for frag in entries.iter() {
            if !written_on_line {
                text.add(&frag.text)
                    .font(font.n())
                    .position(pos.0, y)
                    .size(FONTSIZE)
                    .max_width(width)
                    .color(Color::from_rgb(frag.colour.r, frag.colour.g, frag.colour.b))
                    .v_align_bottom();
                written_on_line = true;
                initialised = true;
            } else {
                text.chain(&frag.text)
                    .color(Color::from_rgb(frag.colour.r, frag.colour.g, frag.colour.b))
                    .size(FONTSIZE);
            }
        }
        if initialised {
            y = text.last_bounds().min_y();
        }
    }
    gfx.render(&text);
}

pub fn append_entry(turn: i32, fragments: Vec<LogFragment>) {
    let mut log = LOG.lock().unwrap();
    if let Some(existing) = log.get_mut(&turn) {
        existing.extend(fragments);
    } else {
        log.insert(turn, fragments);
    }
}

pub fn clear_log() {
    LOG.lock().unwrap().clear();
}

pub fn setup_log() {
    clear_log();
    events::clear_events();

    Logger::new()
        .append("Welcome!")
        .colour(CYAN)
        .append_n("Press [?] at any time to view controls")
        .period()
        .log();
}

pub fn clone_log() -> BTreeMap<i32, Vec<LogFragment>> {
    return LOG.lock().unwrap().clone();
}

pub fn restore_log(log: &mut BTreeMap<i32, Vec<LogFragment>>) {
    LOG.lock().unwrap().clear();
    LOG.lock().unwrap().append(log);
}
