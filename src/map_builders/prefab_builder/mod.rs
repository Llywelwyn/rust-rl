use super::{remove_unreachable_areas_returning_most_distant, Map, MapBuilder, Position, TileType, SHOW_MAPGEN};
use rltk::RandomNumberGenerator;
pub mod prefab_levels;
pub mod prefab_sections;
pub mod prefab_vaults;
use std::collections::HashSet;

#[allow(dead_code)]
#[derive(PartialEq, Clone)]
pub enum PrefabMode {
    RexLevel { template: &'static str },
    Constant { level: prefab_levels::PrefabLevel },
    Sectional { section: prefab_sections::PrefabSection },
    Vaults,
}

pub struct PrefabBuilder {
    map: Map,
    starting_position: Position,
    depth: i32,
    history: Vec<Map>,
    mode: PrefabMode,
    spawn_list: Vec<(usize, String)>,
    previous_builder: Option<Box<dyn MapBuilder>>,
}

impl MapBuilder for PrefabBuilder {
    fn build_map(&mut self, rng: &mut RandomNumberGenerator) {
        return self.build(rng);
    }
    //  Getters
    fn get_map(&mut self) -> Map {
        return self.map.clone();
    }
    fn get_starting_pos(&mut self) -> Position {
        return self.starting_position.clone();
    }
    fn get_spawn_list(&self) -> &Vec<(usize, String)> {
        return &self.spawn_list;
    }
    // Mapgen visualisation stuff
    fn get_snapshot_history(&self) -> Vec<Map> {
        return self.history.clone();
    }
    fn take_snapshot(&mut self) {
        if SHOW_MAPGEN {
            let mut snapshot = self.map.clone();
            for v in snapshot.revealed_tiles.iter_mut() {
                *v = true;
            }
            self.history.push(snapshot);
        }
    }
}

#[allow(dead_code)]
impl PrefabBuilder {
    pub fn rex_level(new_depth: i32, template: &'static str) -> PrefabBuilder {
        PrefabBuilder {
            map: Map::new(new_depth),
            starting_position: Position { x: 0, y: 0 },
            depth: new_depth,
            history: Vec::new(),
            mode: PrefabMode::RexLevel { template },
            spawn_list: Vec::new(),
            previous_builder: None,
        }
    }
    pub fn constant(new_depth: i32, level: prefab_levels::PrefabLevel) -> PrefabBuilder {
        PrefabBuilder {
            map: Map::new(new_depth),
            starting_position: Position { x: 0, y: 0 },
            depth: new_depth,
            history: Vec::new(),
            mode: PrefabMode::Constant { level },
            spawn_list: Vec::new(),
            previous_builder: None,
        }
    }
    pub fn sectional(
        new_depth: i32,
        section: prefab_sections::PrefabSection,
        previous_builder: Box<dyn MapBuilder>,
    ) -> PrefabBuilder {
        PrefabBuilder {
            map: Map::new(new_depth),
            starting_position: Position { x: 0, y: 0 },
            depth: new_depth,
            history: Vec::new(),
            mode: PrefabMode::Sectional { section },
            spawn_list: Vec::new(),
            previous_builder: Some(previous_builder),
        }
    }
    pub fn vaults(new_depth: i32, previous_builder: Box<dyn MapBuilder>) -> PrefabBuilder {
        PrefabBuilder {
            map: Map::new(new_depth),
            starting_position: Position { x: 0, y: 0 },
            depth: new_depth,
            history: Vec::new(),
            mode: PrefabMode::Vaults,
            spawn_list: Vec::new(),
            previous_builder: Some(previous_builder),
        }
    }

    fn build(&mut self, rng: &mut RandomNumberGenerator) {
        match self.mode {
            PrefabMode::RexLevel { template } => self.load_rex_map(&template),
            PrefabMode::Constant { level } => self.load_ascii_map(&level),
            PrefabMode::Sectional { section } => self.apply_sectional(&section, rng),
            PrefabMode::Vaults => self.apply_room_vaults(rng),
        }
        self.take_snapshot();

        // Find starting pos by starting at middle and walking left until finding a floor tile
        if self.starting_position.x == 0 {
            self.starting_position = Position { x: self.map.width / 2, y: self.map.height / 2 };
            let mut start_idx = self.map.xy_idx(self.starting_position.x, self.starting_position.y);
            while self.map.tiles[start_idx] != TileType::Floor {
                self.starting_position.x -= 1;
                start_idx = self.map.xy_idx(self.starting_position.x, self.starting_position.y);
            }
            self.take_snapshot();

            // Find all tiles we can reach from the starting point
            let exit_tile = remove_unreachable_areas_returning_most_distant(&mut self.map, start_idx);
            self.take_snapshot();

            // Place the stairs
            self.map.tiles[exit_tile] = TileType::DownStair;
            self.take_snapshot();
        }
    }

