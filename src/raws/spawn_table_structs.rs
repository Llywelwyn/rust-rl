use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct SpawnTable {
    pub id: String,
    pub table: Vec<SpawnTableEntry>,
}

#[derive(Deserialize, Debug)]
pub struct SpawnTableEntry {
    pub id: String,
    pub weight: i32,
    pub min: i32,
    pub max: i32,
}
