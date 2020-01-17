// https://github.com/mikolalysenko/l1-path-finder

// https://en.wikipedia.org/wiki/Jump_point_search
use std::collections::{BinaryHeap, HashMap, HashSet};

use std::cmp::Ordering;
use std::f32::consts::SQRT_2;
use std::f32::EPSILON;
use std::ops::Sub;

use ndarray::Array;
use ndarray::Array1;
use ndarray::Array2;
//use ndarray::ArrayBase;

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

fn manhattan_heuristic(source: &Point2d, target: &Point2d) -> f32 {
    (absdiff(source.x, target.x) + absdiff(source.y, target.y)) as f32
}

static SQRT_2_MINUS_2: f32 = SQRT_2 - 2.0;

fn octal_heuristic(source: &Point2d, target: &Point2d) -> f32 {
    let dx = absdiff(source.x, target.x);
    let dy = absdiff(source.y, target.y);
    let min = std::cmp::min(dx, dy);
    dx as f32 + dy as f32 + SQRT_2_MINUS_2 * min as f32
}

fn euclidean_heuristic(source: &Point2d, target: &Point2d) -> f32 {
    let x = source.x as i32 - target.x as i32;
    let xx = x * x;
    let y = source.y as i32 - target.y as i32;
    let yy = y * y;
    let sum = xx + yy;
    (sum as f32).sqrt()
}

fn no_heuristic(_source: &Point2d, _target: &Point2d) -> f32 {
    0.0
}

#[derive(Debug, Copy, Clone, PartialEq)]
struct Direction {
    x: i32,
    y: i32,
}

impl Direction {
    fn to_value(self) -> u8 {
        match (self.x, self.y) {
            (1, 0) => 2,
            (0, 1) => 4,
            (-1, 0) => 6,
            (0, -1) => 8,
            // Diagonal
            (1, 1) => 3,
            (-1, 1) => 5,
            (-1, -1) => 7,
            (1, -1) => 9,
            _ => panic!("This shouldnt happen"),
        }
    }

    fn from_value(value: u8) -> Self {
        match value {
            2 => Direction { x: 1, y: 0 },
            4 => Direction { x: 0, y: 1 },
            6 => Direction { x: -1, y: 0 },
            8 => Direction { x: 0, y: -1 },
            // Diagonal
            3 => Direction { x: 1, y: 1 },
            5 => Direction { x: -1, y: 1 },
            7 => Direction { x: -1, y: -1 },
            9 => Direction { x: 1, y: -1 },
            _ => panic!("This shouldnt happen"),
        }
    }

