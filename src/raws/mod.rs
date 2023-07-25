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
}

rltk::embedded_resource!(RAW_ITEMS, "../../raws/items.json");
rltk::embedded_resource!(RAW_MOBS, "../../raws/mobs.json");
rltk::embedded_resource!(RAW_PROPS, "../../raws/props.json");
rltk::embedded_resource!(RAW_SPAWN_TABLES, "../../raws/spawn_tables.json");

pub fn load_raws() {
    rltk::link_resource!(RAW_ITEMS, "../../raws/items.json");
    rltk::link_resource!(RAW_MOBS, "../../raws/mobs.json");
    rltk::link_resource!(RAW_PROPS, "../../raws/props.json");
    rltk::link_resource!(RAW_SPAWN_TABLES, "../../raws/spawn_tables.json");

    let decoded_raws = get_decoded_raws();
    RAWS.lock().unwrap().load(decoded_raws);
}

pub fn get_decoded_raws() -> Raws {
    // Get items from file
    let mut raw_data = rltk::embedding::EMBED.lock().get_resource("../../raws/items.json".to_string()).unwrap();
    let mut raw_string = std::str::from_utf8(&raw_data).expect("Unable to convert to a valid UTF-8 string.");
    let items: Vec<Item> = serde_json::from_str(&raw_string).expect("Unable to parse items.json");
    // Get mobs from file
    raw_data = rltk::embedding::EMBED.lock().get_resource("../../raws/mobs.json".to_string()).unwrap();
    raw_string = std::str::from_utf8(&raw_data).expect("Unable to convert to a valid UTF-8 string.");
    let mobs: Vec<Mob> = serde_json::from_str(&raw_string).expect("Unable to parse mobs.json");
    // Get props from file
    raw_data = rltk::embedding::EMBED.lock().get_resource("../../raws/props.json".to_string()).unwrap();
    raw_string = std::str::from_utf8(&raw_data).expect("Unable to convert to a valid UTF-8 string.");
    let props: Vec<Prop> = serde_json::from_str(&raw_string).expect("Unable to parse props.json");
    // Get spawntables from file
    raw_data = rltk::embedding::EMBED.lock().get_resource("../../raws/spawn_tables.json".to_string()).unwrap();
    raw_string = std::str::from_utf8(&raw_data).expect("Unable to convert to a valid UTF-8 string.");
    let spawn_tables: Vec<SpawnTable> = serde_json::from_str(&raw_string).expect("Unable to parse spawn_tables.json");

    // Create combined raws
    let raws = Raws { items, mobs, props, spawn_tables };
    return raws;
}
