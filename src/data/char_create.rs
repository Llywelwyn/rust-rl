// --- GUI ---
pub const CHAR_CREATE_HEADER: &str = "Who are you? [Aa-Zz]";
pub const ANCESTRY_INFO_HEADER: &str = "Your ancestry grants...";
pub const CLASS_INFO_HEADER: &str = "Your class grants...";
// --- ANCESTRY RENDERABLES ---
pub const ELF_GLYPH: char = '@';
pub const ELF_COLOUR: (u8, u8, u8) = (0, 255, 0);
pub const DWARF_GLYPH: char = 'h';
pub const DWARF_COLOUR: (u8, u8, u8) = (255, 0, 0);
pub const CATFOLK_GLYPH: char = '@';
pub const CATFOLK_COLOUR: (u8, u8, u8) = (200, 200, 255);
// --- ANCESTRY BONUSES ---
pub const ELF_SPEED_BONUS: i32 = 1;
pub const ELF_TELEPATH_RANGE: i32 = 6;
pub const DWARF_DEFENCE_MOD: i32 = 1;
pub const CATFOLK_SPEED_BONUS: i32 = 2;
// --- ANCESTRY ATTRIBUTE MAXIMUMS ---
pub const TOTAL_ATTRIBUTE_POINTS_MAXIMUM: i32 = 75;
pub const HUMAN_MAX_ATTR: [i32; 6] = [19, 19, 19, 19, 19, 19];
pub const ELF_MAX_ATTR: [i32; 6] = [15, 18, 15, 20, 20, 18];
pub const DWARF_MAX_ATTR: [i32; 6] = [19, 17, 20, 16, 16, 16];
pub const GNOME_MAX_ATTR: [i32; 6] = [16, 18, 16, 20, 18, 18];
pub const CATFOLK_MAX_ATTR: [i32; 6] = [16, 20, 16, 16, 18, 20];
pub const UNKNOWN_MAX_ATTR: [i32; 6] = [18, 18, 18, 18, 18, 18];
// --- CLASS MIN ATTRIBUTES ---
pub const FIGHTER_MIN_ATTR: (i32, i32, i32, i32, i32, i32) = (10, 8, 10, 6, 6, 8);
pub const ROGUE_MIN_ATTR: (i32, i32, i32, i32, i32, i32) = (8, 10, 8, 6, 8, 10);
pub const WIZARD_MIN_ATTR: (i32, i32, i32, i32, i32, i32) = (6, 8, 6, 10, 10, 8);
pub const VILLAGER_MIN_ATTR: (i32, i32, i32, i32, i32, i32) = (6, 6, 6, 6, 6, 6);
// --- CLASS ATTRIBUTE IMPROVE CHANCES ---
pub const FIGHTER_IMPR_CHANCE: [i32; 6] = [30, 20, 30, 6, 7, 7];
pub const ROGUE_IMPR_CHANCE: [i32; 6] = [18, 30, 20, 9, 8, 15];
pub const WIZARD_IMPR_CHANCE: [i32; 6] = [10, 15, 20, 30, 15, 10];
pub const VILLAGER_IMPR_CHANCE: [i32; 6] = [15, 15, 25, 15, 15, 15];
// --- CLASS STARTING ITEMS --- ## If any of these are changed, update ancestry infotext in src/gui/character_creation.rs.
pub const FIGHTER_STARTING_FOOD: &str = "1d2+1";
pub const FIGHTER_STARTING_WEAPON: &str = "equip_shortsword";
pub const FIGHTER_STARTING_ARMOUR: &str = "equip_body_ringmail";
pub const FIGHTER_STARTING_SHIELD: &str = "equip_mediumshield";
pub const ROGUE_STARTING_FOOD: &str = "1d2+2";
pub const ROGUE_STARTING_WEAPON: &str = "equip_rapier";
pub const ROGUE_STARTING_ARMOUR: &str = "equip_body_weakleather";
pub const WIZARD_STARTING_FOOD: &str = "1d2+1";
pub const WIZARD_STARTING_WEAPON: &str = "equip_dagger";
pub const WIZARD_STARTING_ARMOUR: &str = "equip_back_protection";
pub const WIZARD_MAX_SCROLL_LVL: i32 = 3;
pub const WIZARD_SCROLL_AMOUNT: &str = "1d3";
pub const WIZARD_POTION_AMOUNT: &str = "1d3-1";
pub const VILLAGER_STARTING_FOOD: &str = "1d3+2";
