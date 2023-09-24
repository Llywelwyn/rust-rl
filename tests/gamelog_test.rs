// tests/gamelog_test.rs
use rust_rl::gamelog::*;
use rust_rl::consts::events::*;
use lazy_static::lazy_static;
use std::sync::Mutex;

// To ensure this test module uses a single thread.
lazy_static! {
    static ref SINGLE_THREAD: Mutex<()> = Mutex::new(());
}

#[test]
fn recording_event() {
    let _lock = SINGLE_THREAD.lock();
    clear_events();
    record_event(EVENT::Turn(1));
    record_event(EVENT::Turn(0));
    record_event(EVENT::Turn(-1));
    record_event(EVENT::Killed("mob".to_string()));
}

#[test]
fn getting_event_count() {
    let _lock = SINGLE_THREAD.lock();
    clear_events();
    record_event(EVENT::Turn(1));
    assert_eq!(get_event_count(EVENT::COUNT_TURN), 1);
    record_event(EVENT::Turn(3));
    assert_eq!(get_event_count(EVENT::COUNT_TURN), 4);
    clear_events();
    assert_eq!(get_event_count(EVENT::COUNT_TURN), 0);
}

#[test]
fn cloning_events() {
    let _lock = SINGLE_THREAD.lock();
    clear_events();
    record_event(EVENT::Level(1));
    record_event(EVENT::Turn(5));
    record_event(EVENT::Identified("item".to_string()));
    let cloned_events = clone_events();
    assert_eq!(EVENTS.lock().unwrap().clone(), cloned_events);
}
