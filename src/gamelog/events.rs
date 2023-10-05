use std::collections::{ HashSet, HashMap };
use std::sync::Mutex;
use crate::consts::events::EVENT;
use crate::consts::names::*;

lazy_static! {
    /// A count of each event that has happened over the run. i.e. "turns", "descended", "ascended"
    pub static ref EVENT_COUNTER: Mutex<HashMap<String, i32>> = Mutex::new(HashMap::new());
    // A record of events that happened on a given turn. i.e. "Advanced to level 2".
    pub static ref EVENTS: Mutex<HashMap<u32, Vec<String>>> = Mutex::new(HashMap::new());
    // A record of floors visited, and monsters killed. Used to determine if an event is significant.
    static ref VISITED: Mutex<HashSet<String>> = Mutex::new({
        let mut set = HashSet::new();
        set.insert(NAME_OVERMAP.to_string());
        set
    });
    static ref KILLED: Mutex<HashSet<String>> = Mutex::new(HashSet::new());
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
    let (mut events, mut event_counts) = (EVENTS.lock().unwrap(), EVENT_COUNTER.lock().unwrap());
    events.clear();
    event_counts.clear();
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
        EVENT::Turn(n) => {
            modify_event_count(EVENT::COUNT_TURN, n);
            significant_event = false;
        }
        // If de-levelling is ever implemented, this needs refactoring (along with a lot of stuff).
        EVENT::Level(n) => {
            modify_event_count(EVENT::COUNT_LEVEL, n);
            let new_lvl = get_event_count(EVENT::COUNT_LEVEL);
            if new_lvl == 1 {
                new_event = format!("You embarked on your first adventure!");
            } else {
                new_event = format!("Advanced to level {}", new_lvl);
            }
        }
        EVENT::ChangedFloor(n) => {
            modify_event_count(EVENT::COUNT_CHANGED_FLOOR, 1);
            if VISITED.lock().unwrap().contains(&n) {
                significant_event = false;
            } else {
                VISITED.lock().unwrap().insert(n.clone());
                new_event = format!("Visited {} for the first time", n);
            }
        }
        EVENT::KickedSomething(n) => {
            modify_event_count(EVENT::COUNT_KICK, n);
            significant_event = false;
        }
        EVENT::BrokeDoor(n) => {
            modify_event_count(EVENT::COUNT_BROKE_DOOR, n);
            significant_event = false;
        }
        EVENT::PlayerConfused(n) => {
            modify_event_count(EVENT::COUNT_PLAYER_CONFUSED, n);
            significant_event = false;
        }
        EVENT::LookedForHelp(n) => {
            modify_event_count(EVENT::COUNT_LOOKED_FOR_HELP, n);
            significant_event = false;
        }
        EVENT::Killed(name) => {
            modify_event_count(EVENT::COUNT_KILLED, 1);
            if KILLED.lock().unwrap().contains(&name) {
                significant_event = false;
            } else {
                KILLED.lock().unwrap().insert(name.clone());
                new_event = format!("Killed your first {}", name);
            }
        }
        EVENT::Discovered(name) => {
            new_event = format!("Discovered {}", name);
        }
        EVENT::Identified(name) => {
            new_event = format!("Identified {}", crate::gui::with_article(&name));
        }
        EVENT::PlayerDied(str) => {
            // Generating the String is handled in the death effect, to avoid passing the ecs here.
            new_event = format!("{}", str);
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
