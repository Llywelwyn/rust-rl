// tests/map_test.rs
use rust_rl::map::TileType;

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
