use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize, Debug)]
pub struct Item {
    pub id: String,
    pub name: Name,
    pub renderable: Option<Renderable>,
    pub weight: Option<f32>,
    pub value: Option<f32>,
    pub equip: Option<Equippable>,
    pub flags: Option<Vec<String>>,
    pub effects: Option<HashMap<String, String>>,
    pub magic: Option<MagicItem>,
}

#[derive(Deserialize, Debug)]
pub struct Name {
    pub name: String,
    pub plural: String,
}

#[derive(Deserialize, Debug)]
pub struct Equippable {
    pub flag: String,
    pub damage: String,
    pub to_hit: Option<i32>,
}

#[derive(Deserialize, Debug)]
pub struct Renderable {
    pub glyph: String,
    pub fg: String,
    pub bg: String,
    pub order: i32,
}

#[derive(Deserialize, Debug)]
pub struct MagicItem {
    pub class: String,
    pub naming: String,
}
