use crate::gui::Ancestry;
use crate::gui::Class;
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
}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct DMSerializationHelper {
    pub map: super::map::MasterDungeonMap,
    pub log: Vec<Vec<crate::gamelog::LogFragment>>,
    pub events: HashMap<String, i32>,
}

#[derive(Component, ConvertSaveload, Clone)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct OtherLevelPosition {
    pub x: i32,
    pub y: i32,
    pub id: i32,
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
pub struct Faction {
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq, Hash)]
pub enum Movement {
    Static,
    Random,
    RandomWaypoint { path: Option<Vec<usize>> },
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct MoveMode {
    pub mode: Movement,
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Prop {}

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
pub struct Clock {}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct TakingTurn {}

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
    Starving,
}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct HungerClock {
    pub state: HungerState,
    pub duration: i32,
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct ProvidesNutrition {}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct HasAncestry {
    pub name: Ancestry,
}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct HasClass {
    pub name: Class,
}

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
    pub weight: f32,
    pub god: bool,
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

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq, Hash)]
pub enum BUC {
    Cursed,
    Uncursed,
    Blessed,
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Beatitude {
    pub buc: BUC,
    pub known: bool,
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Item {
    pub weight: f32, // in lbs
    pub value: f32,  // base
}

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq, Hash)]
pub enum MagicItemClass {
    Common,
    Uncommon,
    Rare,
    VeryRare,
    Legendary,
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct MagicItem {
    pub class: MagicItemClass,
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct ObfuscatedName {
    pub name: String,
    pub plural: String,
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct IdentifiedItem {
    pub name: String,
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct EquipmentChanged {}

#[derive(PartialEq, Copy, Clone, Serialize, Deserialize)]
pub enum BurdenLevel {
    Burdened,
    Strained,
    Overloaded,
}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct Burden {
    pub level: BurdenLevel,
}

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

#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct ProvidesHealing {
    pub n_dice: i32,
    pub sides: i32,
    pub modifier: i32,
}

#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct InflictsDamage {
    pub n_dice: i32,
    pub sides: i32,
    pub modifier: i32,
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
pub struct WantsToApproach {
    pub idx: i32,
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct WantsToFlee {
    pub indices: Vec<usize>, // Dijkstra
}

#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct Chasing {
    pub target: Entity,
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Consumable {}

#[derive(Component, Debug, ConvertSaveload)]
pub struct Charges {
    pub uses: i32,
    pub max_uses: i32,
}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct SpawnParticleLine {
    pub glyph: rltk::FontCharType,
    pub tail_glyph: rltk::FontCharType,
    pub colour: RGB,
    pub lifetime_ms: f32,
    pub trail_colour: RGB,
    pub trail_lifetime_ms: f32,
}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct SpawnParticleSimple {
    pub glyph: rltk::FontCharType,
    pub colour: RGB,
    pub lifetime_ms: f32,
}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct SpawnParticleBurst {
    pub glyph: rltk::FontCharType,
    pub head_glyph: rltk::FontCharType,
    pub tail_glyph: rltk::FontCharType,
    pub colour: RGB,
    pub lerp: RGB,
    pub lifetime_ms: f32,
    pub trail_colour: RGB,
    pub trail_lifetime_ms: f32,
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
