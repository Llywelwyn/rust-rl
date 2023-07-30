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
use std::sync::Mutex;

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
}

rltk::embedded_resource!(RAW_ITEMS, "../../raws/items.json");
rltk::embedded_resource!(RAW_MOBS, "../../raws/mobs.json");
rltk::embedded_resource!(RAW_PROPS, "../../raws/props.json");
rltk::embedded_resource!(RAW_SPAWN_TABLES, "../../raws/spawn_tables.json");
rltk::embedded_resource!(RAW_LOOT_TABLES, "../../raws/loot_tables.json");

pub fn load_raws() {
    rltk::link_resource!(RAW_ITEMS, "../../raws/items.json");
    rltk::link_resource!(RAW_MOBS, "../../raws/mobs.json");
    rltk::link_resource!(RAW_PROPS, "../../raws/props.json");
    rltk::link_resource!(RAW_SPAWN_TABLES, "../../raws/spawn_tables.json");
    rltk::link_resource!(RAW_LOOT_TABLES, "../../raws/loot_tables.json");

    let decoded_raws = get_decoded_raws();
    RAWS.lock().unwrap().load(decoded_raws);
}

pub fn get_decoded_raws() -> Raws {
    let items: Vec<Item> = ParseJson::parse_raws_into_vector("../../raws/items.json".to_string());
    let mobs: Vec<Mob> = ParseJson::parse_raws_into_vector("../../raws/mobs.json".to_string());
    let props: Vec<Prop> = ParseJson::parse_raws_into_vector("../../raws/props.json".to_string());
    let spawn_tables: Vec<SpawnTable> = ParseJson::parse_raws_into_vector("../../raws/spawn_tables.json".to_string());
    let loot_tables: Vec<LootTable> = ParseJson::parse_raws_into_vector("../../raws/loot_tables.json".to_string());

    return Raws { items, mobs, props, spawn_tables, loot_tables };
}

trait ParseJson {
    fn parse_raws_into_vector(path: String) -> Self;
}
macro_rules! impl_ParseJson {
    (for $($t:ty),+) => {
        $(impl ParseJson for $t {
            fn parse_raws_into_vector(path: String) -> $t {
                let raw_data = rltk::embedding::EMBED.lock().get_resource(path).unwrap();
                let raw_string = std::str::from_utf8(&raw_data).expect("Unable to convert to a valid UTF-8 string.");
                return serde_json::from_str(&raw_string).expect("Unable to parse items.json");
            }
        })*
    }
}
impl_ParseJson!(for Vec<Item>, Vec<Mob>, Vec<Prop>, Vec<SpawnTable>, Vec<LootTable>);
