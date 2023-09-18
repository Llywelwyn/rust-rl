use serde::{ Deserialize, Serialize };

#[derive(Serialize, Deserialize, Clone)]
pub enum EVENT {
    Turn(i32),
    Level(i32),
    ChangedFloor(String),
    PlayerConfused(i32),
    KickedSomething(i32),
    BrokeDoor(i32),
    LookedForHelp(i32),
    Killed(String),
    PlayerDied(String),
    Discovered(String),
    Identified(String),
}

impl EVENT {
    pub const COUNT_TURN: &str = "turns";
    pub const COUNT_KILLED: &str = "killed";
    pub const COUNT_LEVEL: &str = "level";
    pub const COUNT_CHANGED_FLOOR: &str = "changed_floor";
    pub const COUNT_BROKE_DOOR: &str = "BrokeDoor";
    pub const COUNT_PLAYER_CONFUSED: &str = "PlayerConfused";
    pub const COUNT_KICK: &str = "kick";
    pub const COUNT_LOOKED_FOR_HELP: &str = "LookedForHelp";
}