    fn apply_previous_iteration<F>(&mut self, rng: &mut RandomNumberGenerator, mut filter: F)
    where
        F: FnMut(i32, i32, &(usize, String)) -> bool,
    {
        // Build the map
        let prev_builder = self.previous_builder.as_mut().unwrap();
        prev_builder.build_map(rng);
        self.starting_position = prev_builder.get_starting_pos();
        self.map = prev_builder.get_map().clone();
        self.history = prev_builder.get_snapshot_history();
        for e in prev_builder.get_spawn_list().iter() {
            let idx = e.0;
            let x = idx as i32 % self.map.width;
            let y = idx as i32 / self.map.width;
            if filter(x, y, e) {
                self.spawn_list.push((idx, e.1.to_string()))
            }
        }
        self.take_snapshot();
    }

    fn apply_room_vaults(&mut self, rng: &mut RandomNumberGenerator) {
        use prefab_vaults::*;

        // Apply prev builder, and keep all entities
        self.apply_previous_iteration(rng, |_x, _y, _e| true);

        // Roll for a vault
        let vault_roll = rng.roll_dice(1, 6);
        if vault_roll < 4 {
            return;
        }

        // Get all vaults
        let master_vault_list = vec![GOBLINS_4X4, GOBLINS2_4X4, CLASSIC_TRAP_5X5];
        // Filter out vaults from outside the current depth
        let mut possible_vaults: Vec<&PrefabVault> =
            master_vault_list.iter().filter(|v| self.depth >= v.first_depth && self.depth <= v.last_depth).collect();
        // Return if there's no possible vaults
        if possible_vaults.is_empty() {
            return;
        }

        // Pick number of vaults
        let n_vaults = i32::min(rng.roll_dice(1, 3), possible_vaults.len() as i32);
        let mut used_tiles: HashSet<usize> = HashSet::new();

        for _i in 0..n_vaults {
            // Select a vault
            let vault_idx = if possible_vaults.len() == 1 {
                0
            } else {
                (rng.roll_dice(1, possible_vaults.len() as i32) - 1) as usize
            };
            let vault = possible_vaults[vault_idx];
            // Decide if we want to flip the vault
            let mut flip_x: bool = false;
            let mut flip_y: bool = false;
            match vault.can_flip {
                // Equal chance at every orientation
                Flipping::None => {}
                Flipping::Horizontal => {
                    flip_x = rng.roll_dice(1, 2) == 1;
                }
                Flipping::Vertical => {
                    flip_y = rng.roll_dice(1, 2) == 1;
                }
                Flipping::Both => {
                    let roll = rng.roll_dice(1, 4);
                    match roll {
                        1 => {}
                        2 => flip_x = true,
                        3 => flip_y = true,
                        _ => {
                            flip_x = true;
                            flip_y = true;
                        }
                    }
                }
            }

            // Make a list of all places the vault can fit
            let mut vault_positions: Vec<Position> = Vec::new();
            let mut idx = 0usize;
            loop {
                let x = (idx % self.map.width as usize) as i32;
                let y = (idx / self.map.width as usize) as i32;
                // Check for overflow
                if x > 1
                    && (x + vault.width as i32) < self.map.width - 2
                    && y > 1
                    && (y + vault.height as i32) < self.map.height - 2
                {
                    let mut possible = true;
                    for tile_y in 0..vault.height as i32 {
                        for tile_x in 0..vault.width as i32 {
                            let idx = self.map.xy_idx(tile_x + x, tile_y + y);
                            if self.map.tiles[idx] != TileType::Floor {
                                possible = false;
                            }
                            if used_tiles.contains(&idx) {
                                possible = false;
                            }
                        }
                    }
                    // If we find a position that works, push it
                    if possible {
                        vault_positions.push(Position { x, y });
                    }
                }
                // Once we reach the end of the map, break
                idx += 1;
                if idx >= self.map.tiles.len() - 1 {
                    break;
                }
            }

            // If we have a position, make the vault
            if !vault_positions.is_empty() {
                let pos_idx = if vault_positions.len() == 1 {
                    0
                } else {
                    (rng.roll_dice(1, vault_positions.len() as i32) - 1) as usize
                };
                let pos = &vault_positions[pos_idx];
                let chunk_x = pos.x;
                let chunk_y = pos.y;
                // Filter out entities from our spawn list that would have spawned inside this vault
                let width = self.map.width; // For borrow checker.
                let height = self.map.height; // As above.
                self.spawn_list.retain(|e| {
                    let idx = e.0 as i32;
                    let x = idx % width;
                    let y = idx / height;
                    x < chunk_x || x > chunk_x + vault.width as i32 || y < chunk_y || y > chunk_y + vault.height as i32
                });
                let string_vec = PrefabBuilder::read_ascii_to_vec(vault.template);
                let mut i = 0;
                for tile_y in 0..vault.height {
                    for tile_x in 0..vault.width {
                        let mut x_: i32 = tile_x as i32;
                        let mut y_: i32 = tile_y as i32;
                        // Handle flipping
                        if flip_x {
                            x_ = vault.height as i32 - 1 - x_;
                        }
                        if flip_y {
                            y_ = vault.width as i32 - 1 - y_;
                        }
                        self.map.xy_idx(x_ + chunk_x, y_ as i32 + chunk_y);
                        self.char_to_map(string_vec[i], idx);
                        used_tiles.insert(idx);
                        i += 1;
                    }
                }
                rltk::console::log("-> adding vault");
                self.take_snapshot();
                possible_vaults.remove(vault_idx);
            }
        }
    }

