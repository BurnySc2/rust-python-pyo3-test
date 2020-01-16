// https://github.com/mikolalysenko/l1-path-finder

// https://en.wikipedia.org/wiki/Jump_point_search
use std::collections::{BinaryHeap, HashMap, HashSet};

use std::cmp::Ordering;
use std::f32::consts::SQRT_2;
use std::f32::EPSILON;
use std::ops::Sub;

use ndarray::Array1;
use ndarray::Array2;
use ndarray::ArrayBase;

#[allow(dead_code)]
pub fn absdiff<T>(x: T, y: T) -> T
where
    T: Sub<Output = T> + PartialOrd,
{
    if x < y {
        y - x
    } else {
        x - y
    }
}

fn manhattan_heuristic(source: Point2d, target: Point2d) -> f32 {
    (absdiff(source.x, target.x) + absdiff(source.y, target.y)) as f32
}

static SQRT_2_MINUS_2: f32 = SQRT_2 - 2.0;

fn octal_heuristic(source: Point2d, target: Point2d) -> f32 {
    let dx = absdiff(source.x, target.x);
    let dy = absdiff(source.y, target.y);
    let min = std::cmp::min(dx, dy);
    dx as f32 + dy as f32 + SQRT_2_MINUS_2 * min as f32
}

fn euclidean_heuristic(source: Point2d, target: Point2d) -> f32 {
    let x = source.x as i32 - target.x as i32;
    let xx = x * x;
    let y = source.y as i32 - target.y as i32;
    let yy = y * y;
    let sum = xx + yy;
    (sum as f32).sqrt()
}

fn no_heuristic(_source: Point2d, _target: Point2d) -> f32 {
    0.0
}

#[derive(Debug, Copy, Clone)]
struct Direction {
    x: i32,
    y: i32,
}

impl Direction {
    fn is_diagonal(self) -> bool {
        match self {
            // Non diagonal movement
            Direction { x: 0, y: 1 }
            | Direction { x: 1, y: 0 }
            | Direction { x: -1, y: 0 }
            | Direction { x: 0, y: -1 } => false,
            _ => true,
        }
    }

    // 90 degree left turns
    fn left(&self) -> Direction {
        match (self.x, self.y) {
            (1, 0) => Direction { x: 0, y: 1 },
            (0, 1) => Direction { x: -1, y: 0 },
            (-1, 0) => Direction { x: 0, y: -1 },
            (0, -1) => Direction { x: 1, y: 0 },
            // Diagonal
            (1, 1) => Direction { x: -1, y: 1 },
            (-1, 1) => Direction { x: -1, y: -1 },
            (-1, -1) => Direction { x: 1, y: -1 },
            (1, -1) => Direction { x: 1, y: 1 },
            _ => panic!("This shouldnt happen"),
        }
    }

    // 90 degree right turns
    fn right(&self) -> Direction {
        match (self.x, self.y) {
            (1, 0) => Direction { x: 0, y: -1 },
            (0, 1) => Direction { x: 1, y: 0 },
            (-1, 0) => Direction { x: 0, y: 1 },
            (0, -1) => Direction { x: -1, y: 0 },
            // Diagonal
            (1, 1) => Direction { x: 1, y: -1 },
            (-1, 1) => Direction { x: 1, y: 1 },
            (-1, -1) => Direction { x: -1, y: 1 },
            (1, -1) => Direction { x: -1, y: -1 },
            _ => panic!("This shouldnt happen"),
        }
    }

    // 45 degree left turns
    fn half_left(&self) -> Direction {
        match (self.x, self.y) {
            (1, 0) => Direction { x: 1, y: 1 },
            (0, 1) => Direction { x: -1, y: 1 },
            (-1, 0) => Direction { x: -1, y: -1 },
            (0, -1) => Direction { x: 1, y: -1 },
            // Diagonal
            (1, 1) => Direction { x: 0, y: 1 },
            (-1, 1) => Direction { x: -1, y: 0 },
            (-1, -1) => Direction { x: 0, y: -1 },
            (1, -1) => Direction { x: 1, y: 0 },
            _ => panic!("This shouldnt happen"),
        }
    }

