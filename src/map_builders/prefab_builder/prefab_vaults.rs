#[allow(dead_code)]
#[derive(PartialEq, Copy, Clone)]
pub struct PrefabVault {
    pub template: &'static str,
    pub width: usize,
    pub height: usize,
    pub first_depth: i32,
    pub last_depth: i32,
}

#[allow(dead_code)]
pub const CLASSIC_TRAP_5X5: PrefabVault =
    PrefabVault { template: CLASSIC_TRAP_5X5_V, width: 5, height: 5, first_depth: 0, last_depth: 100 };
#[allow(dead_code)]
const CLASSIC_TRAP_5X5_V: &str = "
     
 ^^^ 
 ^!^ 
 ^^^ 
     
";

#[allow(dead_code)]
pub const GOBLINS_4X4: PrefabVault =
    PrefabVault { template: GOBLINS_4X4_V, width: 4, height: 4, first_depth: 0, last_depth: 100 };
const GOBLINS_4X4_V: &str = "
#^  
 #G#
#g# 
 ^g^
";

#[allow(dead_code)]
pub const GOBLINS2_4X4: PrefabVault =
    PrefabVault { template: GOBLINS2_4X4_V, width: 4, height: 4, first_depth: 0, last_depth: 100 };
const GOBLINS2_4X4_V: &str = "
#^#g
G# #
 g# 
# g^
";
