// macros/mod.rs

#[macro_export]
/// Used to check if the player has a given component.
macro_rules! player_has_component {
    ($ecs:expr, $component:ty) => {
        {
            let player = $ecs.fetch::<Entity>();
            let component = $ecs.read_storage::<$component>();
            if let Some(player_component) = component.get(*player) {
                true
            } else {
                false
            }
        }
    };
}

#[macro_export]
/// Used to check if a given entity has a given Intrinsic.
macro_rules! has {
    ($ecs:expr, $entity:expr, $intrinsic:expr) => {
        {
            let intrinsics = $ecs.read_storage::<crate::Intrinsics>();
            if let Some(has_intrinsics) = intrinsics.get($entity) {
                has_intrinsics.list.contains(&$intrinsic)
            } else {
                false
            }
        }
    };
}

#[macro_export]
/// Used to check if the player has a given Intrinsic.
macro_rules! player_has {
    ($ecs:expr, $intrinsic:expr) => {
        {
            let player = $ecs.fetch::<Entity>();
            let intrinsics = $ecs.read_storage::<crate::Intrinsics>();
            if let Some(player_intrinsics) = intrinsics.get(*player) {
                player_intrinsics.list.contains(&$intrinsic)
            } else {
                false
            }
        }
    };
}

#[macro_export]
/// Handles adding an Intrinsic to the player, and adding it to the IntrinsicChanged component.
macro_rules! add_intr {
    ($ecs:expr, $entity:expr, $intrinsic:expr) => {
        {
            let mut intrinsics = $ecs.write_storage::<crate::Intrinsics>();
            if let Some(player_intrinsics) = intrinsics.get_mut($entity) {
                if !player_intrinsics.list.contains(&$intrinsic) {
                    player_intrinsics.list.insert($intrinsic);
                    let mut intrinsic_changed = $ecs.write_storage::<crate::IntrinsicChanged>();
                    if let Some(this_intrinsic_changed) = intrinsic_changed.get_mut($entity) {
                        this_intrinsic_changed.gained.insert($intrinsic);
                    } else {
                        intrinsic_changed.insert($entity, crate::IntrinsicChanged {
                            gained: {
                                let mut m = std::collections::HashSet::new();
                                m.insert($intrinsic);
                                m
                            },
                            lost: std::collections::HashSet::new()
                        }).expect("Failed to insert IntrinsicChanged component.");
                    }
                }
            } else {
                intrinsics.insert($entity, crate::Intrinsics {
                    list: {
                        let mut m = std::collections::HashSet::new();
                        m.insert($intrinsic);
                        m
                    }
                }).expect("Failed to insert Intrinsics component.");
                let mut intrinsic_changed = $ecs.write_storage::<crate::IntrinsicChanged>();
                intrinsic_changed.insert($entity, crate::IntrinsicChanged {
                    gained: {
                        let mut m = std::collections::HashSet::new();
                        m.insert($intrinsic);
                        m
                    },
                    lost: std::collections::HashSet::new()
                }).expect("Failed to insert IntrinsicChanged component.");
            }
        }
    };
}