    // 45 degree right turns
    fn half_right(&self) -> Direction {
        match (self.x, self.y) {
            (1, 0) => Direction { x: 1, y: -1 },
            (0, 1) => Direction { x: 1, y: 1 },
            (-1, 0) => Direction { x: -1, y: 1 },
            (0, -1) => Direction { x: -1, y: -1 },
            // Diagonal
            (1, 1) => Direction { x: 1, y: 0 },
            (-1, 1) => Direction { x: 0, y: 1 },
            (-1, -1) => Direction { x: -1, y: 0 },
            (1, -1) => Direction { x: 0, y: -1 },
            _ => panic!("This shouldnt happen"),
        }
    }

    // 135 degree left turns
    fn left135(&self) -> Direction {
        match (self.x, self.y) {
            // Diagonal
            (1, 1) => Direction { x: -1, y: 0 },
            (-1, 1) => Direction { x: 0, y: -1 },
            (-1, -1) => Direction { x: 1, y: 0 },
            (1, -1) => Direction { x: 0, y: 1 },
            _ => panic!("This shouldnt happen"),
        }
    }
    // 135 degree right turns
    fn right135(self) -> Direction {
        match (self.x, self.y) {
            // Diagonal
            (1, 1) => Direction { x: 0, y: -1 },
            (-1, 1) => Direction { x: 1, y: 0 },
            (-1, -1) => Direction { x: 0, y: 1 },
            (1, -1) => Direction { x: -1, y: 0 },
            _ => panic!("This shouldnt happen"),
        }
    }
}

#[derive(Debug, Hash, Eq, PartialEq, Copy, Clone)]
pub struct Point2d {
    pub x: usize,
    pub y: usize,
}

impl Point2d {
    fn add(&self, other: &Self) -> Point2d {
        Point2d {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
    fn add_tuple(&self, other: (i32, i32)) -> Point2d {
        Point2d {
            x: (self.x as i32 + other.0) as usize,
            y: (self.y as i32 + other.1) as usize,
        }
    }
    fn add_direction(&self, other: Direction) -> Point2d {
        Point2d {
            x: (self.x as i32 + other.x) as usize,
            y: (self.y as i32 + other.y) as usize,
        }
    }
    fn is_in_grid(&self, grid: Vec<Vec<u8>>) -> bool {
        grid[self.y][self.x] == 1
    }
}

#[derive(Debug)]
struct JumpPoint {
    start: Point2d,
    direction: Direction,
    cost_to_start: f32,
    total_cost_estimate: f32,
    //    left_is_blocked: bool,
    //    right_is_blocked: bool,
}

impl PartialEq for JumpPoint {
    fn eq(&self, other: &Self) -> bool {
        absdiff(self.total_cost_estimate, other.total_cost_estimate) < EPSILON
    }
}

impl PartialOrd for JumpPoint {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        other
            .total_cost_estimate
            .partial_cmp(&self.total_cost_estimate)
    }
}

// The result of this implementation doesnt seem to matter - instead what matters, is that it is implemented
impl Ord for JumpPoint {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .total_cost_estimate
            .partial_cmp(&self.total_cost_estimate)
            .unwrap()
    }
}

impl Eq for JumpPoint {}

struct PathFinder {
    grid: Array2<u8>,
    heuristic: String,
    jump_points: BinaryHeap<JumpPoint>,
    // Contains points which were already visited
    came_from: HashMap<Point2d, Point2d>,
    checked: HashSet<Point2d>,
    duplicate_checks: u64,
}

