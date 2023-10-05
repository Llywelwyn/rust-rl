use crate::{
    gamelog,
    gui::unique,
    Beatitude,
    Charges,
    MagicItem,
    MasterDungeonMap,
    Name,
    ObfuscatedName,
    Stackable,
    Renderable,
    WantsToAssignKey,
    WantsToRemoveKey,
    Key,
};
use specs::prelude::*;
use crate::consts::messages;
use bracket_lib::prelude::*;
use crate::invkeys::*;

pub struct KeyHandling {}

const DEBUG_KEYHANDLING: bool = false;

impl<'a> System<'a> for KeyHandling {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, WantsToAssignKey>,
        WriteStorage<'a, WantsToRemoveKey>,
        WriteStorage<'a, Key>,
        ReadStorage<'a, Stackable>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, ObfuscatedName>,
        ReadStorage<'a, Renderable>,
        ReadStorage<'a, Beatitude>,
        ReadStorage<'a, MagicItem>,
        ReadStorage<'a, Charges>,
        ReadExpect<'a, MasterDungeonMap>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            entities,
            mut wants_keys,
            mut wants_removekey,
            mut keys,
            stackable,
            names,
            obfuscated_names,
            renderables,
            beatitudes,
            magic_items,
            wands,
            dm,
        ) = data;

        // For every entity that wants to be picked up, that still needs a key assigned.
        for (e, _wants_key) in (&entities, &wants_keys).join() {
            if DEBUG_KEYHANDLING {
                console::log(&format!("KEYHANDLING: Assigning key to {:?}", e));
            }
            let (stacks, mut handled, unique) = (
                if let Some(_) = stackable.get(e) { true } else { false },
                false,
                unique(
                    e,
                    &names,
                    &obfuscated_names,
                    &renderables,
                    &beatitudes,
                    &magic_items,
                    Some(&wands),
                    &dm
                ),
            );
            if stacks {
                if DEBUG_KEYHANDLING {
                    console::log(&format!("KEYHANDLING: Item is stackable."));
                }
                let maybe_key = item_exists(&unique);
                if maybe_key.is_some() {
                    if DEBUG_KEYHANDLING {
                        console::log(&format!("KEYHANDLING: Existing stack found for this item."));
                    }
                    let key = maybe_key.unwrap();
                    keys.insert(e, Key { idx: key }).expect("Unable to insert Key.");
                    if DEBUG_KEYHANDLING {
                        console::log(&format!("KEYHANDLING: Assigned key idx {} to item.", key));
                    }
                    handled = true;
                }
            }
            if !handled {
                if DEBUG_KEYHANDLING {
                    console::log(
                        &format!("KEYHANDLING: Item is not stackable, or no existing stack found.")
                    );
                }
                if let Some(idx) = assign_next_available() {
                    if DEBUG_KEYHANDLING {
                        console::log(
                            &format!("KEYHANDLING: Assigned next available index {} to item.", idx)
                        );
                    }
                    keys.insert(e, Key { idx }).expect("Unable to insert Key.");
                    register_stackable(stacks, unique, idx);
                } else {
                    if DEBUG_KEYHANDLING {
                        console::log(&format!("KEYHANDLING: No more keys available."));
                    }
                    gamelog::Logger
                        ::new()
                        .append(messages::NO_MORE_KEYS)
                        .colour(WHITE)
                        .period()
                        .log();
                }
            }
        }
        for (e, _wants_key) in (&entities, &wants_removekey).join() {
            let idx = keys.get(e).unwrap().idx;
            if DEBUG_KEYHANDLING {
                console::log(&format!("KEYHANDLING: Removing key from {:?}", e));
            }
            // If the item is *not* stackable, then we can just remove the key and clear the index.
            if let None = stackable.get(e) {
                if DEBUG_KEYHANDLING {
                    console::log(
                        &format!("KEYHANDLING: Item is not stackable, clearing index {}.", idx)
                    );
                }
                clear_idx(idx);
                keys.remove(e);
                continue;
            }
            // If the item *is* stackable, then we need to check if there are any other items that
            // share this key assignment, before clearing the index.
            if DEBUG_KEYHANDLING {
                console::log(
                    &format!(
                        "KEYHANDLING: Item is stackable, checking if any other items share this key."
                    )
                );
            }
            let mut sole_item_with_key = true;
            for (entity, key) in (&entities, &keys).join() {
                if entity != e && key.idx == idx {
                    if DEBUG_KEYHANDLING {
                        console::log(&format!("KEYHANDLING: Another item shares index {}", idx));
                    }
                    sole_item_with_key = false;
                    break;
                }
            }
            // If no other items shared this key, free up the index.
            if sole_item_with_key {
                if DEBUG_KEYHANDLING {
                    console::log(
                        &format!("KEYHANDLING: No other items found, clearing index {}.", idx)
                    );
                }
                clear_idx(idx);
            }
            // Either way, remove the key component from this item, because we're dropping it.
            if DEBUG_KEYHANDLING {
                console::log(&format!("KEYHANDLING: Removing key component from item."));
            }
            keys.remove(e);
        }

        wants_removekey.clear();
        wants_keys.clear();
    }
}
