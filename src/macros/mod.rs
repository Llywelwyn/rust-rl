// macros/mod.rs

#[macro_export]
macro_rules! player {
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
macro_rules! add_intr {
    ($ecs:expr, $intrinsic:expr) => {
        let player = $ecs.fetch::<Entity>();
        let mut intrinsics = $ecs.write_storage::<crate::Intrinsics>();
        if let Some(player_intrinsics) = intrinsics.get_mut(*player) {
            if !player_intrinsics.list.contains(&$intrinsic) {
                player_intrinsics.list.insert($intrinsic);
                let mut intrinsic_changed = $ecs.write_storage::<crate::IntrinsicChanged>();
                if let Some(this_intrinsic_changed) = intrinsic_changed.get_mut(*player) {
                    this_intrinsic_changed.gained.insert($intrinsic);
                } else {
                    intrinsic_changed.insert(*player, crate::IntrinsicChanged {
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
            unreachable!("add_intr!(): The player should always have an Intrinsics component.");
        }
    };
}
