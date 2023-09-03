// POST-PROCESSING
pub const WITH_DARKEN_BY_DISTANCE: bool = true; // If further away tiles should get darkened, instead of a harsh transition to non-visible.
pub const VIEWPORT_W: i32 = 40;
pub const VIEWPORT_H: i32 = 30;

pub const BRIGHTEN_FG_COLOUR_BY: i32 = 16;
pub const GLOBAL_OFFSET_MIN_CLAMP: f32 = -0.5;
pub const GLOBAL_OFFSET_MAX_CLAMP: f32 = 1.0;
pub const WITH_SCANLINES_BRIGHTEN_AMOUNT: f32 = 0.1; // 0.0 = no brightening, 1.0 = full brightening.
pub const NON_VISIBLE_MULTIPLIER: f32 = 0.3; // 0.0 = black, 1.0 = full colour.
pub const NON_VISIBLE_MULTIPLIER_IF_SCANLINES: f32 = 0.8; // as above, but when using scanlines. should be higher.
pub const MAX_DARKENING: f32 = 0.45; // 0.0 = black, 1.0 = full colour - only used if WITH_DARKEN_BY_DISTANCE is true.
pub const MAX_DARKENING_IF_SCANLINES: f32 = 0.9; // as above, but when using scanlines. should be higher.
pub const START_DARKEN_AT_N_TILES: f32 = 8.0; // start darkening at this distance (should always be less than entity::DEFAULT_VIEWSHED_STANDARD).

pub const SHORT_PARTICLE_LIFETIME: f32 = 100.0; // in ms
pub const DEFAULT_PARTICLE_LIFETIME: f32 = 200.0;
pub const LONG_PARTICLE_LIFETIME: f32 = 300.0;

pub const TARGETING_CURSOR_COL: (u8, u8, u8) = rltk::GOLDENROD;
pub const TARGETING_LINE_COL: (u8, u8, u8) = rltk::LIGHTGOLDENROD;
pub const TARGETING_AOE_COL: (u8, u8, u8) = (20, 20, 20);
pub const TARGETING_VALID_COL: (u8, u8, u8) = (10, 10, 10);

// THEMES
pub const BLOODSTAIN_COLOUR: (u8, u8, u8) = (153, 0, 0);
// DEFAULT THEME
pub const DEFAULT_BG_COLOUR: (u8, u8, u8) = (29, 50, 50);
pub const DEFAULT_BG_OFFSETS: (i32, i32, i32) = (10, 10, 10);
pub const WALL_COLOUR: (u8, u8, u8) = (229, 191, 94);
pub const WALL_OFFSETS: (i32, i32, i32) = (48, 48, 48);
pub const FLOOR_COLOUR: (u8, u8, u8) = (25, 204, 122);
pub const FLOOR_OFFSETS: (i32, i32, i32) = (10, 10, 10);
pub const STAIR_COLOUR: (u8, u8, u8) = (200, 200, 0);
pub const STAIR_OFFSETS: (i32, i32, i32) = (10, 10, 10);
pub const WOOD_FLOOR_COLOUR: (u8, u8, u8) = (41, 30, 20);
pub const WOOD_FLOOR_OFFSETS: (i32, i32, i32) = (10, 10, 10);
pub const FENCE_FG_COLOUR: (u8, u8, u8) = (110, 24, 0);
pub const FENCE_COLOUR: (u8, u8, u8) = (45, 30, 10);
pub const FENCE_OFFSETS: (i32, i32, i32) = (10, 10, 10);
pub const BRIDGE_COLOUR: (u8, u8, u8) = (42, 48, 37);
pub const BRIDGE_OFFSETS: (i32, i32, i32) = (10, 10, 10);
pub const GRAVEL_COLOUR: (u8, u8, u8) = (26, 26, 53);
pub const GRAVEL_OFFSETS: (i32, i32, i32) = (10, 10, 10);
pub const ROAD_COLOUR: (u8, u8, u8) = (8, 38, 40);
pub const ROAD_OFFSETS: (i32, i32, i32) = (10, 10, 10);
pub const GRASS_COLOUR: (u8, u8, u8) = (9, 65, 6);
pub const GRASS_OFFSETS: (i32, i32, i32) = (3, 20, 10);
pub const FOLIAGE_COLOUR: (u8, u8, u8) = (5, 60, 5);
pub const FOLIAGE_OFFSETS: (i32, i32, i32) = (10, 10, 10);
pub const HEAVY_FOLIAGE_COLOUR: (u8, u8, u8) = (5, 60, 5);
pub const HEAVY_FOLIAGE_OFFSETS: (i32, i32, i32) = (10, 10, 10);
pub const SAND_COLOUR: (u8, u8, u8) = (70, 70, 21);
pub const SAND_OFFSETS: (i32, i32, i32) = (10, 10, 10);
pub const SHALLOW_WATER_COLOUR: (u8, u8, u8) = (24, 47, 99);
pub const SHALLOW_WATER_OFFSETS: (i32, i32, i32) = (3, 10, 45);
pub const DEEP_WATER_COLOUR: (u8, u8, u8) = (18, 33, 63);
pub const DEEP_WATER_OFFSETS: (i32, i32, i32) = (5, 10, 32);
pub const BARS_COLOUR: (u8, u8, u8) = (100, 100, 100);
pub const IMPASSABLE_MOUNTAIN_COLOUR: (u8, u8, u8) = (20, 23, 20);
pub const IMPASSABLE_MOUNTAIN_OFFSETS: (i32, i32, i32) = (4, 4, 4);
// FOREST THEME
pub const FOREST_WALL_COLOUR: (u8, u8, u8) = (0, 153, 0);

// DEFAULT THEME
#[allow(dead_code)]
pub const WALL_GLYPH: char = '#';
pub const FLOOR_GLYPH: char = '.';
pub const DOWN_STAIR_GLYPH: char = '>';
pub const UP_STAIR_GLYPH: char = '<';
pub const WOOD_FLOOR_GLYPH: char = '.';
pub const FENCE_GLYPH: char = '=';
pub const BRIDGE_GLYPH: char = '.';
pub const GRAVEL_GLYPH: char = ';';
pub const ROAD_GLYPH: char = '.';
pub const GRASS_GLYPH: char = '"';
pub const FOLIAGE_GLYPH: char = ':';
pub const HEAVY_FOLIAGE_GLYPH: char = ';';
pub const SAND_GLYPH: char = '.';
pub const SHALLOW_WATER_GLYPH: char = '~';
pub const DEEP_WATER_GLYPH: char = '≈';
pub const BARS_GLYPH: char = '#';
pub const IMPASSABLE_MOUNTAIN_GLYPH: char = '▲';

// FOREST THEME
pub const FOREST_WALL_GLYPH: char = '♣';

// Overmap/transition stuff
pub const TO_OVERMAP_GLYPH: char = '<';
pub const TO_OVERMAP_COLOUR: (u8, u8, u8) = (205, 127, 50);
pub const TO_TOWN_GLYPH: char = 'o';
pub const TO_TOWN_COLOUR: (u8, u8, u8) = (205, 127, 50);
