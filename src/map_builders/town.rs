use super::{BuilderChain, BuilderMap, InitialMapBuilder, Position, TileType};
use std::collections::HashSet;

pub fn town_builder(new_id: i32, _rng: &mut rltk::RandomNumberGenerator, width: i32, height: i32) -> BuilderChain {
    let difficulty = 0;
    rltk::console::log(format!("DEBUGINFO: Building town (ID:{}, DIFF:{})", new_id, difficulty));
    let mut chain = BuilderChain::new(new_id, width, height, difficulty);
    chain.start_with(TownBuilder::new());

    return chain;
}

pub struct TownBuilder {}

impl InitialMapBuilder for TownBuilder {
    #[allow(dead_code)]
    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data: &mut BuilderMap) {
        self.build_map(rng, build_data);
    }
}

impl TownBuilder {
    pub fn new() -> Box<TownBuilder> {
        return Box::new(TownBuilder {});
    }

    pub fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data: &mut BuilderMap) {
        // Make visible for snapshot
        for t in build_data.map.visible_tiles.iter_mut() {
            *t = true;
        }

        self.grass_layer(build_data);
        let piers = self.water_and_piers(rng, build_data);
        let (mut available_building_tiles, wall_gap_y) = self.town_walls(rng, build_data);
        let mut buildings = self.buildings(rng, build_data, &mut available_building_tiles);
        let doors = self.add_doors(rng, build_data, &mut buildings, wall_gap_y);
        self.path_from_tiles_to_nearest_tiletype(build_data, &doors, TileType::Road, TileType::Road, true);
        self.path_from_tiles_to_nearest_tiletype(build_data, &piers, TileType::Road, TileType::Road, false);

        build_data.take_snapshot();

        // Sort buildings by size
        let mut building_size: Vec<(usize, i32)> = Vec::new();
        for (i, building) in buildings.iter().enumerate() {
            building_size.push((i, building.2 * building.3));
        }
        building_size.sort_by(|a, b| b.1.cmp(&a.1));

        // Largest building as start position
        let tavern = &buildings[building_size[0].0];
        build_data.starting_position = Some(Position { x: tavern.0 + (tavern.2 / 2), y: tavern.1 + (tavern.3 / 2) });

        // Smallest building as mine entrance
        let mine = &buildings[building_size[building_size.len() - 1].0];
        let exit_idx = build_data.map.xy_idx(mine.0 + (mine.2 / 2), mine.1 + (mine.3 / 2));
        build_data.map.tiles[exit_idx] = TileType::DownStair;
    }

    fn grass_layer(&mut self, build_data: &mut BuilderMap) {
        // Grass everywhere
        for t in build_data.map.tiles.iter_mut() {
            *t = TileType::Grass;
        }
        build_data.take_snapshot();
    }

    fn water_and_piers(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data: &mut BuilderMap) -> Vec<usize> {
        let mut n = (rng.roll_dice(1, 65535) as f32) / 65535f32;
        let mut water_width: Vec<i32> = Vec::new();
        let variance = 5;
        let minimum_width = variance + 10;
        let shallow_width = 6;
        let sand_width = shallow_width + 4;

        for y in 0..build_data.height {
            let n_water = (f32::sin(n) * variance as f32) as i32 + minimum_width + rng.roll_dice(1, 2);
            water_width.push(n_water);
            n += 0.1;
            for x in 0..n_water {
                let idx = build_data.map.xy_idx(x, y);
                build_data.map.tiles[idx] = TileType::DeepWater;
            }
            for x in n_water..n_water + shallow_width {
                let idx = build_data.map.xy_idx(x, y);
                build_data.map.tiles[idx] = TileType::ShallowWater;
            }
            for x in n_water + shallow_width..n_water + sand_width {
                let idx = build_data.map.xy_idx(x, y);
                build_data.map.tiles[idx] = TileType::Sand;
            }
        }
        build_data.take_snapshot();

        // Add piers
        let mut placed_piers: Vec<i32> = Vec::new();
        let pier_width = 4;
        let mut pier_idxs: Vec<usize> = Vec::new();

        for _i in 0..rng.roll_dice(1, 3) + 2 {
            let mut y;
            loop {
                y = rng.roll_dice(1, build_data.height - 3) - 1;
                if !(placed_piers.contains(&y) || placed_piers.contains(&(y + pier_width))) {
                    break;
                }
            }

            for i in 0..=pier_width {
                placed_piers.push(y + i);
            }

            let start_roll = rng.roll_dice(1, 4);
            let largest_water_width;
            if water_width[y as usize] > water_width[y as usize + 1] {
                largest_water_width = water_width[y as usize];
            } else {
                largest_water_width = water_width[y as usize + 1];
            }

            // Make pier length
            for x in 2 + start_roll..largest_water_width + sand_width {
                let idx = build_data.map.xy_idx(x, y);
                build_data.map.tiles[idx] = TileType::Fence;
                let idx = build_data.map.xy_idx(x, y + 1);
                build_data.map.tiles[idx] = TileType::WoodFloor;
                let idx = build_data.map.xy_idx(x, y + 2);
                build_data.map.tiles[idx] = TileType::WoodFloor;
                let idx = build_data.map.xy_idx(x, y + 3);
                build_data.map.tiles[idx] = TileType::Fence;
            }

            // Set end of pier to fences
            for y in y + 1..y + pier_width - 1 {
                let idx = build_data.map.xy_idx(2 + start_roll, y);
                build_data.map.tiles[idx] = TileType::Fence;
            }
            build_data.take_snapshot();

            pier_idxs.push(build_data.map.xy_idx(largest_water_width + sand_width, y));
            pier_idxs.push(build_data.map.xy_idx(largest_water_width + sand_width, y + 1));
            pier_idxs.push(build_data.map.xy_idx(largest_water_width + sand_width, y + 2));
            pier_idxs.push(build_data.map.xy_idx(largest_water_width + sand_width, y + 3));
        }

        return pier_idxs;
    }

    fn town_walls(
        &mut self,
        rng: &mut rltk::RandomNumberGenerator,
        build_data: &mut BuilderMap,
    ) -> (HashSet<usize>, i32) {
        let mut available_building_tiles: HashSet<usize> = HashSet::new();

        const BORDER: i32 = 4;
        const OFFSET_FROM_LEFT: i32 = 30 + BORDER;
        const PATH_OFFSET_FROM_CENTRE: i32 = 4;
        const HALF_PATH_THICKNESS: i32 = 3;

        let wall_gap_y =
            (build_data.height / 2) + rng.roll_dice(1, PATH_OFFSET_FROM_CENTRE * 2) - 1 - PATH_OFFSET_FROM_CENTRE;

        for y in BORDER..build_data.height - BORDER {
            if !(y > wall_gap_y - HALF_PATH_THICKNESS && y < wall_gap_y + HALF_PATH_THICKNESS) {
                let idx = build_data.map.xy_idx(OFFSET_FROM_LEFT, y);
                build_data.map.tiles[idx] = TileType::Wall;

                let idx_right = build_data.map.xy_idx(build_data.width - BORDER, y);
                build_data.map.tiles[idx_right] = TileType::Wall;

                for x in OFFSET_FROM_LEFT + 1..build_data.width - BORDER {
                    let gravel_idx = build_data.map.xy_idx(x, y);
                    let roll = rng.roll_dice(1, 6);
                    match roll {
                        1 => build_data.map.tiles[gravel_idx] = TileType::Foliage,
                        2 => build_data.map.tiles[gravel_idx] = TileType::HeavyFoliage,
                        _ => {}
                    }
                    if y > BORDER + 1
                        && y < build_data.height - BORDER - 1
                        && x > OFFSET_FROM_LEFT + 2
                        && x < build_data.width - BORDER - 1
                    {
                        available_building_tiles.insert(gravel_idx);
                    }
                }
            } else {
                for x in OFFSET_FROM_LEFT - 3..build_data.width {
                    let road_idx = build_data.map.xy_idx(x, y);
                    build_data.map.tiles[road_idx] = TileType::Road;
                }
            }
        }
        build_data.take_snapshot();

        for x in OFFSET_FROM_LEFT..build_data.width - BORDER + 1 {
            let idx_top = build_data.map.xy_idx(x, BORDER - 1);
            build_data.map.tiles[idx_top] = TileType::Wall;
            let idx_bottom = build_data.map.xy_idx(x, build_data.height - BORDER);
            build_data.map.tiles[idx_bottom] = TileType::Wall;
        }
        build_data.take_snapshot();

        (available_building_tiles, wall_gap_y)
    }

    fn buildings(
        &mut self,
        rng: &mut rltk::RandomNumberGenerator,
        build_data: &mut BuilderMap,
        available_building_tiles: &mut HashSet<usize>,
    ) -> Vec<(i32, i32, i32, i32)> {
        let mut buildings: Vec<(i32, i32, i32, i32)> = Vec::new();
        let mut n_buildings = 0;

        const BORDER: i32 = 2;
        const REQUIRED_BUILDINGS: i32 = 8;
        const OFFSET_FROM_LEFT: i32 = 30;
        const MIN_BUILDING_SIZE: i32 = 4;
        const MAX_BUILDING_SIZE: i32 = 10;

        while n_buildings < REQUIRED_BUILDINGS {
            let bx = rng.roll_dice(1, build_data.map.width - OFFSET_FROM_LEFT - BORDER) + OFFSET_FROM_LEFT;
            let by = rng.roll_dice(1, build_data.map.height) - BORDER;
            let bw = rng.roll_dice(1, MAX_BUILDING_SIZE - MIN_BUILDING_SIZE) + MIN_BUILDING_SIZE;
            let bh = rng.roll_dice(1, MAX_BUILDING_SIZE - MIN_BUILDING_SIZE) + MIN_BUILDING_SIZE;
            let mut possible = true;
            for y in by..by + bh {
                for x in bx..bx + bw {
                    if x < 0 || x > build_data.width - 1 || y < 0 || y > build_data.height - 1 {
                        possible = false;
                    } else {
                        let idx = build_data.map.xy_idx(x, y);
                        if !available_building_tiles.contains(&idx) {
                            possible = false;
                        }
                    }
                }
            }
            if possible {
                n_buildings += 1;
                buildings.push((bx, by, bw, bh));
                for y in by..by + bh {
                    for x in bx..bx + bw {
                        let idx = build_data.map.xy_idx(x, y);
                        build_data.map.tiles[idx] = TileType::WoodFloor;
                        available_building_tiles.remove(&idx);
                        available_building_tiles.remove(&(idx + 1));
                        available_building_tiles.remove(&(idx + build_data.width as usize));
                        available_building_tiles.remove(&(idx - 1));
                        available_building_tiles.remove(&(idx - build_data.width as usize));
                    }
                }
                build_data.take_snapshot();
            }
        }

        // Outlines
        let mut mapclone = build_data.map.clone();
        for y in BORDER..build_data.height - BORDER {
            for x in OFFSET_FROM_LEFT + BORDER..build_data.width - BORDER {
                let idx = build_data.map.xy_idx(x, y);
                if build_data.map.tiles[idx] == TileType::WoodFloor {
                    let mut neighbours = 0;
                    if build_data.map.tiles[idx - 1] != TileType::WoodFloor {
                        neighbours += 1;
                    }
                    if build_data.map.tiles[idx + 1] != TileType::WoodFloor {
                        neighbours += 1;
                    }
                    if build_data.map.tiles[idx - build_data.width as usize] != TileType::WoodFloor {
                        neighbours += 1;
                    }
                    if build_data.map.tiles[idx + build_data.width as usize] != TileType::WoodFloor {
                        neighbours += 1;
                    }
                    if neighbours > 0 {
                        mapclone.tiles[idx] = TileType::Wall;
                    }
                }
            }
        }
        build_data.map = mapclone;
        build_data.take_snapshot();

        buildings
    }

    fn add_doors(
        &mut self,
        rng: &mut rltk::RandomNumberGenerator,
        build_data: &mut BuilderMap,
        buildings: &mut Vec<(i32, i32, i32, i32)>,
        wall_gap_y: i32,
    ) -> Vec<usize> {
        let mut doors = Vec::new();
        for building in buildings.iter() {
            let door_x = building.0 + 1 + rng.roll_dice(1, building.2 - 3);
            let cy = building.1 + (building.3 / 2);
            let idx = if cy > wall_gap_y {
                // Door on north wall
                build_data.map.xy_idx(door_x, building.1)
            } else {
                build_data.map.xy_idx(door_x, building.1 + building.3 - 1)
            };
            build_data.map.tiles[idx] = TileType::Floor;
            build_data.spawn_list.push((idx, "door".to_string()));
            doors.push(idx);
        }
        build_data.take_snapshot();
        doors
    }

    fn path_from_tiles_to_nearest_tiletype(
        &mut self,
        build_data: &mut BuilderMap,
        tiles: &[usize],
        tiletype: TileType,
        new_road_tiletype: TileType,
        include_new_tiles: bool,
    ) {
        let mut roads = self.find_tiletype(build_data, tiletype);

        build_data.map.populate_blocked();
        for tile_idx in tiles.iter() {
            let mut nearest_tiletype: Vec<(usize, f32)> = Vec::new();
            let tile_pt = rltk::Point::new(
                *tile_idx as i32 % build_data.map.width as i32,
                *tile_idx as i32 / build_data.map.width as i32,
            );
            for r in roads.iter() {
                nearest_tiletype.push((
                    *r,
                    rltk::DistanceAlg::Manhattan.distance2d(
                        tile_pt,
                        rltk::Point::new(*r as i32 % build_data.map.width, *r as i32 / build_data.map.width),
                    ),
                ));
            }
            nearest_tiletype.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

            let destination = nearest_tiletype[0].0;
            let path = rltk::a_star_search(*tile_idx, destination, &mut build_data.map);
            if path.success {
                for step in path.steps.iter() {
                    let idx = *step as usize;
                    build_data.map.tiles[idx] = new_road_tiletype;
                    if include_new_tiles {
                        roads.push(idx);
                    }
                }
            }
            build_data.take_snapshot();
        }
    }

    fn find_tiletype(&mut self, build_data: &mut BuilderMap, tile: TileType) -> Vec<usize> {
        let mut found_tiles = Vec::new();
        for y in 0..build_data.height {
            for x in 0..build_data.width {
                let idx = build_data.map.xy_idx(x, y);
                if build_data.map.tiles[idx] == tile {
                    found_tiles.push(idx);
                }
            }
        }

        found_tiles
    }
}
