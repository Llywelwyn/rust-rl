#[allow(dead_code)]
#[derive(PartialEq, Copy, Clone)]
pub enum Flipping {
    None,
    Horizontal,
    Vertical,
    Both,
}
#[allow(dead_code)]
#[derive(PartialEq, Copy, Clone)]
pub struct PrefabVault {
    pub template: &'static str,
    pub width: usize,
    pub height: usize,
    pub first_depth: i32,
    pub last_depth: i32,
    pub can_flip: Flipping,
}

pub const CLASSIC_TRAP_5X5: PrefabVault = PrefabVault {
    template: CLASSIC_TRAP_5X5_V,
    width: 5,
    height: 5,
    first_depth: 0,
    last_depth: 100,
    can_flip: Flipping::None,
};
const CLASSIC_TRAP_5X5_V: &str = "
     
 ^^^ 
 ^!^ 
 ^^^ 
     
";

pub const CLASSIC_TRAP_CARDINALGAP_5X5: PrefabVault = PrefabVault {
    template: CLASSIC_TRAP_CARDINALGAP_5X5_V,
    width: 5,
    height: 5,
    first_depth: 0,
    last_depth: 100,
    can_flip: Flipping::Both,
};
const CLASSIC_TRAP_CARDINALGAP_5X5_V: &str = "
     
 ^ ^ 
 ^!^ 
 ^^^ 
     
";

pub const CLASSIC_TRAP_DIAGONALGAP_5X5: PrefabVault = PrefabVault {
    template: CLASSIC_TRAP_DIAGONALGAP_5X5_V,
    width: 5,
    height: 5,
    first_depth: 0,
    last_depth: 100,
    can_flip: Flipping::Both,
};
const CLASSIC_TRAP_DIAGONALGAP_5X5_V: &str = "
     
 ^^  
 ^!^ 
 ^^^ 
     
";

pub const GOBLINS_4X4: PrefabVault = PrefabVault {
    template: GOBLINS_4X4_V,
    width: 4,
    height: 4,
    first_depth: 0,
    last_depth: 100,
    can_flip: Flipping::Both,
};
const GOBLINS_4X4_V: &str = "
#^  
 #G#
#g# 
 ^g^
";

pub const GOBLINS2_4X4: PrefabVault = PrefabVault {
    template: GOBLINS2_4X4_V,
    width: 4,
    height: 4,
    first_depth: 0,
    last_depth: 100,
    can_flip: Flipping::Both,
};
const GOBLINS2_4X4_V: &str = "
#^#g
G# #
 g# 
# g^
";

pub const GOBLINS_5X5: PrefabVault = PrefabVault {
    template: GOBLINS_5X5_V,
    width: 5,
    height: 5,
    first_depth: 0,
    last_depth: 100,
    can_flip: Flipping::Both,
};
const GOBLINS_5X5_V: &str = "
 ^#g 
G#?#^
/g g#
## ^#
^# # 
";

pub const GOBLINS_6X6: PrefabVault = PrefabVault {
    template: GOBLINS_6X6_V,
    width: 6,
    height: 6,
    first_depth: 0,
    last_depth: 100,
    can_flip: Flipping::Both,
};
const GOBLINS_6X6_V: &str = "
   #  
 #^#g 
#G#$#^
 /gGg#
g##$^ 
 ^ # ^
";

pub const FLUFF_6X3: PrefabVault = PrefabVault {
    template: FLUFF_6X3_V,
    width: 6,
    height: 3,
    first_depth: 0,
    last_depth: 100,
    can_flip: Flipping::Both,
};
const FLUFF_6X3_V: &str = "
###≈^ 
 ^≈ #≈
 ≈##≈ 
";

pub const FLUFF2_6X3: PrefabVault = PrefabVault {
    template: FLUFF2_6X3_V,
    width: 6,
    height: 3,
    first_depth: 0,
    last_depth: 100,
    can_flip: Flipping::Both,
};
const FLUFF2_6X3_V: &str = "
 ^≈###
≈# ≈^ 
 ≈##≈ 
";

pub const HOUSE_NOTRAP_7X7: PrefabVault = PrefabVault {
    template: HOUSE_NOTRAP_7X7_V,
    width: 7,
    height: 7,
    first_depth: 0,
    last_depth: 100,
    can_flip: Flipping::Both,
};
const HOUSE_NOTRAP_7X7_V: &str = "
#   # 
#   g #
  # #  
   ?   
  # #  
# g   #
 #   # 
";

pub const HOUSE_TRAP_7X7: PrefabVault = PrefabVault {
    template: HOUSE_TRAP_7X7_V,
    width: 7,
    height: 7,
    first_depth: 0,
    last_depth: 100,
    can_flip: Flipping::Both,
};
const HOUSE_TRAP_7X7_V: &str = "
#   # 
#     #
  #^#  
  ^? g 
 g#^#  
#     #
 #   # 
";

pub const ORC_HOUSE_8X8: PrefabVault = PrefabVault {
    template: ORC_HOUSE_8X8_V,
    width: 8,
    height: 8,
    first_depth: 0,
    last_depth: 100,
    can_flip: Flipping::Both,
};
const ORC_HOUSE_8X8_V: &str = "
        
###### 
#o g # 
#   %# 
# %o # 
#   ?# 
##+### 
       
";
