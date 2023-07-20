use super::TileType;

#[derive(PartialEq, Eq, Hash, Clone)]
pub struct MapChunk {
    pub pattern: Vec<TileType>,
    pub exits: [Vec<bool>; 4],
    pub has_exits: bool,
    pub compatible_with: [Vec<usize>; 4],
}

pub fn tile_idx_in_chunk(chunk_size: i32, x: i32, y: i32) -> usize {
    ((y * chunk_size) + x) as usize
}

pub fn patterns_to_constraints(patterns: Vec<Vec<TileType>>, chunk_size: i32) -> Vec<MapChunk> {
    // Move patterns into constraints obj
    let mut constraints: Vec<MapChunk> = Vec::new();
    for pattern in patterns {
        let mut new_chunk = MapChunk {
            pattern: pattern,
            exits: [Vec::new(), Vec::new(), Vec::new(), Vec::new()],
            has_exits: true,
            compatible_with: [Vec::new(), Vec::new(), Vec::new(), Vec::new()],
        };
        for exit in new_chunk.exits.iter_mut() {
            for _i in 0..chunk_size {
                exit.push(false);
            }
        }

        let mut number_of_exits = 0;
        for x in 0..chunk_size {
            // Check north
            let north_idx = tile_idx_in_chunk(chunk_size, x, 0);
            if new_chunk.pattern[north_idx] == TileType::Floor {
                new_chunk.exits[0][x as usize] = true;
                number_of_exits += 1;
            }
            // Check south
            let south_idx = tile_idx_in_chunk(chunk_size, x, chunk_size - 1);
            if new_chunk.pattern[south_idx] == TileType::Floor {
                new_chunk.exits[1][x as usize] = true;
                number_of_exits += 1;
            }
            // Check west
            let west_idx = tile_idx_in_chunk(chunk_size, 0, x);
            if new_chunk.pattern[west_idx] == TileType::Floor {
                new_chunk.exits[2][x as usize] = true;
                number_of_exits += 1;
            }
            // Check east
            let east_idx = tile_idx_in_chunk(chunk_size, chunk_size - 1, x);
            if new_chunk.pattern[east_idx] == TileType::Floor {
                new_chunk.exits[3][x as usize] = true;
                number_of_exits += 1;
            }
        }

        if number_of_exits == 0 {
            new_chunk.has_exits = false;
        }
        constraints.push(new_chunk);
    }

    // Build compatibility matrix
    let ch = constraints.clone();
    for constraint in constraints.iter_mut() {
        for (j, potential) in ch.iter().enumerate() {
            // If no exits at all, it's compatible
            if !constraint.has_exits || !potential.has_exits {
                for compatible in constraint.compatible_with.iter_mut() {
                    compatible.push(j);
                }
            } else {
                // Evaluate compatibility by dir
                for (direction, exit_list) in constraint.exits.iter_mut().enumerate() {
                    let opposite = match direction {
                        0 => 1, // North-South
                        1 => 0, // South-North
                        2 => 3, // West-East
                        _ => 2, // East-West
                    };

                    let mut it_fits = false;
                    let mut has_any = false;
                    for (slot, can_enter) in exit_list.iter().enumerate() {
                        if *can_enter {
                            has_any = true;
                            if potential.exits[opposite][slot] {
                                it_fits = true;
                            }
                        }
                    }
                    if it_fits {
                        constraint.compatible_with[direction].push(j);
                    }
                    if !has_any {
                        // No exits, match only if other edge also has none
                        let matching_exit_count = potential.exits[opposite].iter().filter(|a| !**a).count();
                        if matching_exit_count == 0 {
                            constraint.compatible_with[direction].push(j);
                        }
                    }
                }
            }
        }
    }

    return constraints;
}
