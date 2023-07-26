use serde::{Deserialize, Serialize};

#[derive(PartialEq, Eq, Hash, Copy, Clone, Serialize, Deserialize)]
pub enum TileType {
    // Walls (opaque)
    Wall,
    // Impassable (transparent)
    DeepWater,
    Fence,
    // Floors (walkable)
    Floor,
    WoodFloor,
    Gravel,
    Road,
    Grass,
    Sand,
    ShallowWater,
    Bridge,
    // Stairs (changes floor)
    DownStair,
}

pub fn tile_walkable(tt: TileType) -> bool {
    match tt {
        TileType::Floor
        | TileType::WoodFloor
        | TileType::Gravel
        | TileType::Road
        | TileType::Grass
        | TileType::Sand
        | TileType::ShallowWater
        | TileType::Bridge
        | TileType::DownStair => true,
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
        TileType::Road => 0.8,
        TileType::Grass => 1.1,
        TileType::ShallowWater => 1.2,
        _ => 1.0,
    }
}
