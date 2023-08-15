use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize, Debug)]
pub struct FactionData {
    pub id: String,
    pub responses: HashMap<String, String>,
}

#[derive(PartialEq, Eq, Hash, Copy, Clone)]
pub enum Reaction {
    Ignore,
    Attack,
    Flee,
}