    fn from_value_reverse(value: u8) -> Self {
        match value {
            6 => Direction { x: 1, y: 0 },
            8 => Direction { x: 0, y: 1 },
            2 => Direction { x: -1, y: 0 },
            4 => Direction { x: 0, y: -1 },
            // Diagonal
            7 => Direction { x: 1, y: 1 },
            9 => Direction { x: -1, y: 1 },
            3 => Direction { x: -1, y: -1 },
            5 => Direction { x: 1, y: -1 },
            _ => panic!("This shouldnt happen"),
        }
    }

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
    fn left(self) -> Direction {
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
    fn right(self) -> Direction {
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
    fn half_left(self) -> Direction {
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
    fn half_right(self) -> Direction {
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
    fn left135(self) -> Direction {
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

pub struct PathFinder {
    grid: Array2<u8>,
    heuristic: String,
    jump_points: BinaryHeap<JumpPoint>,
    // Contains points which were already visited
    came_from: HashMap<Point2d, Point2d>,
}

#[allow(dead_code)]
impl PathFinder {
    fn traverse(
        &mut self,
        start: &Point2d,
        direction: Direction,
        target: &Point2d,
        cost_to_start: f32,
        heuristic: fn(&Point2d, &Point2d) -> f32,
    ) {
        // How far we moved from the start of the function call
        let mut traversed_count: u32 = 0;
        let add_nodes: Vec<(Direction, Direction)>;
        if direction.is_diagonal() {
            // The first two entries will be checked for left_blocked and right_blocked, if a wall was encountered but that position is now free (forced neighbors?)
            // If the vec has more than 2 elements, then the remaining will not be checked for walls (this is the case in diagonal movement where it forks off to horizontal+vertical movement)
            // (blocked_direction from current_node, traversal_direction)
            let (half_left, half_right) = (direction.half_left(), direction.half_right());
            add_nodes = vec![
                (direction.left135(), direction.left()),
                (direction.right135(), direction.right()),
                (half_left, half_left),
                (half_right, half_right),
            ];
        } else {
            add_nodes = vec![
                (direction.left(), direction.half_left()),
                (direction.right(), direction.half_right()),
            ];
        }
        let mut current_point = *start;
        // Stores wall status - if a side is no longer blocked: create jump point and fork path
        let (mut left_blocked, mut right_blocked) = (false, false);
        loop {
            // Goal found, construct path
            if current_point == *target {
                self.add_came_from(&current_point, &start);
//                println!("Found goal: {:?} {:?}", current_point, direction);
//                println!("Size of open list: {:?}", self.jump_points.len());
//                println!("Size of came from: {:?}", self.came_from.len());
                return ();
            }
            // We loop over each direction that isnt the traversal direction
            // For diagonal traversal this is 2 checks (left is wall, right is wall), and 2 forks (horizontal+vertical movement)
            // For non-diagonal traversal this is only checking if there are walls on the side
            for (index, (check_dir, traversal_dir)) in add_nodes.iter().enumerate() {
                // Check if in that direction is a wall
                let check_point = self.new_point_in_grid(&current_point, *check_dir);
                if (index == 0 && left_blocked || index == 1 && right_blocked || index > 1)
                    && traversed_count > 0
                    && check_point.is_some()
                {
                    // If there is no longer a wall in that direction, add jump point to binary heap
                    let new_cost_to_start = if traversal_dir.is_diagonal() {
                        cost_to_start + SQRT_2 * traversed_count as f32
                    } else {
                        cost_to_start + traversed_count as f32
                    };

                    if index < 2 {
                        if self.add_came_from(&current_point, &start) {
                            // We were already at this point because a new jump point was created here - this means we either are going in a circle or we come from a path that is longer?
                            break;
                        }
                        // Add forced neighbor to min-heap
                        self.jump_points.push(JumpPoint {
                            start: current_point,
                            direction: *traversal_dir,
                            cost_to_start: new_cost_to_start,
                            total_cost_estimate: new_cost_to_start
                                + heuristic(&current_point, target),
                        });
                    // If this is non-diagonal traversal, this is used to store a 'came_from' point
                    } else {
                        // If this is diagonal traversal, instantly traverse the non-diagonal directions without adding them to min-heap first
                        //                        let next_point = current_point.add_direction(*traversal_dir);
                        self.traverse(
                            &current_point,
                            *traversal_dir,
                            target,
                            new_cost_to_start,
                            heuristic,
                        );
                        // The non-diagonal traversal created a jump point and added it to the min-heap, so to backtrack from target/goal, we need to add this position to 'came_from'
                        self.add_came_from(&current_point, &start);
                    }
                    // Mark the side no longer as blocked
                    if index == 0 {
                        left_blocked = false;
                    } else if index == 1 {
                        right_blocked = false;
                    }
                } else if index == 0 && check_point.is_none() {
                    // If this direction (left) has now a wall, mark as blocked
                    left_blocked = true;
                } else if index == 1 && check_point.is_none() {
                    // If this direction (right) has now a wall, mark as blocked
                    right_blocked = true
                }
            }

            if let Some(new_point) = self.new_point_in_grid(&current_point, direction) {
                // Next traversal point is pathable
                current_point = new_point;
                traversed_count += 1;
            } else {
                // Next traversal point is a wall - this traversal is done
                break;
            }
        }
    }

    fn add_came_from(&mut self, p: &Point2d, p2: &Point2d) -> bool {
        // Returns 'already_visited' boolean
        if !self.came_from.contains_key(p) {
            self.came_from.insert(*p, *p2);
            return false;
        }
        return true;
    }

    fn is_in_grid(&self, point: Point2d) -> bool {
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

    fn goal_reached(&self, target: &Point2d) -> bool {
        self.came_from.contains_key(&target)
    }

    fn get_direction(&self, source: &Point2d, target: &Point2d) -> Direction {
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
        source: &Point2d,
        target: &Point2d,
        construct_full_path: bool,
    ) -> Option<Vec<Point2d>> {
        let mut path = vec![];
        let mut pos = target;

        path.push(*target);
        while pos != source {
            pos = self.came_from.get(&pos).unwrap();
            path.push(*pos);
        }
        // TODO use variable construct_full_path

        path.reverse();
        Some(path)
    }

    fn find_path(&mut self, source: &Point2d, target: &Point2d) -> Option<Vec<Point2d>> {
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

        let heuristic: fn(&Point2d, &Point2d) -> f32;
        match self.heuristic.as_ref() {
            "manhattan" => heuristic = manhattan_heuristic,
            "octal" => heuristic = octal_heuristic,
            "euclidean" => heuristic = euclidean_heuristic,
            // Memory overflow!
            // "none" => heuristic = no_heuristic,
            _ => heuristic = euclidean_heuristic,
        }

        // Add 4 starting nodes (diagonal traversals) around source point
        for dir in [
            Direction { x: 1, y: 1 },
            Direction { x: -1, y: 1 },
            Direction { x: -1, y: -1 },
            Direction { x: 1, y: -1 },
        ]
        .iter()
        {
            let left_blocked = self.new_point_in_grid(source, dir.left()).is_none();
            let right_blocked = self.new_point_in_grid(source, dir.right()).is_none();
            self.jump_points.push(JumpPoint {
                start: *source,
                direction: *dir,
                cost_to_start: 0.0,
                total_cost_estimate: 0.0 + heuristic(&source, target),
            });
        }

        while let Some(JumpPoint {
            start,
            direction,
            cost_to_start,
            total_cost_estimate: _,
        }) = self.jump_points.pop()
        {
            if self.goal_reached(&target) {
                return self.construct_path(source, target, false);
            }

            self.traverse(&start, direction, &target, cost_to_start, heuristic);
        }

        None
    }
}

static SOURCE: Point2d = Point2d { x: 5, y: 5 };
static TARGET: Point2d = Point2d { x: 10, y: 12 };

pub fn jps_pf(grid: Array2<u8>) -> PathFinder {
    PathFinder {
        grid: grid,
        heuristic: String::from("octal"),
        jump_points: BinaryHeap::new(),
        came_from: HashMap::new(),
    }
}

pub fn jps_test(pf: &mut PathFinder, source: Point2d, target: Point2d) -> Option<Vec<Point2d>> {
    let path = pf.find_path(&source, &target);
    return path;
}

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

    let array = Array::from(my_vec).into_shape((height, width)).unwrap();
    Ok((array, height as u32, width as u32))
}

#[cfg(test)] // Only compiles when running tests
mod tests {
    use super::*;
    #[allow(unused_imports)]
    use test::Bencher;

    #[test]
    fn test_direction_to_value() {
        // Non diagonal
        assert_eq!(Direction { x: 1, y: 0 }.to_value(), 2);
        assert_eq!(Direction { x: 0, y: 1 }.to_value(), 4);
        assert_eq!(Direction { x: -1, y: 0 }.to_value(), 6);
        assert_eq!(Direction { x: 0, y: -1 }.to_value(), 8);
        // Diagonal
        assert_eq!(Direction { x: 1, y: 1 }.to_value(), 3);
        assert_eq!(Direction { x: -1, y: 1 }.to_value(), 5);
        assert_eq!(Direction { x: -1, y: -1 }.to_value(), 7);
        assert_eq!(Direction { x: 1, y: -1 }.to_value(), 9);
    }

    #[test]
    fn test_value_to_direction() {
        // Non diagonal
        assert_eq!(Direction::from_value(2), Direction { x: 1, y: 0 });
        assert_eq!(Direction::from_value(4), Direction { x: 0, y: 1 });
        assert_eq!(Direction::from_value(6), Direction { x: -1, y: 0 });
        assert_eq!(Direction::from_value(8), Direction { x: 0, y: -1 });
        // Diagonal
        assert_eq!(Direction::from_value(3), Direction { x: 1, y: 1 });
        assert_eq!(Direction::from_value(5), Direction { x: -1, y: 1 });
        assert_eq!(Direction::from_value(7), Direction { x: -1, y: -1 });
        assert_eq!(Direction::from_value(9), Direction { x: 1, y: -1 });
    }

    #[test]
    fn test_value_to_direction_rev() {
        // Non diagonal
        assert_eq!(Direction::from_value_reverse(2), Direction { x: -1, y: 0 });
        assert_eq!(Direction::from_value_reverse(4), Direction { x: 0, y: -1 });
        assert_eq!(Direction::from_value_reverse(6), Direction { x: 1, y: 0 });
        assert_eq!(Direction::from_value_reverse(8), Direction { x: 0, y: 1 });
        // Diagonal
        assert_eq!(Direction::from_value_reverse(3), Direction { x: -1, y: -1 });
        assert_eq!(Direction::from_value_reverse(5), Direction { x: 1, y: -1 });
        assert_eq!(Direction::from_value_reverse(7), Direction { x: 1, y: 1 });
        assert_eq!(Direction::from_value_reverse(9), Direction { x: -1, y: 1 });
    }

    #[bench]
    fn bench_jps_test_from_file(b: &mut Bencher) {
        // Setup
        let result = read_grid_from_file(String::from("AutomatonLE.txt"));
        let (array, height, width) = result.unwrap();
        // Spawn to spawn
        let source = Point2d { x: 32, y: 51 };
        let target = Point2d { x: 150, y: 129 };

        // Main ramp to main ramp
//                let source = Point2d { x: 32, y: 51 };
//                let target = Point2d { x: 150, y: 129 };
        let mut pf = jps_pf(array.clone());
        // Run bench
        b.iter(|| jps_test(&mut pf, source, target));
    }

    #[bench]
    fn bench_jps_test(b: &mut Bencher) {
        let grid = grid_setup(30);
        let mut pf = jps_pf(grid);
        b.iter(|| jps_test(&mut pf, SOURCE, TARGET));
    }
}
