use super::Renderable;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Mob {
    pub id: String,
    pub name: String,
    pub renderable: Option<Renderable>,
    pub flags: Option<Vec<String>>,
    pub stats: MobStats,
    pub vision_range: i32,
}

#[derive(Deserialize, Debug)]
pub struct MobStats {
    pub max_hp: i32,
    pub hp: i32,
    pub power: i32,
    pub defence: i32,
}