#[allow(dead_code)]
impl PathFinder {
    fn traverse(
        &mut self,
        start: Point2d,
        direction: Direction,
        target: &Point2d,
        cost_to_start: f32,
        //        left_is_blocked: bool,
        //        right_is_blocked: bool,
        heuristic: fn(Point2d, Point2d) -> f32,
    ) {
        let mut traversed_count: u32 = 0;
        //        let mut is_diagonal = false;
        let add_nodes: Vec<(Direction, Direction)>;
        let is_diagonal: bool;
        match direction {
            // Non diagonal movement
            Direction { x: 0, y: 1 }
            | Direction { x: 1, y: 0 }
            | Direction { x: -1, y: 0 }
            | Direction { x: 0, y: -1 } => {
                is_diagonal = false;
                add_nodes = vec![
                    (direction.left(), direction.half_left()),
                    (direction.right(), direction.half_right()),
                ];
            }
            _ => {
                is_diagonal = true;
                // The first two entries will be checked for left_blocked and right_blocked (forced neighbors?)
                // If the vec has more than 2 elements, then the remaining will not be checked (this is the case in diagonal movement)
                // (blocked_direction from current_node, traversal_direction)
                add_nodes = vec![
                    (direction.left135(), direction.left()),
                    (direction.right135(), direction.right()),
                    (direction.half_left(), direction.half_left()),
                    (direction.half_right(), direction.half_right()),
                ];
            }
        }
        let mut current_point = start;
        let (mut left_blocked, mut right_blocked) = (false, false);
        loop {
            // Only executed for diagonal movement
            //            if traversed_count > 0 && self.checked.contains(&current_point) {
            //                self.duplicate_checks += 1;
            //            }
            //            self.checked.insert(current_point);

            for (index, (check_dir, traversal_dir)) in add_nodes.iter().enumerate() {
                let temp_point = self.new_point_in_grid(&current_point, *check_dir);
                if traversed_count > 0
                    && (index == 0 && left_blocked || index == 1 && right_blocked || index > 1)
                    && temp_point.is_some()
                {
                    let new_cost_to_start = if traversal_dir.is_diagonal() {
                        cost_to_start + SQRT_2 * traversed_count as f32
                    } else {
                        cost_to_start + traversed_count as f32
                    };
                    let new_total_cost_estimate =
                        new_cost_to_start + heuristic(current_point, *target);

                    self.jump_points.push(JumpPoint {
                        start: current_point,
                        direction: *traversal_dir,
                        cost_to_start: new_cost_to_start,
                        total_cost_estimate: new_cost_to_start + new_total_cost_estimate,
                    });
                    if traversed_count > 0 {
                        // Identical:
                        // if !self.came_from.contains_key(&current_point) {
                        //  self.came_from.insert(current_point, start);
                        // }
                        //            println!("Inserting {:?} : {:?}", current_point, start);
                        self.came_from.entry(current_point).or_insert(start);
                    }
                    if index == 0 {
                        left_blocked = false;
                    } else if index == 1 {
                        right_blocked = false;
                    }
                } else if index == 0 && !left_blocked && temp_point.is_none() {
                    left_blocked = true;
                } else if index == 1 && !right_blocked && temp_point.is_none() {
                    right_blocked = true
                }
            }

            if let Some(new_point) = self.new_point_in_grid(&current_point, direction) {
                // Next traversal point is pathable, but do nothing
                current_point = new_point;
                if current_point == *target {
                    //                    println!("Found goal: {:?}", current_point);
                    self.came_from.insert(*target, start);
                    break;
                }
                // If we were already in this point, don't traverse again
                //                                                if is_diagonal && self.came_from.contains_key(&current_point) {
                if self.came_from.contains_key(&current_point) {
                    break;
                }
                traversed_count += 1;
            } else {
                // Next traversal point is a wall
                break;
            }
        }
    }

    fn is_in_grid(&self, point: Point2d) -> bool {
        //        self.grid[point.y][point.x] == 1
        self.grid[[point.y, point.x]] == 1
    }
    fn new_point_in_grid(&self, point: &Point2d, direction: Direction) -> Option<Point2d> {
        // Returns new point if point in that direction is not blocked
        let new_point = point.add_direction(direction);
        if self.is_in_grid(new_point) {
            return Some(new_point);
        }
        None
    }

    fn get_direction(&self, source: Point2d, target: Point2d) -> Direction {
        let mut x = 0;
        let mut y = 0;
        if target.x < source.x {
            x = -1;
        } else if target.x > source.x {
            x = 1;
        }
        if target.y < source.y {
            y = -1;
        } else if target.y > source.y {
            y = 1;
        }
        Direction { x, y }
    }

