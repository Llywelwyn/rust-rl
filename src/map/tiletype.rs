use serde::{ Deserialize, Serialize };

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
    pub fn sprite(&self, base: bool, float: f32) -> &str {
        if base {
            return self.h(float);
        }
        return self.v(float);
    }
    fn h(&self, float: f32) -> &str {
        let options = match self {
            TileType::Wall => vec!["wall_b"],
            _ => unreachable!("Tried to get a h (base) sprite for a non-wall tile."),
        };
        return options[(float * (options.len() as f32)) as usize];
    }
    fn v(&self, float: f32) -> &str {
        let options = match self {
            TileType::ImpassableMountain => vec!["wall_b"],
            TileType::Wall => vec!["wall_top"],
            TileType::DeepWater => vec!["water", "water2"],
            TileType::Fence => vec!["wall_b"],
            TileType::Bars => vec!["wall_b"],
            TileType::Floor => vec!["fluff", "fluff2"],
            TileType::WoodFloor => vec!["fluff", "fluff2"],
            TileType::Gravel => vec!["fluff", "fluff2"],
            TileType::Road => vec!["fluff", "fluff2"],
            TileType::Grass => vec!["fluff", "fluff2"],
            TileType::Foliage => vec!["fluff", "fluff2"],
            TileType::HeavyFoliage => vec!["fluff", "fluff2"],
            TileType::Sand => vec!["fluff", "fluff2"],
            TileType::ShallowWater => vec!["water", "water2"],
            TileType::Bridge => vec!["wall_b"],
            TileType::DownStair => vec!["wall_b"],
            TileType::UpStair => vec!["wall_b"],
            TileType::ToLocal(_) => vec!["wall_b"],
            TileType::ToOvermap(_) => vec!["wall_b"],
        };
        return options[(float * (options.len() as f32)) as usize];
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