    pub fn apply_sectional(&mut self, section: &prefab_sections::PrefabSection, rng: &mut RandomNumberGenerator) {
        use prefab_sections::*;
        let string_vec = PrefabBuilder::read_ascii_to_vec(section.template);

        // Place the new section
        let chunk_x;
        match section.placement.0 {
            HorizontalPlacement::Left => chunk_x = 0,
            HorizontalPlacement::Center => chunk_x = (self.map.width / 2) - (section.width as i32 / 2),
            HorizontalPlacement::Right => chunk_x = (self.map.width - 1) - section.width as i32,
        }

        let chunk_y;
        match section.placement.1 {
            VerticalPlacement::Top => chunk_y = 0,
            VerticalPlacement::Center => chunk_y = (self.map.height / 2) - (section.height as i32 / 2),
            VerticalPlacement::Bottom => chunk_y = (self.map.height - 1) - section.height as i32,
        }

        // Build the map
        self.apply_previous_iteration(rng, |x, y, _e| {
            x < chunk_x || x > (chunk_x + section.width as i32) || y < chunk_y || y > (chunk_y + section.height as i32)
        });

        let mut i = 0;
        for ty in 0..section.height {
            for tx in 0..section.width {
                if tx < self.map.width as usize && ty < self.map.height as usize {
                    let idx = self.map.xy_idx(tx as i32 + chunk_x, ty as i32 + chunk_y);
                    self.char_to_map(string_vec[i], idx);
                }
                i += 1;
            }
        }
        self.take_snapshot();
    }

    fn char_to_map(&mut self, ch: char, idx: usize) {
        match ch {
            ' ' => self.map.tiles[idx] = TileType::Floor,
            '#' => self.map.tiles[idx] = TileType::Wall,
            '>' => self.map.tiles[idx] = TileType::DownStair,
            '@' => {
                let x = idx as i32 % self.map.width;
                let y = idx as i32 / self.map.width;
                self.map.tiles[idx] = TileType::Floor;
                self.starting_position = Position { x: x as i32, y: y as i32 };
            }
            'g' => {
                self.map.tiles[idx] = TileType::Floor;
                self.spawn_list.push((idx, "goblin".to_string()));
            }
            'G' => {
                self.map.tiles[idx] = TileType::Floor;
                self.spawn_list.push((idx, "goblin chieftain".to_string()));
            }
            'o' => {
                self.map.tiles[idx] = TileType::Floor;
                self.spawn_list.push((idx, "orc".to_string()));
            }
            '^' => {
                self.map.tiles[idx] = TileType::Floor;
                self.spawn_list.push((idx, "bear trap".to_string()));
            }
            '%' => {
                self.map.tiles[idx] = TileType::Floor;
                self.spawn_list.push((idx, "rations".to_string()));
            }
            '!' => {
                self.map.tiles[idx] = TileType::Floor;
                self.spawn_list.push((idx, "health potion".to_string()));
            }
            _ => {
                rltk::console::log(format!("Unknown glyph loading map: {}", (ch as u8) as char));
            }
        }
    }

    #[allow(dead_code)]
    fn load_rex_map(&mut self, path: &str) {
        let xp_file = rltk::rex::XpFile::from_resource(path).unwrap();

        for layer in &xp_file.layers {
            for y in 0..layer.height {
                for x in 0..layer.width {
                    let cell = layer.get(x, y).unwrap();
                    if x < self.map.width as usize && y < self.map.height as usize {
                        // Saving these for later, for flipping the pref horizontally/vertically/both.
                        // let flipped_x = (self.map.width - 1) - x as i32;
                        // let flipped_y = (self.map.height - 1) - y as i32;
                        let idx = self.map.xy_idx(x as i32, y as i32);
                        // We're doing some nasty casting to make it easier to type things like '#' in the match
                        self.char_to_map(cell.ch as u8 as char, idx);
                    }
                }
            }
        }
    }

    fn read_ascii_to_vec(template: &str) -> Vec<char> {
        let mut string_vec: Vec<char> = template.chars().filter(|a| *a != '\r' && *a != '\n').collect();
        for c in string_vec.iter_mut() {
            if *c as u8 == 160u8 {
                *c = ' ';
            }
        }
        return string_vec;
    }

    #[allow(dead_code)]
    fn load_ascii_map(&mut self, level: &prefab_levels::PrefabLevel) {
        let string_vec = PrefabBuilder::read_ascii_to_vec(level.template);

        let mut i = 0;
        for y in 0..level.height {
            for x in 0..level.width {
                if x < self.map.width as usize && y < self.map.height as usize {
                    let idx = self.map.xy_idx(x as i32, y as i32);
                    self.char_to_map(string_vec[i], idx);
                }
                i += 1;
            }
        }
    }
}