    fn construct_path(
        &self,
        source: Point2d,
        target: Point2d,
        construct_full_path: bool,
    ) -> Option<Vec<Point2d>> {
        let mut path = vec![];
        let mut pos = self.came_from.get(&target)?;

        println!("Duplciate checks: {:?}", self.duplicate_checks);

        if construct_full_path {
            let mut old_pos = target;
            path.push(target);
            loop {
                let d = self.get_direction(old_pos, *pos);
                while *pos != old_pos {
                    old_pos = old_pos.add_direction(d);
                    path.push(old_pos);
                    //                    println!("{:?}", old_pos);
                }
                if old_pos == source {
                    break;
                }
                pos = self.came_from.get(pos)?;
            }
        } else {
            path.push(target);
            loop {
                path.push(*pos);
                //                println!("{:?}", pos);
                pos = self.came_from.get(pos)?;
                if *pos == source {
                    break;
                }
            }
            path.push(source);
        }
        path.reverse();
        Some(path)
    }

    fn find_path(&mut self, source: &Point2d, target: &Point2d) -> Option<Vec<Point2d>> {
        // Return early when start is in the wall
        //        if self.grid[source.y][source.x] == 0 {
        if self.grid[[source.y, source.x]] == 0 {
            println!(
                "Returning early, source position is not in grid: {:?}",
                source
            );
            return None;
        }
        if self.grid[[target.y, target.x]] == 0 {
            println!(
                "Returning early, target position is not in grid: {:?}",
                target
            );
            return None;
        }

        //        let mut jump_points = BinaryHeap::new();
        //        let mut visited = HashSet::new();
        //        visited.insert(source);

        let heuristic: fn(Point2d, Point2d) -> f32;
        match self.heuristic.as_ref() {
            "manhattan" => heuristic = manhattan_heuristic,
            "octal" => heuristic = octal_heuristic,
            "euclidean" => heuristic = euclidean_heuristic,
            "none" => heuristic = no_heuristic,
            _ => heuristic = euclidean_heuristic,
        }

        // Add 8 starting nodes around source point
        for (index, n) in vec![
            (1, 0),
            (0, 1),
            (-1, 0),
            (0, -1),
            (1, 1),
            (-1, 1),
            (-1, -1),
            (1, -1),
        ]
        .iter()
        .enumerate()
        {
            let cost = if index > 3 { SQRT_2 } else { 1.0 };
            let new_node = source.add_tuple(*n);
            let estimate = heuristic(new_node, *target);
            let dir = Direction { x: n.0, y: n.1 };
            let left_blocked = self.new_point_in_grid(source, dir.left()).is_none();
            let right_blocked = self.new_point_in_grid(source, dir.right()).is_none();
            self.jump_points.push(JumpPoint {
                start: *source,
                direction: dir,
                cost_to_start: 0.0,
                total_cost_estimate: cost + estimate,
                //                left_is_blocked: left_blocked,
                //                right_is_blocked: right_blocked,
            });
            //            self.came_from.insert(new_node, *source);
        }

        while let Some(JumpPoint {
            start,
            direction,
            cost_to_start,
            total_cost_estimate: _,
            //            left_is_blocked,
            //            right_is_blocked,
        }) = self.jump_points.pop()
        {
            if !self.is_in_grid(start) {
                continue;
            }

            self.traverse(
                start,
                direction,
                &target,
                cost_to_start,
                //                left_is_blocked,
                //                right_is_blocked,
                heuristic,
            );

            if self.came_from.contains_key(target) {
                return self.construct_path(*source, *target, false);
            }
        }

        None
    }
}

static SOURCE: Point2d = Point2d { x: 5, y: 5 };
static TARGET: Point2d = Point2d { x: 10, y: 12 };

//pub fn jps_test(grid: [[u8; 100]; 100]) {
//pub fn jps_test(grid: Vec<Vec<u8>>) {
pub fn jps_test(grid: Array2<u8>, source: Point2d, target: Point2d) -> Option<Vec<Point2d>> {
    let mut pf = PathFinder {
        grid,
        heuristic: String::from("octal"),
        jump_points: BinaryHeap::new(),
        came_from: HashMap::new(),
        checked: HashSet::new(),
        duplicate_checks: 0,
    };
    //    let mut pf = PathFinder::default();
    //    pf.heuristic = String::from("euclidean");
    //    pf.grid = grid.clone();

    let path = pf.find_path(&source, &target);
    return path;
}

