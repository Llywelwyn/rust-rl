use serde::{ Deserialize, Serialize };
use crate::data::sprites::*;

#[derive(PartialEq, Eq, Hash, Copy, Clone, Serialize, Deserialize, Debug)]
pub enum TileType {
    // Walls (opaque)
    ImpassableMountain,
    Wall,
    // Impassable (transparent)
    DeepWater,
    Fence,
    Bars,
    // Floors (walkable)
    Floor,
    WoodFloor,
    Gravel,
    Road,
    Grass,
    Foliage,
    HeavyFoliage,
    Sand,
    ShallowWater,
    Bridge,
    // Stairs (changes floor)
    DownStair,
    UpStair,
    // To/From Overmap - ids are in src/data/ids.rs, are used in try_change_level() in src/player.rs
    ToOvermap(i32),
    ToLocal(i32),
}

impl TileType {
    pub fn sprite(&self) -> usize {
        match self {
            TileType::ImpassableMountain => STATUE,
            TileType::Wall => WALL_BASE,
            TileType::DeepWater => WATER_DEEP,
            TileType::Fence => WALL_BASE,
            TileType::Bars => WALL_BASE,
            TileType::Floor => FLOOR,
            TileType::WoodFloor => FLOOR_WOOD,
            TileType::Gravel => FLOOR,
            TileType::Road => PATH_GRASS,
            TileType::Grass => FLOOR_GRASS,
            TileType::Foliage => FLOOR_GRASS,
            TileType::HeavyFoliage => FLOOR_GRASS,
            TileType::Sand => FLOOR,
            TileType::ShallowWater => WATER_DEEP,
            TileType::Bridge => FLOOR,
            TileType::DownStair => STAIR_D,
            TileType::UpStair => STAIR_A,
            TileType::ToLocal(_) => MUSHROOM,
            TileType::ToOvermap(_) => MUSHROOM_ORANGE,
        }
    }

    pub fn variants(&self) -> usize {
        match self {
            TileType::ImpassableMountain => 1,
            TileType::Wall => 4,
            TileType::DeepWater => 2,
            TileType::Fence => 1,
            TileType::Bars => 1,
            TileType::Floor => 6,
            TileType::WoodFloor => 3,
            TileType::Gravel => 1,
            TileType::Road => 4,
            TileType::Grass => 6,
            TileType::Foliage => 1,
            TileType::HeavyFoliage => 1,
            TileType::Sand => 1,
            TileType::ShallowWater => 2,
            TileType::Bridge => 1,
            TileType::DownStair => 1,
            TileType::UpStair => 1,
            TileType::ToLocal(_) => 1,
            TileType::ToOvermap(_) => 1,
        }
    }
}

pub fn tile_walkable(tt: TileType) -> bool {
    match tt {
        | TileType::ImpassableMountain
        | TileType::Wall
        | TileType::DeepWater
        | TileType::Fence
        | TileType::Bars => false,
        _ => true,
    }
}
pub fn tile_opaque(tt: TileType) -> bool {
    match tt {
        TileType::ImpassableMountain => true,
        TileType::Wall => true,
        _ => false,
    }
}
pub fn tile_blocks_telepathy(tt: TileType) -> bool {
    match tt {
        _ => false,
    }
}
pub fn tile_cost(tt: TileType) -> f32 {
    match tt {
        TileType::Road => 0.75,
        TileType::Grass => 1.2,
        TileType::ShallowWater => 1.5,
        _ => 1.0,
    }
}
pub fn get_dest(this_tile: TileType, backtracking: bool) -> Destination {
    let result = if !backtracking {
        match this_tile {
            // If on downstair, GOTO next level, and end up on an upstair
            TileType::DownStair => Destination::NextLevel,
            // If on overmap ToLocal tile, GOTO local map, and end up on an overmap ToOvermap tile with corresponding ID
            TileType::ToLocal(id) => Destination::ToOvermap(id),
            _ => Destination::None,
        }
    } else {
        match this_tile {
            TileType::UpStair => Destination::PreviousLevel,
            TileType::ToOvermap(id) => Destination::ToLocal(id),
            _ => Destination::None,
        }
    };
    return result;
}

pub enum Destination {
    PreviousLevel,
    NextLevel,
    ToOvermap(i32),
    ToLocal(i32),
    None,
}
