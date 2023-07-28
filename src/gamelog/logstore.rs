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

pub fn print_log(console: &mut Box<dyn Console>, pos: Point, descending: bool, len: usize, maximum_len: i32) {
    let mut y = pos.y;
    let mut x = pos.x;
    LOG.lock().unwrap().iter().rev().take(len).for_each(|log| {
        let mut len_so_far: i32 = 0;
        let mut entry_len = 0;
        log.iter().for_each(|frag| {
            entry_len += frag.text.len() as i32;
        });
        let lines = entry_len / maximum_len;
        y -= lines;
        log.iter().for_each(|frag| {
            if len_so_far + frag.text.len() as i32 > maximum_len {
                y += 1;
                x = pos.x;
                len_so_far = 0;
            }
            if y > pos.y - len as i32 {
                console.print_color(x, y, frag.colour.into(), RGB::named(rltk::BLACK).into(), &frag.text);
            }
            x += frag.text.len() as i32;
            len_so_far += frag.text.len() as i32;
        });
        if descending {
            y += 1;
        } else {
            y -= 1 + lines;
        }
        x = pos.x;
    });
}

pub fn setup_log() {
    clear_log();
    events::clear_events();

    Logger::new()
        .append("Welcome!")
        .colour(rltk::CYAN)
        .append_n("Press [?] at any time to view controls")
        .period()
        .log();
}

pub fn clone_log() -> Vec<Vec<crate::gamelog::LogFragment>> {
    return LOG.lock().unwrap().clone();
}

pub fn restore_log(log: &mut Vec<Vec<crate::gamelog::LogFragment>>) {
    LOG.lock().unwrap().clear();
    LOG.lock().unwrap().append(log);
}
