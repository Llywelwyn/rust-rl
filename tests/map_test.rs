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

fn init_maps_for_tests() -> (MasterDungeonMap, Map, Map) {
    let dm = MasterDungeonMap::new();
    let (overmap, difficulty, name, short_name, depth) = (false, 0, "Test Map", "Test Map", 0);
    let map1 = Map::new(overmap, 1, 64, 64, difficulty, name, short_name, depth);
    let map2 = Map::new(overmap, 2, 128, 128, difficulty, name, short_name, depth);
    (dm, map1, map2)
}

#[test]
fn map_saving() {
    let (mut dm, map1, map2) = init_maps_for_tests();
    dm.store_map(&map1);
    dm.store_map(&map2);
}

#[test]
fn map_loading() {
    let (mut dm, map1, map2) = init_maps_for_tests();
    dm.store_map(&map1);
    let loaded_map1 = dm.get_map(map1.id).unwrap();
    assert_eq!(loaded_map1.overmap, map1.overmap);
    assert_eq!(loaded_map1.id, map1.id);
    assert_eq!(loaded_map1.width, map1.width);
    assert_eq!(loaded_map1.height, map1.height);
    assert_eq!(loaded_map1.difficulty, map1.difficulty);
    assert_eq!(loaded_map1.name, map1.name);
    assert_eq!(loaded_map1.short_name, map1.short_name);
    assert_eq!(loaded_map1.depth, map1.depth);
    assert_eq!(loaded_map1.tiles.len(), map1.tiles.len());
    assert_eq!(loaded_map1.messages, map1.messages);
    dm.store_map(&map2);
    let loaded_map2 = dm.get_map(map2.id).unwrap();
    assert_eq!(loaded_map2.width, map2.width);
    assert_ne!(loaded_map2.width, map1.width);
}
