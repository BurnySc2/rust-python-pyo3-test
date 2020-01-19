// https://github.com/mikolalysenko/l1-path-finder

// https://en.wikipedia.org/wiki/Jump_point_search
use std::collections::{BinaryHeap, HashMap};

use std::cmp::Ordering;
use std::f32::consts::SQRT_2;
use std::f32::EPSILON;
use std::ops::Sub;

use ndarray::Array;
use ndarray::Array2;

use fnv::FnvHashMap;
use fnv::FnvHasher;

use pyo3::prelude::*;
use pyo3::types::PyAny;
use pyo3::wrap_pyfunction;
use pyo3::PyObjectProtocol;

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

#[pyclass]
#[derive(Debug, Hash, Eq, PartialEq, Copy, Clone)]
pub struct Point2d {
    pub x: usize,
    pub y: usize,
}

impl Point2d {
    fn add_direction(&self, other: Direction) -> Point2d {
        Point2d {
            x: (self.x as i32 + other.x) as usize,
            y: (self.y as i32 + other.y) as usize,
        }
    }

    fn get_direction(&self, target: &Point2d) -> Direction {
        let x: i32;
        let y: i32;
        match self.x.cmp(&target.x) {
            Ordering::Greater => x = -1,
            Ordering::Less => x = 1,
            Ordering::Equal => x = 0,
        }
        match self.y.cmp(&target.y) {
            Ordering::Greater => y = -1,
            Ordering::Less => y = 1,
            Ordering::Equal => y = 0,
        }
        Direction { x, y }
    }
}

