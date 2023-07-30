use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct LootTable {
    pub id: String,
    pub table: Vec<LootTableEntry>,
}

#[derive(Deserialize, Debug)]
pub struct LootTableEntry {
    pub id: String,
    pub weight: i32,
}
