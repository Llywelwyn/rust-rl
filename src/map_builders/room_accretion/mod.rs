use super::{ BuilderMap, Map, InitialMapBuilder, TileType, Point };
use bracket_lib::prelude::*;

mod consts;
use consts::*;

/// Room Accretion map builder.
pub struct RoomAccretionBuilder {}

impl InitialMapBuilder for RoomAccretionBuilder {
    #[allow(dead_code)]
    fn build_map(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        self.build(rng, build_data);
    }
}

impl RoomAccretionBuilder {
    /// Constructor for Room Accretion.
    pub fn new() -> Box<RoomAccretionBuilder> {
        Box::new(RoomAccretionBuilder {})
    }

    fn build(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        //
    }
}

fn grid_with_dimensions(h: usize, w: usize, value: i32) -> Vec<Vec<i32>> {
    let mut grid = Vec::with_capacity(h);
    for _ in 0..h {
        let row = vec![value; w];
        grid.push(row);
    }
    grid
}

fn in_bounds(x: i32, y: i32, build_data: &BuilderMap) -> bool {
    x > 0 && x < build_data.height && y > 0 && y < build_data.width
}

fn draw_continuous_shape_on_grid(
    room: &Vec<Vec<i32>>,
    top_offset: usize,
    left_offset: usize,
    grid: &mut Vec<Vec<i32>>
) {
    for row in 0..room.len() {
        for col in 0..room[0].len() {
            if room[row][col] != 0 {
                let target_row = row + top_offset;
                let target_col = col + left_offset;
                if target_row < grid.len() && target_col < grid[0].len() {
                    grid[target_row][target_col] = room[row][col];
                }
            }
        }
    }
}

struct Coordinate {
    pub location: Point,
    pub value: i32,
}

fn draw_individual_coordinates_on_grid(coordinates: &Vec<Coordinate>, grid: &mut Vec<Vec<i32>>) {
    for c in coordinates {
        let x = c.location.x as usize;
        let y = c.location.y as usize;
        if y < grid.len() && x < grid[0].len() {
            grid[y][x] = c.value;
        }
    }
}

fn get_cell_neighbours(
    cells: &Vec<Vec<i32>>,
    row: usize,
    col: usize,
    h: usize,
    w: usize
) -> Vec<i32> {
    let mut neighbours = Vec::new();
    for x in row.saturating_sub(1)..=std::cmp::min(row + 1, h - 1) {
        for y in col.saturating_sub(1)..=std::cmp::min(col + 1, w - 1) {
            if x != row || y != col {
                neighbours.push(cells[x][y]);
            }
        }
    }
    neighbours
}

fn make_ca_room(rng: &mut RandomNumberGenerator) -> Vec<Vec<i32>> {
    let width = rng.range(5, 10);
    let height = rng.range(5, 10);
    let mut cells = grid_with_dimensions(height, width, 0);
    cells = cells
        .into_iter()
        .map(|row| {
            row.into_iter()
                .map(|_| if rng.roll_dice(1, 2) == 1 { 1 } else { 0 })
                .collect()
        })
        .collect();

    let transform_cell = |state: i32, neighbours: &Vec<i32>| -> i32 {
        let rules: &[CellRules] = &CA[state as usize];
        let mut new_state = state;
        for rule in rules {
            let n_neighbours = neighbours
                .iter()
                .filter(|&&neighbour| neighbour == rule.adjacent_type)
                .count();
            if rule.operator.eval(n_neighbours as i32, rule.n) {
                new_state = rule.into;
            }
        }
        new_state
    };

    for _ in 0..5 {
        let mut new_cells = vec![vec![0; width]; height];
        for row in 0..height {
            for col in 0..height {
                let neighbours = get_cell_neighbours(&cells, row, col, height, width);
                let new_state = transform_cell(cells[row][col], &neighbours);
                new_cells[row][col] = new_state;
            }
        }
        cells = new_cells;
    }

    cells
}

fn direction_of_door(
    grid: &Vec<Vec<i32>>,
    row: usize,
    col: usize,
    build_data: &BuilderMap
) -> Direction {
    if grid[row][col] != 0 {
        return Direction::NoDir;
    }
    let mut solution = Direction::NoDir;
    let mut dir_iter = DirectionIterator::new();
    for dir in &mut dir_iter {
        let new_col = (col as i32) + dir.transform().x;
        let new_row = (row as i32) + dir.transform().y;
        let opp_col = (col as i32) - dir.transform().x;
        let opp_row = (row as i32) - dir.transform().y;
        if
            in_bounds(new_col, new_row, &build_data) &&
            in_bounds(new_col, new_row, &build_data) &&
            grid[opp_row as usize][opp_col as usize] != 0
        {
            solution = dir;
        }
    }
    return solution;
}

#[derive(Copy, Clone, PartialEq)]
pub struct DoorSite {
    pub x: i32,
    pub y: i32,
    pub dir: Direction,
}

fn choose_random_door_site(
    room: Vec<Vec<i32>>,
    rng: &mut RandomNumberGenerator,
    build_data: &BuilderMap
) -> Vec<DoorSite> {
    let mut grid = grid_with_dimensions(HEIGHT, WIDTH, 0);
    let mut door_sites: Vec<DoorSite> = Vec::new();
    const LEFT_OFFSET: usize = ((WIDTH as f32) / 2.0) as usize;
    const TOP_OFFSET: usize = ((HEIGHT as f32) / 2.0) as usize;
    draw_continuous_shape_on_grid(&room, TOP_OFFSET, LEFT_OFFSET, &mut grid);
    for row in 0..HEIGHT {
        for col in 0..WIDTH {
            if grid[row][col] == 0 {
                let door_dir = direction_of_door(&grid, row, col, &build_data);
                if door_dir == Direction::NoDir {
                    continue;
                }
                let mut door_failed = false;
                let (mut trace_row, mut trace_col) = (
                    (row as i32) + door_dir.transform().y,
                    (col as i32) + door_dir.transform().x,
                );
                let mut i = 0;
                while i < 10 && in_bounds(trace_row, trace_col, &build_data) && !door_failed {
                    if grid[trace_row as usize][trace_col as usize] != 0 {
                        door_failed = true;
                    }
                    trace_col += door_dir.transform().x;
                    trace_row += door_dir.transform().y;
                    i += 1;
                }
                if !door_failed {
                    // May need more information here.
                    door_sites.push(DoorSite {
                        x: col as i32,
                        y: row as i32,
                        dir: door_dir,
                    });
                }
            }
        }
    }
    let mut chosen_doors: Vec<DoorSite> = Vec::new();
    let mut dir_iter = DirectionIterator::new();
    for dir in &mut dir_iter {
        let doors_facing_this_dir: Vec<&DoorSite> = door_sites
            .iter()
            .filter(|&door| door.dir == dir)
            .collect();
        if !doors_facing_this_dir.is_empty() {
            let index = rng.range(0, doors_facing_this_dir.len());
            chosen_doors.push(*doors_facing_this_dir[index]);
        }
    }
    chosen_doors
}
