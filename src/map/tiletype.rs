use serde::{ Deserialize, Serialize };
use bracket_lib::prelude::*;
use crate::consts::visuals::*;

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
    pub fn sprite(&self, base: bool, float: f32, bloody: Option<RGB>) -> &str {
        if base {
            return self.h(float, bloody);
        }
        return self.v(float, bloody);
    }
    fn h(&self, float: f32, _bloody: Option<RGB>) -> &str {
        let options = match self {
            TileType::Wall => vec!["wall_b", "wall_b_cracked"],
            _ => unreachable!("Tried to get a h (base) sprite for a non-wall tile."),
        };
        return options[(float * (options.len() as f32)) as usize];
    }
    fn v(&self, float: f32, bloody: Option<RGB>) -> &str {
        let mut options = match self {
            TileType::ImpassableMountain => vec!["wall_b"],
            TileType::Wall => vec!["wall"],
            TileType::DeepWater => vec!["water", "water2"],
            TileType::Fence => vec!["tiles4"],
            TileType::Bars => vec!["wall_b"],
            TileType::Floor => vec!["dot"],
            TileType::WoodFloor => vec!["planks", "planks_missing", "planks_missing2"],
            TileType::Gravel => vec!["fluff", "fluff2"],
            TileType::Road =>
                vec![
                    "tiles",
                    "tiles_missing",
                    "tiles_missing2",
                    "tiles_missing3",
                    "tiles_missing4",
                    "tiles_missing5",
                    "tiles_missing6"
                ],
            TileType::Grass => vec!["fluff", "fluff2"],
            TileType::Foliage => vec!["grass_small", "grass"],
            TileType::HeavyFoliage => vec!["grass_flower"],
            TileType::Sand => vec!["fluff", "fluff2"],
            TileType::ShallowWater => vec!["water", "water2"],
            TileType::Bridge => vec!["planks"],
            TileType::DownStair => vec!["wall_b"],
            TileType::UpStair => vec!["wall_b"],
            TileType::ToLocal(_) => vec!["wall_b"],
            TileType::ToOvermap(_) => vec!["wall_b"],
        };
        if bloody.is_some() && tile_walkable(*self) {
            options.extend(
                vec!["blood1", "blood2", "blood3", "blood4", "blood5", "blood6", "blood7"]
            );
        }
        return options[(float * (options.len() as f32)) as usize];
    }
    pub fn offset(&self) -> (i32, i32, i32) {
        match self {
            TileType::ImpassableMountain => IMPASSABLE_MOUNTAIN_OFFSETS,
            TileType::Wall => WALL_OFFSETS,
            TileType::DeepWater => DEEP_WATER_OFFSETS,
            TileType::Fence => FENCE_OFFSETS,
            TileType::Bars => BARS_OFFSETS,
            TileType::Floor => FLOOR_OFFSETS,
            TileType::WoodFloor => WOOD_FLOOR_OFFSETS,
            TileType::Gravel => GRAVEL_OFFSETS,
            TileType::Road => ROAD_OFFSETS,
            TileType::Grass => GRASS_OFFSETS,
            TileType::Foliage => FOLIAGE_OFFSETS,
            TileType::HeavyFoliage => HEAVY_FOLIAGE_OFFSETS,
            TileType::Sand => SAND_OFFSETS,
            TileType::ShallowWater => SHALLOW_WATER_OFFSETS,
            TileType::Bridge => BRIDGE_OFFSETS,
            TileType::DownStair => STAIR_OFFSETS,
            TileType::UpStair => STAIR_OFFSETS,
            TileType::ToLocal(_) => WALL_OFFSETS,
            TileType::ToOvermap(_) => WALL_OFFSETS,
        }
    }
    pub fn col(&self, bloody: Option<RGB>) -> RGB {
        if let Some(bloody) = bloody {
            return bloody;
        }
        RGB::named(match self {
            TileType::ImpassableMountain => IMPASSABLE_MOUNTAIN_COLOUR,
            TileType::Wall => WALL_COLOUR,
            TileType::DeepWater => DEEP_WATER_COLOUR,
            TileType::Fence => FENCE_COLOUR,
            TileType::Bars => BARS_COLOUR,
            TileType::Floor => FLOOR_COLOUR,
            TileType::WoodFloor => WOOD_FLOOR_COLOUR,
            TileType::Gravel => GRAVEL_COLOUR,
            TileType::Road => ROAD_COLOUR,
            TileType::Grass => GRASS_COLOUR,
            TileType::Foliage => FOLIAGE_COLOUR,
            TileType::HeavyFoliage => HEAVY_FOLIAGE_COLOUR,
            TileType::Sand => SAND_COLOUR,
            TileType::ShallowWater => SHALLOW_WATER_COLOUR,
            TileType::Bridge => BRIDGE_COLOUR,
            TileType::DownStair => STAIR_COLOUR,
            TileType::UpStair => STAIR_COLOUR,
            TileType::ToLocal(_) => WALL_COLOUR,
            TileType::ToOvermap(_) => WALL_COLOUR,
        })
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
