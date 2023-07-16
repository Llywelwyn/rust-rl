use super::{
    generate_voronoi_spawn_regions, remove_unreachable_areas_returning_most_distant, spawner, Map, MapBuilder,
    Position, TileType, SHOW_MAPGEN,
};
use rltk::RandomNumberGenerator;
use specs::prelude::*;
use std::collections::HashMap;

pub struct MazeBuilder {
    map: Map,
    starting_position: Position,
    depth: i32,
    history: Vec<Map>,
    noise_areas: HashMap<i32, Vec<usize>>,
}

impl MapBuilder for MazeBuilder {
    fn build_map(&mut self, rng: &mut RandomNumberGenerator) {
        return self.build(rng);
    }
    fn spawn_entities(&mut self, ecs: &mut World) {
        for area in self.noise_areas.iter() {
            spawner::spawn_region(ecs, area.1, self.depth);
        }
    }
    //  Getters
    fn get_map(&mut self) -> Map {
        return self.map.clone();
    }
    fn get_starting_pos(&mut self) -> Position {
        return self.starting_position.clone();
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

impl MazeBuilder {
    pub fn new(new_depth: i32) -> MazeBuilder {
        MazeBuilder {
            map: Map::new(new_depth),
            starting_position: Position { x: 0, y: 0 },
            depth: new_depth,
            history: Vec::new(),
            noise_areas: HashMap::new(),
        }
    }

    #[allow(clippy::map_entry)]
    fn build(&mut self, rng: &mut RandomNumberGenerator) {
        // Maze gen
        let mut maze = Grid::new((self.map.width / 2) - 2, (self.map.height / 2) - 2, rng);
        maze.generate_maze(self);

        // Starting point @ top left
        self.starting_position = Position { x: 2, y: 2 };
        let start_idx = self.map.xy_idx(self.starting_position.x, self.starting_position.y);
        self.take_snapshot();
        // Find all tiles we can reach from the starting point
        let exit_tile = remove_unreachable_areas_returning_most_distant(&mut self.map, start_idx);
        self.take_snapshot();
        // Place the stairs
        self.map.tiles[exit_tile] = TileType::DownStair;
        self.take_snapshot();

        // Now we build a noise map for use in spawning entities later
        self.noise_areas = generate_voronoi_spawn_regions(&self.map, rng);
    }
}

struct Grid<'a> {
    width: i32,
    height: i32,
    cells: Vec<Cell>,
    backtrace: Vec<usize>,
    current: usize,
    rng: &'a mut RandomNumberGenerator,
}

impl<'a> Grid<'a> {
    fn new(width: i32, height: i32, rng: &mut RandomNumberGenerator) -> Grid {
        let mut grid = Grid { width, height, cells: Vec::new(), backtrace: Vec::new(), current: 0, rng };

        for row in 0..height {
            for column in 0..width {
                grid.cells.push(Cell::new(row, column))
            }
        }

        return grid;
    }

    fn calculate_index(&self, row: i32, column: i32) -> i32 {
        if row < 0 || column < 0 || column > self.width - 1 || row > self.height - 1 {
            -1
        } else {
            column + (row * self.width)
        }
    }

    fn get_available_neighbours(&self) -> Vec<usize> {
        let mut neighbours: Vec<usize> = Vec::new();
        let current_row = self.cells[self.current].row;
        let current_column = self.cells[self.current].column;

        let neighbour_indices: [i32; 4] = [
            self.calculate_index(current_row - 1, current_column),
            self.calculate_index(current_row, current_column + 1),
            self.calculate_index(current_row + 1, current_column),
            self.calculate_index(current_row, current_column - 1),
        ];

        for i in neighbour_indices.iter() {
            if *i != -1 && !self.cells[*i as usize].visited {
                neighbours.push(*i as usize);
            }
        }

        return neighbours;
    }

    fn find_next_cell(&mut self) -> Option<usize> {
        let neighbours = self.get_available_neighbours();
        if neighbours.is_empty() {
            return None;
        }
        if neighbours.len() == 1 {
            return Some(neighbours[0]);
        }
        return Some(neighbours[(self.rng.roll_dice(1, neighbours.len() as i32) - 1) as usize]);
    }

    fn generate_maze(&mut self, generator: &mut MazeBuilder) {
        let mut i = 0;
        loop {
            self.cells[self.current].visited = true;
            let next = self.find_next_cell();

            match next {
                Some(next) => {
                    self.cells[next].visited = true;
                    self.backtrace.push(self.current);
                    let (lower_part, higher_part) = self.cells.split_at_mut(std::cmp::max(self.current, next));
                    let cell1 = &mut lower_part[std::cmp::min(self.current, next)];
                    let cell2 = &mut higher_part[0];
                    cell1.remove_walls(cell2);
                    self.current = next;
                }
                None => {
                    if self.backtrace.is_empty() {
                        break;
                    }
                    self.current = self.backtrace[0];
                    self.backtrace.remove(0);
                }
            }
            if i % 50 == 0 {
                self.copy_to_map(&mut generator.map);
                generator.take_snapshot();
            }
            i += 1;
        }
    }

    fn copy_to_map(&self, map: &mut Map) {
        // Clear map
        for i in map.tiles.iter_mut() {
            *i = TileType::Wall;
        }

        for cell in self.cells.iter() {
            let x = cell.column + 1;
            let y = cell.row + 1;
            let idx = map.xy_idx(x * 2, y * 2);

            map.tiles[idx] = TileType::Floor;
            if !cell.walls[TOP] {
                map.tiles[idx - map.width as usize] = TileType::Floor
            }
            if !cell.walls[RIGHT] {
                map.tiles[idx + 1] = TileType::Floor
            }
            if !cell.walls[BOTTOM] {
                map.tiles[idx + map.width as usize] = TileType::Floor
            }
            if !cell.walls[LEFT] {
                map.tiles[idx - 1] = TileType::Floor
            }
        }
    }
}

const TOP: usize = 0;
const RIGHT: usize = 1;
const BOTTOM: usize = 2;
const LEFT: usize = 3;

#[derive(Copy, Clone)]
struct Cell {
    row: i32,
    column: i32,
    walls: [bool; 4],
    visited: bool,
}

impl Cell {
    fn new(row: i32, column: i32) -> Cell {
        Cell { row, column, walls: [true, true, true, true], visited: false }
    }

    fn remove_walls(&mut self, next: &mut Cell) {
        let x = self.column - next.column;
        let y = self.row - next.row;

        if x == 1 {
            self.walls[LEFT] = false;
            next.walls[RIGHT] = false;
        } else if x == -1 {
            self.walls[RIGHT] = false;
            next.walls[LEFT] = false;
        } else if y == 1 {
            self.walls[TOP] = false;
            next.walls[BOTTOM] = false
        } else if y == -1 {
            self.walls[BOTTOM] = false;
            next.walls[TOP] = false;
        }
    }
}
