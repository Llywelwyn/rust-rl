// tests/components_test.rs
use rust_rl::components::*;

#[test]
fn damagetype_equality() {
    let dt1 = DamageType::Physical;
    let dt2 = DamageType::Physical;
    assert_eq!(dt1, dt2);
    let dt3 = DamageType::Magic;
    assert_ne!(dt1, dt3);
}

#[test]
fn damagetype_ismagic() {
    let dt1 = DamageType::Physical;
    let dt2 = DamageType::Magic;
    assert!(!dt1.is_magic());
    assert!(dt2.is_magic());
}
