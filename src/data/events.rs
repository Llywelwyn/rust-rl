use serde::{ Deserialize, Serialize };

#[derive(Serialize, Deserialize, Clone)]
pub enum EVENT {
    TURN(i32),
    LEVEL(i32),
    CHANGED_FLOOR(i32),
    PLAYER_CONFUSED(i32),
    KICKED_SOMETHING(i32),
    BROKE_DOOR(i32),
    LOOKED_FOR_HELP(i32),
    KILLED(String),
    DISCOVERED(String),
    IDENTIFIED(String),
}

impl EVENT {
    pub const COUNT_TURN: &str = "turns";
    pub const COUNT_DEATH: &str = "deaths";
    pub const COUNT_LEVEL: &str = "level";
    pub const COUNT_CHANGED_FLOOR: &str = "changed_floor";
    pub const COUNT_BROKE_DOOR: &str = "broke_door";
    pub const COUNT_PLAYER_CONFUSED: &str = "player_confused";
    pub const COUNT_KICK: &str = "kick";
    pub const COUNT_LOOKED_FOR_HELP: &str = "looked_for_help";
}
