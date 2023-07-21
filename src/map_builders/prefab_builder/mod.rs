use super::{
    remove_unreachable_areas_returning_most_distant, spawner, Map, MapBuilder, Position, TileType, SHOW_MAPGEN,
};
use rltk::RandomNumberGenerator;
use specs::prelude::*;
mod prefab_levels;
mod prefab_sections;

#[allow(dead_code)]
#[derive(PartialEq, Clone)]
pub enum PrefabMode {
    RexLevel { template: &'static str },
    Constant { level: prefab_levels::PrefabLevel },
    Sectional { section: prefab_sections::PrefabSection },
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

impl PrefabBuilder {
    #[allow(dead_code)]
    pub fn new(new_depth: i32, previous_builder: Option<Box<dyn MapBuilder>>) -> PrefabBuilder {
        PrefabBuilder {
            map: Map::new(new_depth),
            starting_position: Position { x: 0, y: 0 },
            depth: new_depth,
            history: Vec::new(),
            mode: PrefabMode::Sectional { section: prefab_sections::UNDERGROUND_FORT },
            spawn_list: Vec::new(),
            previous_builder,
        }
    }

    fn build(&mut self, rng: &mut RandomNumberGenerator) {
        match self.mode {
            PrefabMode::RexLevel { template } => self.load_rex_map(&template),
            PrefabMode::Constant { level } => self.load_ascii_map(&level),
            PrefabMode::Sectional { section } => self.apply_sectional(&section, rng),
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
        let prev_builder = self.previous_builder.as_mut().unwrap();
        prev_builder.build_map(rng);
        self.starting_position = prev_builder.get_starting_pos();
        self.map = prev_builder.get_map().clone();
        // Iterate previous spawn list, culling entities within new section
        for entity in prev_builder.get_spawn_list().iter() {
            let idx = entity.0;
            let x = idx as i32 % self.map.width;
            let y = idx as i32 / self.map.width;
            if x < chunk_x
                || x > (chunk_x + section.width as i32)
                || y < chunk_y
                || y > (chunk_y + section.height as i32)
            {
                self.spawn_list.push((idx, entity.1.to_string()));
            }
        }
        self.take_snapshot();

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
