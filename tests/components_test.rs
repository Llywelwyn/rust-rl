// tests/components_test.rs
use rust_rl::components::*;
use std::collections::HashMap;

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

#[test]
fn get_damage_modifiers() {
    let dm = HasDamageModifiers {
        modifiers: {
            let mut m = HashMap::new();
            m.insert(DamageType::Physical, DamageModifier::Weakness);
            m.insert(DamageType::Magic, DamageModifier::Resistance);
            m
        },
    };
    assert_eq!(dm.modifier(&DamageType::Physical), &DamageModifier::Weakness);
    assert_eq!(dm.modifier(&DamageType::Magic), &DamageModifier::Resistance);
    assert_ne!(dm.modifier(&DamageType::Forced), &DamageModifier::Immune);
}

#[test]
fn get_damage_modifier_multiplier() {
    let none_mod = &DamageModifier::None.multiplier();
    let weak_mod = &DamageModifier::Weakness.multiplier();
    let res_mod = &DamageModifier::Resistance.multiplier();
    let immune_mod = &DamageModifier::Immune.multiplier();
    assert_eq!(none_mod, &1.0);
    assert_eq!(weak_mod, &2.0);
    assert_eq!(res_mod, &0.5);
    assert_eq!(immune_mod, &0.0);
}
