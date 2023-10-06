use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize, Debug)]
pub struct Item {
    pub id: String,
    pub name: Name,
    pub renderable: Option<Renderable>,
    pub class: String,
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
    pub sprite: String,
    pub alt: Option<String>,
    pub fg: String,
    pub fg_alt: Option<String>,
    pub order: i32,
    pub order_alt: Option<i32>,
    pub x: Option<f32>,
    pub x_alt: Option<f32>,
    pub y: Option<f32>,
    pub y_alt: Option<f32>,
}

#[derive(Deserialize, Debug)]
pub struct MagicItem {
    pub class: String,
    pub naming: String,
}
