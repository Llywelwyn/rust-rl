use rltk::RGB;
use serde::{Deserialize, Serialize};
use specs::error::NoError;
use specs::prelude::*;
use specs::saveload::{ConvertSaveload, Marker};
use specs_derive::*;
use std::collections::HashMap;

// Serialization helper code. We need to implement ConvertSaveload for each type that contains an
// Entity.
pub struct SerializeMe;
// Special component that exists to help serialize the game data
#[derive(Component, Serialize, Deserialize, Clone)]
pub struct SerializationHelper {
    pub map: super::map::Map,
    pub log: Vec<Vec<crate::gamelog::LogFragment>>,
    pub events: HashMap<String, i32>,
}

#[derive(Component, ConvertSaveload, Clone)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

#[derive(Component, ConvertSaveload, Clone)]
pub struct Renderable {
    pub glyph: rltk::FontCharType,
    pub fg: RGB,
    pub bg: RGB,
    pub render_order: i32,
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Player {}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Prop {}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Monster {}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct LootTable {
    pub table: String,
    pub chance: f32,
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Energy {
    pub current: i32,
    pub speed: i32,
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct TakingTurn {}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Bystander {}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Quips {
    pub available: Vec<String>,
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Mind {}

#[derive(Component, ConvertSaveload, Clone)]
pub struct Viewshed {
    pub visible_tiles: Vec<rltk::Point>,
    pub range: i32,
    pub dirty: bool,
}

#[derive(Component, ConvertSaveload, Clone)]
pub struct Telepath {
    pub telepath_tiles: Vec<rltk::Point>,
    pub range: i32,
    pub dirty: bool,
}

#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct Name {
    pub name: String,
    pub plural: String,
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct BlocksTile {}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct BlocksVisibility {}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Door {
    pub open: bool,
}

#[derive(Serialize, Deserialize, Copy, Clone, PartialEq)]
pub enum HungerState {
    Satiated,
    Normal,
    Hungry,
    Weak,
    Fainting,
}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct HungerClock {
    pub state: HungerState,
    pub duration: i32,
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct ProvidesNutrition {}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Pool {
    pub max: i32,
    pub current: i32,
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Pools {
    pub hit_points: Pool,
    pub mana: Pool,
    pub xp: i32,
    pub bac: i32,
    pub level: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Attribute {
    pub base: i32,
    pub modifiers: i32,
    pub bonus: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq, Hash)]
pub enum Skill {
    Melee,
    Defence,
    Magic,
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Skills {
    pub skills: HashMap<Skill, i32>,
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Attributes {
    pub strength: Attribute,
    pub dexterity: Attribute,
    pub constitution: Attribute,
    pub intelligence: Attribute,
    pub wisdom: Attribute,
    pub charisma: Attribute,
}

#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct WantsToMelee {
    pub target: Entity,
}

#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct GrantsXP {
    pub amount: i32,
}

#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct SufferDamage {
    pub amount: Vec<(i32, bool)>,
}

impl SufferDamage {
    pub fn new_damage(store: &mut WriteStorage<SufferDamage>, victim: Entity, amount: i32, from_player: bool) {
        if let Some(suffering) = store.get_mut(victim) {
            suffering.amount.push((amount, from_player));
        } else {
            let dmg = SufferDamage { amount: vec![(amount, from_player)] };
            store.insert(victim, dmg).expect("Unable to insert damage.");
        }
    }
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Item {}

#[derive(PartialEq, Copy, Clone, Serialize, Deserialize)]
pub enum EquipmentSlot {
    Melee,
    Shield,
    Head,
    Body,
    Hands,
    Feet,
    Neck,
    Back,
}

#[derive(PartialEq, Copy, Clone, Serialize, Deserialize)]
pub enum WeaponAttribute {
    Strength,
    Dexterity,
    Finesse,
}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct MeleeWeapon {
    pub attribute: WeaponAttribute,
    pub damage_n_dice: i32,
    pub damage_die_type: i32,
    pub damage_bonus: i32,
    pub hit_bonus: i32,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct NaturalAttack {
    pub name: String,
    pub damage_n_dice: i32,
    pub damage_die_type: i32,
    pub damage_bonus: i32,
    pub hit_bonus: i32,
}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct NaturalAttacks {
    pub attacks: Vec<NaturalAttack>,
}

#[derive(Component, ConvertSaveload, Clone)]
pub struct ArmourClassBonus {
    pub amount: i32,
}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct Equippable {
    pub slot: EquipmentSlot,
}

#[derive(Component, ConvertSaveload, Clone)]
pub struct Equipped {
    pub owner: Entity,
    pub slot: EquipmentSlot,
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Cursed {}

#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct ProvidesHealing {
    pub amount: i32,
}

#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct InflictsDamage {
    pub amount: i32,
}

#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct Ranged {
    pub range: i32,
}

#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct AOE {
    pub radius: i32,
}

#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct Confusion {
    pub turns: i32,
}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct MagicMapper {}

#[derive(Component, Debug, ConvertSaveload)]
pub struct InBackpack {
    pub owner: Entity,
}

#[derive(Component, Debug, ConvertSaveload)]
pub struct WantsToPickupItem {
    pub collected_by: Entity,
    pub item: Entity,
}

#[derive(Component, Debug, ConvertSaveload)]
pub struct WantsToDropItem {
    pub item: Entity,
}

#[derive(Component, Debug, ConvertSaveload)]
pub struct WantsToRemoveItem {
    pub item: Entity,
}

#[derive(Component, Debug, ConvertSaveload)]
pub struct WantsToUseItem {
    pub item: Entity,
    pub target: Option<rltk::Point>,
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Consumable {}

#[derive(Component, Debug, ConvertSaveload)]
pub struct Wand {
    pub uses: i32,
    pub max_uses: i32,
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Destructible {}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Digger {}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Hidden {}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct SingleActivation {}

#[derive(Component, Clone, ConvertSaveload)]
pub struct ParticleLifetime {
    pub lifetime_ms: f32,
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct EntryTrigger {}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct EntityMoved {}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct MultiAttack {}
