use crate::gui::Ancestry;
use crate::gui::Class;
use bracket_lib::prelude::*;
use serde::{ Deserialize, Serialize };
use specs::error::NoError;
use specs::prelude::*;
use specs::saveload::{ ConvertSaveload, Marker };
use specs_derive::*;
use std::collections::{ HashMap, HashSet };

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
    pub event_counts: HashMap<String, i32>,
    pub events: HashMap<u32, Vec<String>>,
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
    pub glyph: FontCharType,
    pub fg: RGB,
    pub bg: RGB,
    pub render_order: i32,
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Bleeds {
    pub colour: RGB,
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
    RandomWaypoint {
        path: Option<Vec<usize>>,
    },
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
    pub visible_tiles: Vec<Point>,
    pub range: i32,
    pub dirty: bool,
}

#[derive(Component, ConvertSaveload, Clone)]
pub struct Telepath {
    pub telepath_tiles: Vec<Point>,
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct KnownSpell {
    pub display_name: String,
    pub mana_cost: i32,
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct KnownSpells {
    pub list: Vec<KnownSpell>,
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct GrantsSpell {
    pub spell: String,
}

// TODO: GrantsIntrinsic, Intrinsics, etc. ? Done the same way as spells?

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

#[derive(Component, Debug, Serialize, Deserialize, Eq, PartialEq, Hash, Clone)]
pub struct Beatitude {
    pub buc: BUC,
    pub known: bool,
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Item {
    pub weight: f32, // in lbs
    pub value: f32, // base
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
    pub damage_type: DamageType,
    pub attribute: WeaponAttribute,
    pub damage_n_dice: i32,
    pub damage_die_type: i32,
    pub damage_bonus: i32,
    pub hit_bonus: i32,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct NaturalAttack {
    pub name: String,
    pub damage_type: DamageType,
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

#[derive(Component, ConvertSaveload, Clone)]
pub struct ToHitBonus {
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

#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone, Serialize, Deserialize)]
pub enum DamageType {
    Physical,
    Magic, // e.g. magic missiles, silvered weapons
    Fire, // e.g. fireball
    Cold, // e.g. cone of cold
    Poison, // e.g. poison gas
    Forced, // Bypasses any immunities. e.g. Hunger ticks.
}

impl DamageType {
    pub fn is_magic(&self) -> bool {
        match self {
            DamageType::Magic | DamageType::Fire | DamageType::Cold => true,
            _ => false,
        }
    }
}

#[derive(Debug, PartialEq, Copy, Clone, Serialize, Deserialize)]
pub enum DamageModifier {
    None,
    Weakness,
    Resistance,
    Immune,
}

impl DamageModifier {
    const NONE_MOD: f32 = 1.0;
    const WEAK_MOD: f32 = 2.0;
    const RESIST_MOD: f32 = 0.5;
    const IMMUNE_MOD: f32 = 0.0;

    pub fn multiplier(&self) -> f32 {
        match self {
            DamageModifier::None => Self::NONE_MOD,
            DamageModifier::Weakness => Self::WEAK_MOD,
            DamageModifier::Resistance => Self::RESIST_MOD,
            DamageModifier::Immune => Self::IMMUNE_MOD,
        }
    }
}

#[derive(Component, Serialize, Deserialize, Debug, Clone)]
pub struct HasDamageModifiers {
    pub modifiers: HashMap<DamageType, DamageModifier>,
}

impl HasDamageModifiers {
    pub fn modifier(&self, damage_type: &DamageType) -> &DamageModifier {
        self.modifiers.get(damage_type).unwrap_or(&DamageModifier::None)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Intrinsic {
    Regeneration, // Regenerate 1 HP on every tick
    Speed, // 4/3x speed multiplier
}

impl Intrinsic {
    pub fn describe(&self) -> &str {
        match self {
            Intrinsic::Regeneration => "regenerates health",
            Intrinsic::Speed => "is hasted",
        }
    }
}

#[derive(Component, Serialize, Deserialize, Debug, Clone)]
pub struct Intrinsics {
    pub list: HashSet<Intrinsic>,
}

impl Intrinsics {
    pub fn describe(&self) -> String {
        let mut descriptions = Vec::new();
        for intrinsic in &self.list {
            descriptions.push(intrinsic.describe());
        }
        match descriptions.len() {
            0 =>
                unreachable!("describe() should never be called on an empty Intrinsics component."),
            1 => format!("It {}.", descriptions[0]),
            _ => {
                let last = descriptions.pop().unwrap();
                let joined = descriptions.join(", ");
                format!("It {}, and {}.", joined, last)
            }
        }
    }
}

#[derive(Component, Serialize, Deserialize, Debug, Clone)]
pub struct IntrinsicChanged {
    pub gained: HashSet<Intrinsic>,
    pub lost: HashSet<Intrinsic>,
}

#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct InflictsDamage {
    pub damage_type: DamageType,
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
pub struct Blind {}

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
    pub target: Option<Point>,
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
    pub glyph: FontCharType,
    pub tail_glyph: FontCharType,
    pub colour: RGB,
    pub lifetime_ms: f32,
    pub trail_colour: RGB,
    pub trail_lifetime_ms: f32,
}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct SpawnParticleSimple {
    pub glyph: FontCharType,
    pub colour: RGB,
    pub lifetime_ms: f32,
}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct SpawnParticleBurst {
    pub glyph: FontCharType,
    pub head_glyph: FontCharType,
    pub tail_glyph: FontCharType,
    pub colour: RGB,
    pub lerp: RGB,
    pub lifetime_ms: f32,
    pub trail_colour: RGB,
    pub trail_lifetime_ms: f32,
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Destructible {}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct ProvidesRemoveCurse {}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct ProvidesIdentify {}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Digger {}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Hidden {}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct SingleActivation {}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct IdentifiedBeatitude {}

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
