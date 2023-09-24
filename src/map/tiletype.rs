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
    pub fn sprite(&self) -> &str {
        match self {
            TileType::ImpassableMountain => "statue_warrior",
            TileType::Wall => "wall_cave_h_a",
            TileType::DeepWater => "water",
            TileType::Fence => "wall_cave_h_a",
            TileType::Bars => "wall_cave_h_a",
            TileType::Floor => "floor_cobble_a",
            TileType::WoodFloor => "floor_wood_a",
            TileType::Gravel => "floor_cobble_b",
            TileType::Road => "floor_cobble_c",
            TileType::Grass => "floor_grass_a",
            TileType::Foliage => "floor_grass_b",
            TileType::HeavyFoliage => "floor_grass_c",
            TileType::Sand => "floor_cobble_c",
            TileType::ShallowWater => "water",
            TileType::Bridge => "floor_cobble_a",
            TileType::DownStair => "wall_cave_stair_down",
            TileType::UpStair => "wall_cave_stair_up",
            TileType::ToLocal(_) => "wall_crypt_stair_down",
            TileType::ToOvermap(_) => "wall_crypt_stair_up",
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
