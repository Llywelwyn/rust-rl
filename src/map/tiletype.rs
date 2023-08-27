use serde::{ Deserialize, Serialize };

#[derive(PartialEq, Eq, Hash, Copy, Clone, Serialize, Deserialize)]
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
    ToOvermap,
    ToTown,
}

pub fn tile_walkable(tt: TileType) -> bool {
    match tt {
        TileType::ImpassableMountain | TileType::Wall | TileType::DeepWater | TileType::Fence | TileType::Bars => false,
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

pub fn tile_cost(tt: TileType) -> f32 {
    match tt {
        TileType::Road => 0.5,
        TileType::Grass => 1.1,
        TileType::ShallowWater => 1.3,
        _ => 1.0,
    }
}
