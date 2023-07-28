use super::Renderable;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize, Debug)]
pub struct Mob {
    pub id: String,
    pub name: String,
    pub renderable: Option<Renderable>,
    pub flags: Option<Vec<String>>,
    pub level: Option<i32>,
    pub bac: Option<i32>,
    pub attacks: Option<Vec<NaturalAttack>>,
    pub attributes: Option<MobAttributes>,
    pub skills: Option<HashMap<String, i32>>,
    pub vision_range: i32,
    pub quips: Option<Vec<String>>,
}

#[derive(Deserialize, Debug)]
pub struct MobAttributes {
    pub str: Option<i32>,
    pub dex: Option<i32>,
    pub con: Option<i32>,
    pub int: Option<i32>,
    pub wis: Option<i32>,
    pub cha: Option<i32>,
}

#[derive(Deserialize, Debug)]
pub struct NaturalAttack {
    pub name: String,
    pub hit_bonus: i32,
    pub damage: String,
}
