// tests/map_test.rs
use rust_rl::map::*;
use std::collections::HashSet;

#[test]
fn map_settings() {
    let (overmap, id, width, height, difficulty, name, short_name, depth) = (
        false,
        0,
        80,
        50,
        0,
        "Test Map",
        "Test Map",
        0,
    );
    let map = Map::new(overmap, id, width, height, difficulty, name, short_name, depth);

    assert_eq!(map.overmap, overmap);
    assert_eq!(map.id, id);
    assert_eq!(map.width, width);
    assert_eq!(map.height, height);
    assert_eq!(map.difficulty, difficulty);
    assert_eq!(map.name, name);
    assert_eq!(map.short_name, short_name);
    assert_eq!(map.depth, depth);
    assert_eq!(map.tiles.len(), (width * height) as usize);
    assert_eq!(map.messages, HashSet::new());
}

#[test]
fn tiletype_equality() {
    let tile1 = TileType::ImpassableMountain;
    let tile2 = TileType::ImpassableMountain;
    assert_eq!(tile1, tile2);

    let tile3 = TileType::Floor;
    assert_ne!(tile1, tile3);
}

#[test]
fn tiletype_with_var_equality() {
    let tile1 = TileType::ToLocal(5);
    let tile2 = TileType::ToLocal(3);
    assert_ne!(tile1, tile2);

    let tile3 = TileType::ToLocal(3);
    assert_eq!(tile2, tile3);
}
