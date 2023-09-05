use serde::Deserialize;
mod rawmaster;
pub use rawmaster::*;
mod item_structs;
use item_structs::*;
mod mob_structs;
use mob_structs::*;
mod prop_structs;
use prop_structs::Prop;
mod spawn_table_structs;
use spawn_table_structs::*;
mod loot_table_structs;
use loot_table_structs::*;
mod reaction_structs;
pub use reaction_structs::Reaction;
use reaction_structs::{ AncestryData, FactionData };
use std::sync::Mutex;
use bracket_lib::prelude::*;

lazy_static! {
    pub static ref RAWS: Mutex<RawMaster> = Mutex::new(RawMaster::empty());
}

#[derive(Deserialize, Debug)]
pub struct Raws {
    pub items: Vec<Item>,
    pub mobs: Vec<Mob>,
    pub props: Vec<Prop>,
    pub spawn_tables: Vec<SpawnTable>,
    pub loot_tables: Vec<LootTable>,
    pub factions: Vec<FactionData>,
    pub ancestries: Vec<AncestryData>,
}

embedded_resource!(RAW_ITEMS, "../../raws/items.json");
embedded_resource!(RAW_MOBS, "../../raws/mobs.json");
embedded_resource!(RAW_PROPS, "../../raws/props.json");
embedded_resource!(RAW_SPAWN_TABLES, "../../raws/spawn_tables.json");
embedded_resource!(RAW_LOOT_TABLES, "../../raws/loot_tables.json");
embedded_resource!(RAW_FACTIONS, "../../raws/factions.json");
embedded_resource!(RAW_ANCESTRIES, "../../raws/ancestries.json");

pub fn load_raws() {
    link_resource!(RAW_ITEMS, "../../raws/items.json");
    link_resource!(RAW_MOBS, "../../raws/mobs.json");
    link_resource!(RAW_PROPS, "../../raws/props.json");
    link_resource!(RAW_SPAWN_TABLES, "../../raws/spawn_tables.json");
    link_resource!(RAW_LOOT_TABLES, "../../raws/loot_tables.json");
    link_resource!(RAW_FACTIONS, "../../raws/factions.json");
    link_resource!(RAW_ANCESTRIES, "../../raws/ancestries.json");

    let decoded_raws = get_decoded_raws();
    RAWS.lock().unwrap().load(decoded_raws);
}

pub fn get_decoded_raws() -> Raws {
    let items: Vec<Item> = ParseJson::parse_raws_into_vector("../../raws/items.json".to_string());
    let mobs: Vec<Mob> = ParseJson::parse_raws_into_vector("../../raws/mobs.json".to_string());
    let props: Vec<Prop> = ParseJson::parse_raws_into_vector("../../raws/props.json".to_string());
    let spawn_tables: Vec<SpawnTable> = ParseJson::parse_raws_into_vector(
        "../../raws/spawn_tables.json".to_string()
    );
    let loot_tables: Vec<LootTable> = ParseJson::parse_raws_into_vector(
        "../../raws/loot_tables.json".to_string()
    );
    let factions: Vec<FactionData> = ParseJson::parse_raws_into_vector(
        "../../raws/factions.json".to_string()
    );
    let ancestries: Vec<AncestryData> = ParseJson::parse_raws_into_vector(
        "../../raws/ancestries.json".to_string()
    );

    return Raws { items, mobs, props, spawn_tables, loot_tables, factions, ancestries };
}

trait ParseJson {
    fn parse_raws_into_vector(path: String) -> Self;
}
macro_rules! impl_ParseJson {
    (for $($t:ty),+) => {
        $(impl ParseJson for $t {
            fn parse_raws_into_vector(path: String) -> $t {
                let raw_data = EMBED.lock().get_resource(path).unwrap();
                let raw_string = std::str::from_utf8(&raw_data).expect("Failed to convert UTF-8 to &str.");
                return serde_json::from_str(&raw_string).expect("Failed to convert &str to json");
            }
        })*
    };
}
impl_ParseJson!(for Vec<Item>, Vec<Mob>, Vec<Prop>, Vec<SpawnTable>, Vec<LootTable>, Vec<FactionData>, Vec<AncestryData>);
