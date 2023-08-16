use serde::{Deserialize, Serialize};

#[derive(PartialEq, Eq, Hash, Copy, Clone, Serialize, Deserialize)]
pub enum TileType {
    // Walls (opaque)
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
}

pub fn tile_walkable(tt: TileType) -> bool {
    match tt {
        TileType::Floor
        | TileType::WoodFloor
        | TileType::Gravel
        | TileType::Road
        | TileType::Grass
        | TileType::Foliage
        | TileType::HeavyFoliage
        | TileType::Sand
        | TileType::ShallowWater
        | TileType::Bridge
        | TileType::DownStair
        | TileType::UpStair => true,
        _ => false,
    }
}

pub fn tile_opaque(tt: TileType) -> bool {
    match tt {
        TileType::Wall => true,
        _ => false,
    }
}

pub fn tile_cost(tt: TileType) -> f32 {
    match tt {
        TileType::Road => 0.5,
        TileType::Grass => 1.1,
        TileType::ShallowWater => 1.3,
        _ => 1.0,
    }
}
