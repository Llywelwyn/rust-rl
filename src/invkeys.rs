use std::sync::Mutex;
use std::collections::{ HashMap };
use specs::prelude::*;
use crate::gui::UniqueInventoryItem;

lazy_static! {
    pub static ref INVKEYS: Mutex<HashMap<UniqueInventoryItem, usize>> = Mutex::new(HashMap::new());
    pub static ref ASSIGNEDKEYS: Mutex<Vec<bool>> = Mutex::new(vec![false; 52]);
}

/// For (de)serialization.
pub fn clone_invkeys() -> HashMap<UniqueInventoryItem, usize> {
    let invkeys = INVKEYS.lock().unwrap();
    invkeys.clone()
}
pub fn restore_invkeys(invkeys: HashMap<UniqueInventoryItem, usize>) {
    INVKEYS.lock().unwrap().clear();
    INVKEYS.lock().unwrap().extend(invkeys);
}

pub fn item_exists(item: &UniqueInventoryItem) -> Option<usize> {
    let invkeys = INVKEYS.lock().unwrap();
    use bracket_lib::prelude::*;
    console::log(&format!("{:?}", item));
    if invkeys.contains_key(item) {
        Some(*invkeys.get(item).unwrap())
    } else {
        None
    }
}

pub fn assign_next_available() -> Option<usize> {
    let mut lock = ASSIGNEDKEYS.lock().unwrap();
    for (i, key) in lock.iter_mut().enumerate() {
        if !*key {
            *key = true;
            return Some(i);
        }
    }
    None
}

pub fn register_stackable(stacks: bool, item: UniqueInventoryItem, idx: usize) {
    if stacks {
        let mut invkeys = INVKEYS.lock().unwrap();
        invkeys.insert(item, idx);
    }
}

pub fn clear_idx(idx: usize) {
    let mut lock = ASSIGNEDKEYS.lock().unwrap();
    lock[idx] = false;
    let mut invkeys = INVKEYS.lock().unwrap();
    invkeys.retain(|_k, v| *v != idx);
}