//pub fn grid_setup(size: usize) ->  [[u8; 100]; 100] {
//pub fn grid_setup(size: usize) ->  Vec<Vec<u8>> {
pub fn grid_setup(size: usize) -> Array2<u8> {
    // https://stackoverflow.com/a/59043086/10882657
    let mut ndarray = Array2::<u8>::ones((size, size));
    // Set boundaries
    for y in 0..size {
        ndarray[[y, 0]] = 0;
        ndarray[[y, size - 1]] = 0;
    }
    for x in 0..size {
        ndarray[[0, x]] = 0;
        ndarray[[size - 1, x]] = 0;
    }
    ndarray
}

use std::fs::File;
use std::io::Read;
pub fn read_grid_from_file(path: String) -> Result<(Array2<u8>, u32, u32), std::io::Error> {
    let mut file = File::open(path)?;
    //    let mut data = Vec::new();
    let mut data = String::new();

    file.read_to_string(&mut data)?;
    let mut height = 0;
    let mut width = 0;
    // Create one dimensional vec
    let mut my_vec = Vec::new();
    for line in data.lines() {
        width = line.len();
        height += 1;
        for char in line.chars() {
            my_vec.push(char as u8 - 48);
        }
    }

    let array = ArrayBase::from(my_vec).into_shape((height, width)).unwrap();
    Ok((array, height as u32, width as u32))
}

fn vec_2d_setup() -> Vec<Vec<u8>> {
    let width = 100;
    let height = 100;
    let grid = vec![vec![1; width]; height];
    grid
}

fn vec_2d_index_test(my_vec: &Vec<Vec<u8>>) {
    for y in 0..100 {
        for x in 0..100 {
            my_vec[y][x];
        }
    }
}

fn array_2d_setup() -> [[u8; 100]; 100] {
    const WIDTH: usize = 100;
    const HEIGHT: usize = 100;
    let array = [[1u8; WIDTH]; HEIGHT];
    array
}

fn array_2d_index_test(my_vec: &[[u8; 100]; 100]) {
    for y in 0..100 {
        for x in 0..100 {
            my_vec[y][x];
        }
    }
}

fn ndarray_setup() -> Array2<u8> {
    let width = 100;
    let height = 100;
    let grid = Array2::<u8>::ones((width, height));
    grid
}

fn ndarray_index_test(my_vec: &Array2<u8>) {
    for y in 0..100 {
        for x in 0..100 {
            my_vec[[y, x]];
        }
    }
}

#[cfg(test)] // Only compiles when running tests
mod tests {
    use super::*;
    #[allow(unused_imports)]
    use test::Bencher;

    #[bench]
    fn bench_jps_test_from_file(b: &mut Bencher) {
        let result = read_grid_from_file(String::from("src/AutomatonLE.txt"));
        let (array, height, width) = result.unwrap();
        let source = Point2d { x: 32, y: 51 };
        let target = Point2d { x: 150, y: 129 };
        b.iter(|| jps_test(array.clone(), source, target));
    }

    #[bench]
    fn bench_jps_test(b: &mut Bencher) {
        let grid = grid_setup(100);
        b.iter(|| jps_test(grid.clone(), SOURCE, TARGET));
    }

    #[bench]
    fn bench_index_vec_vec_u8(b: &mut Bencher) {
        let grid = vec_2d_setup();
        b.iter(|| vec_2d_index_test(&grid));
    }

    #[bench]
    fn bench_index_array_u8(b: &mut Bencher) {
        let grid = array_2d_setup();
        b.iter(|| array_2d_index_test(&grid));
    }

    #[bench]
    fn bench_index_ndarray_u8(b: &mut Bencher) {
        let grid = ndarray_setup();
        b.iter(|| ndarray_index_test(&grid));
    }
}
