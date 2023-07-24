use super::{events, LogFragment, Logger};
use rltk::prelude::*;
use std::sync::Mutex;

lazy_static! {
    static ref LOG: Mutex<Vec<Vec<LogFragment>>> = Mutex::new(Vec::new());
}

#[allow(dead_code)]
pub fn append_fragment(fragment: LogFragment) {
    LOG.lock().unwrap().push(vec![fragment]);
}

pub fn append_entry(fragments: Vec<LogFragment>) {
    LOG.lock().unwrap().push(fragments);
}

pub fn clear_log() {
    LOG.lock().unwrap().clear();
}

pub fn print_log(console: &mut Box<dyn Console>, pos: Point, descending: bool, len: usize) {
    let mut y = pos.y;
    let mut x = pos.x;
    LOG.lock().unwrap().iter().rev().take(len).for_each(|log| {
        log.iter().for_each(|frag| {
            console.print_color(x, y, frag.colour.into(), RGB::named(rltk::BLACK).into(), &frag.text);
            x += frag.text.len() as i32;
            x += 0;
        });
        if descending {
            y += 1;
        } else {
            y -= 1;
        }
        x = pos.x;
    });
}

pub fn setup_log() {
    clear_log();
    events::clear_events();
    for _ in 0..5 {
        Logger::new().log();
    }
    Logger::new().append("Welcome!").colour(rltk::CYAN).append("Press [?] at any time to view controls").period().log();
}

pub fn clone_log() -> Vec<Vec<crate::gamelog::LogFragment>> {
    return LOG.lock().unwrap().clone();
}

pub fn restore_log(log: &mut Vec<Vec<crate::gamelog::LogFragment>>) {
    LOG.lock().unwrap().clear();
    LOG.lock().unwrap().append(log);
}
