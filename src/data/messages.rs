pub const NOCHARGES_WREST: &str = "You wrest one last charge from the worn-out wand.";
pub const NOCHARGES_DIDNOTHING: &str = "The wand does nothing.";

pub const IDENTIFY_ALL: &str = "You feel attuned to your belongings!";
pub const IDENTIFY_ALL_BLESSED: &str = "Divine favour reveals all";

pub const REMOVECURSE: &str = "You feel a weight lifted!";
pub const REMOVECURSE_BLESSED: &str = "You feel righteous";
pub const REMOVECURSE_BLESSED_FAILED: &str = "You feel righteous! But nothing happened.";

pub const DAMAGE_PLAYER_HIT: &str = "are hit!";
pub const DAMAGE_ITEM_HIT: &str = "is ruined!";
pub const DAMAGE_OTHER_HIT: &str = "is hit!";

pub const HEAL_PLAYER_HIT: &str = "recover some vigour.";
pub const HEAL_PLAYER_HIT_BLESSED: &str = "You feel great";
pub const HEAL_OTHER_HIT: &str = "is rejuvenated!";

pub const MAGICMAP: &str = "You recall your surroundings!";
pub const MAGICMAP_CURSED: &str = "... but forget where you last were";

pub const NUTRITION: &str = "You eat the";
pub const NUTRITION_CURSED: &str = "Blech! Rotten";
pub const NUTRITION_BLESSED: &str = "Delicious";

pub const LEVELUP_PLAYER: &str = "Welcome to experience level";
pub const YOU_PICKUP_ITEM: &str = "You pick up the";
pub const YOU_DROP_ITEM: &str = "You drop the";
pub const YOU_EQUIP_ITEM: &str = "You equip the";
pub const YOU_REMOVE_ITEM: &str = "You unequip your";
pub const YOU_REMOVE_ITEM_CURSED: &str = "You can't remove the";

/// Prefixes death message.
pub const PLAYER_DIED: &str = "You died!";
/// Death message specifiers. Appended after PLAYER_DIED.
pub const PLAYER_DIED_SUICIDE: &str = "You killed yourself";
pub const PLAYER_DIED_NAMED_ATTACKER: &str = "You were killed by";
pub const PLAYER_DIED_UNKNOWN: &str = "You were killed"; // Ultimately, this should never be used. Slowly include specific messages for any death.
/// Death message addendums. Appended at end of death message.
pub const PLAYER_DIED_ADDENDUM_FIRST: &str = " ";
pub const PLAYER_DIED_ADDENDUM_MID: &str = ", ";
pub const PLAYER_DIED_ADDENDUM_LAST: &str = ", and ";
pub const STATUS_CONFUSED_STRING: &str = "confused";
pub const STATUS_BLIND_STRING: &str = "blinded";
// Results in something like: "You died! You were killed by a kobold captain, whilst confused."
