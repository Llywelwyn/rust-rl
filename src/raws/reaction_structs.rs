use serde::Deserialize;
use std::collections::{ HashMap, HashSet };

#[derive(Deserialize, Debug)]
pub struct FactionData {
    pub id: String,
    pub responses: HashMap<String, String>,
}

#[derive(Deserialize, Debug)]
pub struct AncestryData {
    pub id: String,
    pub allies: HashSet<String>,
}

#[derive(PartialEq, Eq, Hash, Copy, Clone)]
pub enum Reaction {
    Ignore,
    Attack,
    Flee,
}
