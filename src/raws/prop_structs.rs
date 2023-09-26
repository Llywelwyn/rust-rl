use super::Renderable;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize, Debug)]
pub struct Prop {
    pub id: String,
    pub name: String,
    pub renderable: Option<Renderable>,
    pub flags: Option<Vec<String>>,
    pub effects: Option<HashMap<String, String>>,
    pub door: Option<Door>,
}

#[derive(Deserialize, Debug)]
pub struct Door {
    pub open: bool,
    pub locked: bool,
    pub blocks_vis: bool,
    pub blocks_move: bool,
}
