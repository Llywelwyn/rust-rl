use serde::{ Deserialize, Serialize };
use bracket_lib::random::RandomNumberGenerator;

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
            TileType::Wall =>
                vec![
                    "wall_cave_h_a",
                    "wall_cave_h_b",
                    "wall_cave_h_c",
                    "wall_cave_h_d",
                    "wall_cave_h_crack"
                ],
            _ => unreachable!("Tried to get a h (base) sprite for a non-wall tile."),
        };
        return options[(float * (options.len() as f32)) as usize];
    }
    fn v(&self, float: f32) -> &str {
        let options = match self {
            TileType::ImpassableMountain => vec!["statue_warrior"],
            TileType::Wall =>
                vec![
                    "wall_cave_v_a",
                    "wall_cave_v_b",
                    "wall_cave_v_c",
                    "wall_cave_v_d",
                    "wall_cave_v_crack"
                ],
            TileType::DeepWater => vec!["water", "water_a1", "water_a2"],
            TileType::Fence => vec!["wall_cave_h_a"],
            TileType::Bars => vec!["wall_cave_h_a"],
            TileType::Floor =>
                vec![
                    "floor_cobble_a",
                    "floor_cobble_b",
                    "floor_cobble_c",
                    "floor_cobble_d",
                    "floor_cobble_e",
                    "floor_cobble_f"
                ],
            TileType::WoodFloor =>
                vec!["floor_wood_a", "floor_wood_b", "floor_wood_c", "floor_wood_d"],
            TileType::Gravel => vec!["floor_cobble_b"],
            TileType::Road =>
                vec![
                    "floor_tile_a",
                    "floor_tile_b",
                    "floor_tile_c",
                    "floor_tile_d",
                    "floor_mossy_a",
                    "floor_mossy_b",
                    "floor_mossy_c",
                    "floor_mossy_d",
                    "floor_mossy_e"
                ],
            TileType::Grass =>
                vec![
                    "floor_grass_a",
                    "floor_grass_b",
                    "floor_grass_c",
                    "floor_grass_d",
                    "floor_grass_e",
                    "floor_grass_f"
                ],
            TileType::Foliage => vec!["floor_grass_b"],
            TileType::HeavyFoliage => vec!["floor_grass_c"],
            TileType::Sand => vec!["floor_cobble_c"],
            TileType::ShallowWater => vec!["water"],
            TileType::Bridge => vec!["floor_cobble_a"],
            TileType::DownStair => vec!["wall_cave_stair_down"],
            TileType::UpStair => vec!["wall_cave_stair_up"],
            TileType::ToLocal(_) => vec!["wall_crypt_stair_down"],
            TileType::ToOvermap(_) => vec!["wall_crypt_stair_up"],
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
