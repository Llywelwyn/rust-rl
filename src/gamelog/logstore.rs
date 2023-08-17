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

pub fn print_log(console: &mut Box<dyn Console>, pos: Point, _descending: bool, len: usize, maximum_len: i32) {
    let mut y = pos.y;
    let mut x = pos.x;
    // Reverse the log, take the number we want to show, and iterate through them
    LOG.lock().unwrap().iter().rev().take(len).for_each(|log| {
        let mut entry_len = -2;
        // Iterate through each message fragment, and get the total length
        // in lines, by adding the length of every fragment and dividing it
        // by the maximum length we desire. Then shuffle our start-y by that much.
        log.iter().for_each(|frag| {
            entry_len += frag.text.len() as i32;
        });
        let lines = entry_len / maximum_len;
        y -= lines;
        let mut i = 0;
        log.iter().for_each(|frag| {
            // Split every fragment up into single characters.
            let parts = frag.text.split("");
            for part in parts {
                // This is an extremely hacky solution to a problem I don't understand yet.
                // -- without this, the lines *here* and the line count *above* wont match.
                if part == "" || part == "\\" {
                    continue;
                }
                if i > entry_len {
                    break;
                }
                i += 1;
                if x + part.len() as i32 > pos.x + maximum_len {
                    if y > pos.y - len as i32 {
                        console.print(x, y, "-");
                    }
                    y += 1;
                    x = pos.x;
                }
                // Stay within bounds
                if y > pos.y - len as i32 {
                    console.print_color(x, y, frag.colour.into(), RGB::named(rltk::BLACK).into(), part);
                }
                x += part.len() as i32;
            }
        });
        // Take away one from the y-axis, because we want to start each entry
        // on a new line, and go up an additional amount depending on how many
        // lines our *previous* entry took.
        y -= 1 + lines;
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
