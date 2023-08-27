use super::names::*;
use super::visuals::*;
use rltk::prelude::*;

pub const ID_OVERMAP: i32 = 1;

pub const ID_TOWN: i32 = 10;
pub const ID_TOWN2: i32 = ID_TOWN + 1;
pub const ID_TOWN3: i32 = ID_TOWN + 2;

pub const ID_INFINITE: i32 = 1000;

pub fn get_local_desc(id: i32) -> String {
    let str = match id {
        ID_TOWN => NAME_STARTER_TOWN,
        ID_INFINITE => NAME_DUNGEON_RANDOM,
        _ => "an unnamed overmap tile",
    };
    return str.to_string();
}

pub fn get_local_col(id: i32) -> RGB {
    let col = match id {
        ID_TOWN => TO_TOWN_COLOUR,
        ID_OVERMAP => TO_OVERMAP_COLOUR,
        _ => (255, 255, 255),
    };
    return RGB::from_u8(col.0, col.1, col.2);
}

pub fn rgb_to_u8(col: RGB) -> (u8, u8, u8) {
    return ((col.r * 255.0) as u8, (col.g * 255.0) as u8, (col.b * 255.0) as u8);
}
