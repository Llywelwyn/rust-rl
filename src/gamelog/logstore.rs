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
    // Start at x, y
    let mut y = pos.y;
    let mut x = pos.x;
    // Reverse the log, take the number we want to show, and iterate through them
    LOG.lock().unwrap().iter().rev().take(len).for_each(|log| {
        let mut len_so_far: i32 = 0;
        let mut entry_len = 0;
        // Iterate through each message fragment, and get the total length
        // in lines, by adding the length of every fragment and dividing it
        // by the maximum length we desire.
        log.iter().for_each(|frag| {
            entry_len += frag.text.len() as i32;
        });
        // I don't know why the +1 is required, or why there were issues on what seemed to be
        // specified a value of 68. I'm pretty sure it's a ""rounding error"" between this method
        // of determining max lines and how the iterator actually counts the characters. Regardless,
        // this seems to work. -- NOTE IN CASE THIS ISSUE COMES BACK? HARD TO REPRODUCE!
        let lines = entry_len / (maximum_len + 1);
        // If the fragment is more than one line long, move our y-value up
        // by this much.
        y -= lines;
        // Iterate through each fragment now, for the draw loop
        log.iter().for_each(|frag| {
            // Split every fragment up into single characters
            let parts = frag.text.split("");
            // For every character, check if the length will exceed
            // the maximum length we're looking for. If it will, go
            // down 1 in the y-axis, return us to the start of the line,
            // and reset our length counter to 0.
            for part in parts {
                if len_so_far + part.len() as i32 > maximum_len {
                    if y > pos.y - len as i32 {
                        console.print(x, y, "-");
                    }
                    y += 1;
                    x = pos.x;
                    len_so_far = 0;
                }
                // If we're still within our "range" (we haven't gone up
                // further in the y-axis than our desired amount), then
                // print the next character. Otherwise, just skip it.
                // -- this makes sure we don't continue drawing outside of
                // the bounds of our message box.
                if y > pos.y - len as i32 {
                    console.print_color(x, y, frag.colour.into(), RGB::named(rltk::BLACK).into(), part);
                }
                // Move across by 1 in the x-axis, and add the length to our counter.
                x += part.len() as i32;
                len_so_far += part.len() as i32;
            }
        });
        // Descending is deprecated for now, so we always ascending upwards.
        // Take away one from the y-axis, because we want to start each entry
        // on a new line, and go up an additional amount depending on how many
        // lines our *previous* entry took.
        y -= 1 + lines;
        // Go back to the start of the new line.
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
