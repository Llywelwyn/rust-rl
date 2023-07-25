use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize, Debug)]
pub struct Item {
    pub id: String,
    pub name: Name,
    pub renderable: Option<Renderable>,
    pub flags: Option<Vec<String>>,
    pub effects: Option<HashMap<String, String>>,
}

#[derive(Deserialize, Debug)]
pub struct Name {
    pub name: String,
    pub plural: String,
}

#[derive(Deserialize, Debug)]
pub struct Renderable {
    pub glyph: String,
    pub fg: String,
    pub bg: String,
    pub order: i32,
}