impl<'source> FromPyObject<'source> for Point2d {
    fn extract(ob: &'source PyAny) -> PyResult<Point2d> {
        unsafe {
            let py = Python::assume_gil_acquired();
            let obj = ob.to_object(py);
            Ok(Self {
                x: obj.getattr(py, "x")?.extract(py)?,
                y: obj.getattr(py, "y")?.extract(py)?,
            })
        }
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

#[pyclass]
pub struct PathFinder {
    grid: Array2<u8>,
    heuristic: String,
    jump_points: BinaryHeap<JumpPoint>,
    // Contains points which were already visited
    came_from: FnvHashMap<Point2d, Point2d>,
}

//#[pymethods]
//impl PathFinder {
//    #[new]
//    fn new(
//        obj: &PyRawObject,
//        grid_: Array2<u8>,
//        heuristic_: String,
//        jump_points_: BinaryHeap<JumpPoint>,
//        came_from_: FnvHashMap<Point2d, Point2d>,
//    ) {
//        obj.init(PathFinder {
//            grid: grid_,
//            heuristic: heuristic_,
//            jump_points: jump_points_,
//            came_from: came_from_,
//        })
//    }
//}

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
        let add_nodes: Vec<(Direction, Direction)> = if direction.is_diagonal() {
            // The first two entries will be checked for left_blocked and right_blocked, if a wall was encountered but that position is now free (forced neighbors?)
            // If the vec has more than 2 elements, then the remaining will not be checked for walls (this is the case in diagonal movement where it forks off to horizontal+vertical movement)
            // (blocked_direction from current_node, traversal_direction)
            let (half_left, half_right) = (direction.half_left(), direction.half_right());
            vec![
                (direction.left135(), direction.left()),
                (direction.right135(), direction.right()),
                (half_left, half_left),
                (half_right, half_right),
            ]
        } else {
            vec![
                (direction.left(), direction.half_left()),
                (direction.right(), direction.half_right()),
            ]
        };
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
                return;
            }
            // We loop over each direction that isnt the traversal direction
            // For diagonal traversal this is 2 checks (left is wall, right is wall), and 2 forks (horizontal+vertical movement)
            // For non-diagonal traversal this is only checking if there are walls on the side
            for (index, (check_dir, traversal_dir)) in add_nodes.iter().enumerate() {
                // Check if in that direction is a wall
                let check_point_is_in_grid =
                    self.is_in_grid(&current_point.add_direction(*check_dir));

                if (index == 0 && left_blocked || index == 1 && right_blocked || index > 1)
                    && traversed_count != 0
                    && check_point_is_in_grid
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

                        // Mark the side no longer as blocked
                        if index == 0 {
                            left_blocked = false;
                        } else {
                            right_blocked = false;
                        }
                    // If this is non-diagonal traversal, this is used to store a 'came_from' point
                    } else {
                        // If this is diagonal traversal, instantly traverse the non-diagonal directions without adding them to min-heap first
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
                } else if index == 0 && !check_point_is_in_grid {
                    // If this direction (left) has now a wall, mark as blocked
                    left_blocked = true;
                } else if index == 1 && !check_point_is_in_grid {
                    // If this direction (right) has now a wall, mark as blocked
                    right_blocked = true
                }
            }

            current_point = current_point.add_direction(direction);
            if !self.is_in_grid(&current_point) {
                // Next traversal point is a wall - this traversal is done
                break;
            }
            // Next traversal point is pathable
            traversed_count += 1;
        }
    }

    fn add_came_from(&mut self, p1: &Point2d, p2: &Point2d) -> bool {
        // Returns 'already_visited' boolean
        if !self.came_from.contains_key(p1) {
            self.came_from.insert(*p1, *p2);
            return false;
        }
        true
    }

    fn is_in_grid(&self, point: &Point2d) -> bool {
        self.grid[[point.y, point.x]] == 1
    }

    fn new_point_in_grid(&self, point: &Point2d, direction: Direction) -> Option<Point2d> {
        // Returns new point if point in that direction is not blocked
        let new_point = point.add_direction(direction);
        if self.is_in_grid(&new_point) {
            return Some(new_point);
        }
        None
    }

    fn goal_reached(&self, target: &Point2d) -> bool {
        self.came_from.contains_key(&target)
    }

    fn construct_path(
        &self,
        source: &Point2d,
        target: &Point2d,
        construct_full_path: bool,
    ) -> Option<Vec<Point2d>> {
        if construct_full_path {
            let mut path: Vec<Point2d> = Vec::with_capacity(100);
            let mut pos = *target;
            path.push(pos);
            while &pos != source {
                let temp_target = *self.came_from.get(&pos).unwrap();
                let dir = pos.get_direction(&temp_target);
                let mut temp_pos = pos.add_direction(dir);
                while temp_pos != temp_target {
                    path.push(temp_pos);
                    temp_pos = temp_pos.add_direction(dir);
                }
                pos = temp_target;
            }
            path.push(*source);
            path.reverse();
            Some(path)
        } else {
            let mut path: Vec<Point2d> = Vec::with_capacity(20);
            path.push(*target);
            let mut pos = self.came_from.get(target).unwrap();
            while pos != source {
                pos = self.came_from.get(&pos).unwrap();
                path.push(*pos);
            }
            path.reverse();
            Some(path)
        }
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
            let _left_blocked = self.new_point_in_grid(source, dir.left()).is_none();
            let _right_blocked = self.new_point_in_grid(source, dir.right()).is_none();
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
            ..
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

pub fn jps_pf(grid: Array2<u8>) -> PathFinder {
    PathFinder {
        grid,
        heuristic: String::from("octal"),
        jump_points: BinaryHeap::with_capacity(1000),
        came_from: FnvHashMap::default(),
    }
}

pub fn jps_test(pf: &mut PathFinder, source: &Point2d, target: &Point2d) -> Option<Vec<Point2d>> {
    pf.find_path(&source, &target)
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
        let (array, _height, _width) = result.unwrap();
        // Spawn to spawn
        let source = Point2d { x: 32, y: 51 };
        let target = Point2d { x: 150, y: 129 };

        // Main ramp to main ramp
        //                let source = Point2d { x: 32, y: 51 };
        //                let target = Point2d { x: 150, y: 129 };
        let mut pf = jps_pf(array);
        let path = jps_test(&mut pf, &source, &target);
        assert_ne!(None, path);
        assert!(path.unwrap().len() > 0);
        // Run bench
        b.iter(|| jps_test(&mut pf, &source, &target));
    }

    #[bench]
    fn bench_jps_test_from_file_no_path(b: &mut Bencher) {
        // Setup
        let result = read_grid_from_file(String::from("AutomatonLE.txt"));
        let (mut array, _height, _width) = result.unwrap();
        // Spawn to spawn
        let source = Point2d { x: 32, y: 51 };
        let target = Point2d { x: 150, y: 129 };

        // Block entrance to main base
        for x in 145..=150 {
            for y in 129..=135 {
                array[[y, x]] = 0;
            }
        }

        let mut pf = jps_pf(array);
        let path = jps_test(&mut pf, &source, &target);
        assert_eq!(None, path);
        // Run bench
        b.iter(|| jps_test(&mut pf, &source, &target));
    }

    #[bench]
    fn bench_jps_test(b: &mut Bencher) {
        let grid = grid_setup(30);
        let mut pf = jps_pf(grid);
        let source: Point2d = Point2d { x: 5, y: 5 };
        let target: Point2d = Point2d { x: 10, y: 12 };
        b.iter(|| jps_test(&mut pf, &source, &target));
    }
}
