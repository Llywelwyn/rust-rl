use std::collections::{ HashSet, HashMap };
use std::sync::Mutex;
use crate::data::events::EVENT;

lazy_static! {
    /// A count of each event that has happened over the run. i.e. "turns", "descended", "ascended"
    static ref EVENT_COUNTER: Mutex<HashMap<String, i32>> = Mutex::new(HashMap::new());
    // A record of events that happened on a given turn. i.e. "Advanced to level 2".
    pub static ref EVENTS: Mutex<HashMap<u32, Vec<String>>> = Mutex::new(HashMap::new());
    static ref VISITED: Mutex<HashSet<i32>> = Mutex::new(HashSet::new());
}

/// Makes a copy of event counts (FOR SERIALIZATION)
pub fn clone_event_counts() -> HashMap<String, i32> {
    EVENT_COUNTER.lock().unwrap().clone()
}
/// Makes a copy of events (FOR SERIALIZATION)
pub fn clone_events() -> HashMap<u32, Vec<String>> {
    EVENTS.lock().unwrap().clone()
}
/// Fetches event counter into mutex (FOR DESERIALIZATION)
pub fn restore_event_counter(events: HashMap<String, i32>) {
    EVENT_COUNTER.lock().unwrap().clear();
    events.iter().for_each(|(k, v)| {
        EVENT_COUNTER.lock().unwrap().insert(k.to_string(), *v);
    });
}
/// Fetches events into mutex (FOR DESERIALIZATION)
pub fn restore_events(events: HashMap<u32, Vec<String>>) {
    EVENTS.lock().unwrap().clear();
    events.iter().for_each(|(k, v)| {
        EVENTS.lock().unwrap().insert(*k, v.to_vec());
    });
}
/// Wipes all events - for starting a new game.
pub fn clear_events() {
    EVENT_COUNTER.lock().unwrap().clear();
    EVENTS.lock().unwrap().clear();
}

#[allow(unused_mut)]
/// Increments the event counter by n for a given event.
fn modify_event_count<T: ToString>(event: T, n: i32) {
    let event_name = event.to_string();
    let mut events_lock = EVENT_COUNTER.lock();
    let mut events = events_lock.as_mut().unwrap();
    if let Some(e) = events.get_mut(&event_name) {
        *e += n;
    } else {
        events.insert(event_name, n);
    }
}
/// Returns how many times an event has taken place.
pub fn get_event_count<T: ToString>(event: T) -> i32 {
    let event_name = event.to_string();
    let events_lock = EVENT_COUNTER.lock();
    let events = events_lock.unwrap();
    if let Some(e) = events.get(&event_name) {
        *e
    } else {
        0
    }
}
/// Records an event on the current turn.
pub fn record_event(event: EVENT) {
    let mut new_event: String = "unknown event".to_string();
    let mut significant_event = true;
    match event {
        EVENT::TURN(n) => {
            modify_event_count(EVENT::COUNT_TURN, n);
            significant_event = false;
        }
        EVENT::LEVEL(n) => {
            modify_event_count(EVENT::COUNT_LEVEL, n);
            new_event = format!("Advanced to level {}", n);
        }
        EVENT::CHANGED_FLOOR(n) => {
            modify_event_count(EVENT::COUNT_CHANGED_FLOOR, 1);
            if VISITED.lock().unwrap().contains(&n) {
                significant_event = false;
            } else {
                VISITED.lock().unwrap().insert(n);
                new_event = format!("Visited level {} for the first time", n);
            }
        }
        EVENT::KICKED_SOMETHING(n) => {
            modify_event_count(EVENT::COUNT_KICK, n);
            significant_event = false;
        }
        EVENT::BROKE_DOOR(n) => {
            modify_event_count(EVENT::COUNT_BROKE_DOOR, n);
            significant_event = false;
        }
        EVENT::PLAYER_CONFUSED(n) => {
            modify_event_count(EVENT::COUNT_PLAYER_CONFUSED, n);
            significant_event = false;
        }
        EVENT::LOOKED_FOR_HELP(n) => {
            modify_event_count(EVENT::COUNT_LOOKED_FOR_HELP, n);
            significant_event = false;
        }
        EVENT::KILLED(name) => {
            new_event = format!("Killed {}", name);
        }
        EVENT::DISCOVERED(name) => {
            new_event = format!("Discovered {}", name);
        }
        EVENT::IDENTIFIED(name) => {
            new_event = format!("Identified {}", name);
        }
    }

    if significant_event {
        EVENTS.lock()
            .as_mut()
            .unwrap()
            .entry(get_event_count(EVENT::COUNT_TURN) as u32)
            .or_insert_with(Vec::new)
            .push(new_event);
    }
}
