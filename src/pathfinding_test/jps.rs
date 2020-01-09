// https://github.com/mikolalysenko/l1-path-finder

// https://en.wikipedia.org/wiki/Jump_point_search

use std::collections::{BinaryHeap, HashMap, HashSet};

use pyo3::ffi::Py_IsInitialized;
use std::cmp::Ordering;
use std::f32::consts::SQRT_2;
use std::f32::EPSILON;
use std::ops::Sub;

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
    return dx as f32 + dy as f32 + SQRT_2_MINUS_2 * min as f32;
}

fn euclidean_heuristic(source: Point2d, target: Point2d) -> f32 {
    let x = source.x - target.x;
    let xx = x * x;
    let y = source.y - target.y;
    let yy = y * y;
    let sum = xx + yy;
    ((xx + yy) as f32).sqrt()
}

fn no_heuristic(source: Point2d, target: Point2d) -> f32 {
    0.0
}

struct Direction {
    x: i32,
    y: i32,
}

impl Direction {
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
            // TODO
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
            // TODO
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
}

#[derive(Hash, Eq, PartialEq, Copy, Clone)]
struct Point2d {
    x: usize,
    y: usize,
}

impl Point2d {
    fn add(&self, other: &Self) -> Point2d {
        Point2d {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
    fn add_tuple(&self, other: &(i32, i32)) -> Point2d {
        Point2d {
            x: (self.x as i32 + other.0) as usize,
            y: (self.y as i32 + other.1) as usize,
        }
    }
    fn add_direction(&self, other: &Direction) -> Point2d {
        Point2d {
            x: (self.x as i32 + other.x) as usize,
            y: (self.y as i32 + other.y) as usize,
        }
    }
    fn is_in_grid(&self, grid: Vec<Vec<u8>>) -> bool {
        grid[self.y][self.x] == 1
    }
}

struct JumpPoint {
    start: Point2d,
    direction: Direction,
    cost_to_start: f32,
    total_cost_estimate: f32,
    left_is_blocked: bool,
    right_is_blocked: bool,
}

impl PartialEq for JumpPoint {
    fn eq(&self, other: &Self) -> bool {
        self.total_cost_estimate - other.total_cost_estimate < EPSILON
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

#[derive(Default)]
struct PathFinder {
    grid: Vec<Vec<u8>>,
    heuristic: String,
    jump_points: BinaryHeap<JumpPoint>,
    came_from: HashMap<Point2d, Point2d>,
}

impl PathFinder {
    fn traverse(
        &mut self,
        start: Point2d,
        direction: &Direction,
        target: &Point2d,
        cost_to_start: f32,
        left_is_blocked: bool,
        right_is_blocked: bool,
    ) {
        let traversed_count: u32 = 0;
        let mut is_diagonal = false;
        match direction {
            Direction { x: 0, y: 1 }
            | Direction { x: 1, y: 0 }
            | Direction { x: -1, y: 0 }
            | Direction { x: 0, y: -1 } => (),
            _ => is_diagonal = true,
        }
        let (mut left_blocked, mut right_blocked) = (false, false);
        // While the path ahead is not blocked: travserse
        while let Some(new_point) = self.new_point_in_grid(&start, &direction) {
            if is_diagonal {
            } else {
                match left_blocked {
                    // Left node was blocked before, but is now no longer blocked -> mark as new starting point to traverse from, with a left rotation of 45 degrees
                    true => {
                        if let Some(point) = self.new_point_in_grid(&new_point, &direction.left()) {
                            left_blocked = false;
                            let new_cost_to_start = if is_diagonal {
                                traversed_count as f32 * SQRT_2 + cost_to_start
                            } else {
                                traversed_count as f32
                            };
                            let new_cost_estimate = octal_heuristic(point, *target);
                            self.jump_points.push(JumpPoint {
                                start: point,
                                direction: direction.half_left(),
                                cost_to_start: new_cost_to_start,
                                total_cost_estimate: new_cost_to_start + new_cost_estimate,
                                left_is_blocked: true,
                                right_is_blocked: false,
                            })
                        }
                    }
                    // Left node wasn't blocked before, but is now blocked
                    false => {
                        if let Some(p) = self.new_point_in_grid(&new_point, &direction.left()) {
                        } else {
                            left_blocked = true;
                        }
                    }
                }

                match right_blocked {
                    // Left node was blocked before, but is now no longer blocked -> mark as new starting point to traverse from, with a left rotation of 45 degrees
                    true => {
                        if let Some(point) = self.new_point_in_grid(&new_point, &direction.right())
                        {
                            right_blocked = false;
                            let new_cost_to_start = if is_diagonal {
                                traversed_count as f32 * SQRT_2 + cost_to_start
                            } else {
                                traversed_count as f32
                            };
                            let new_cost_estimate = octal_heuristic(point, *target);
                            self.jump_points.push(JumpPoint {
                                start: point,
                                direction: direction.half_left(),
                                cost_to_start: new_cost_to_start,
                                total_cost_estimate: new_cost_to_start + new_cost_estimate,
                                left_is_blocked: false,
                                right_is_blocked: true,
                            })
                        }
                    }
                    // Left node wasn't blocked before, but is now blocked
                    false => {
                        if let Some(p) = self.new_point_in_grid(&new_point, &direction.right()) {
                        } else {
                            right_blocked = true;
                        }
                    }
                }
            }
        }
    }

    fn is_in_grid(&self, point: Point2d) -> bool {
        self.grid[point.y][point.x] == 1
    }
    fn new_point_in_grid(&self, point: &Point2d, direction: &Direction) -> Option<Point2d> {
        // Returns new point if point in that direction is not blocked
        let new_point = point.add_direction(&direction);
        if self.is_in_grid(new_point) {
            return Some(new_point);
        }
        None
    }

    fn find_path(&mut self, source: &Point2d, target: &Point2d) -> Option<Vec<(usize, usize)>> {
        // Return early when start is in the wall
        if self.grid[source.y][source.x] == 0 {
            return None;
        }

        let mut jump_points = BinaryHeap::new();
        let mut visited = HashSet::new();
        visited.insert(source);

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
            let new_node = source.add_tuple(n);
            let estimate = octal_heuristic(new_node, *target);
            self.came_from.insert(new_node, *source);
            let dir = Direction { x: n.0, y: n.1 };
            let left_blocked = self.new_point_in_grid(source, &dir.left()).is_none();
            let right_blocked = self.new_point_in_grid(source, &dir.right()).is_none();
            jump_points.push(JumpPoint {
                start: new_node,
                direction: dir,
                cost_to_start: cost,
                total_cost_estimate: cost + estimate,
                left_is_blocked: left_blocked,
                right_is_blocked: right_blocked,
            });
        }

        while let Some(JumpPoint {
            start,
            direction,
            cost_to_start,
            total_cost_estimate,
            left_is_blocked,
            right_is_blocked,
        }) = jump_points.pop()
        {
            if !self.is_in_grid(start) {
                continue;
            }

            self.traverse(
                start,
                &direction,
                &target,
                cost_to_start,
                left_is_blocked,
                right_is_blocked,
            );
        }

        None
    }
}

static SOURCE: Point2d = Point2d { x: 5, y: 5 };
static TARGET: Point2d = Point2d { x: 50, y: 90 };

#[cfg(test)] // Only compiles when running tests
mod tests {
    use super::*;
    #[allow(unused_imports)]
    use test::Bencher;

    fn jps_test() {
        // https://stackoverflow.com/a/59043086/10882657
        // Width and height can be unknown at compile time
        let width = 100;
        let height = 100;
        let mut grid = vec![vec![1; width]; height];

        // Width and height must be known at compile time
        const WIDTH: usize = 10;
        const HEIGHT: usize = 10;
        let mut array = [[1; WIDTH]; HEIGHT];

        // Set boundaries
        for y in 0..HEIGHT {
            grid[y][0] = 0;
            grid[y][WIDTH - 1] = 0;
            array[y][0] = 0;
            array[y][WIDTH - 1] = 0;
        }
        for x in 0..WIDTH {
            grid[0][x] = 0;
            grid[HEIGHT - 1][x] = 0;
            array[0][x] = 0;
            array[HEIGHT - 1][x] = 0;
        }

        let mut pf = PathFinder::default();
        pf.heuristic = String::from("octal");
        pf.grid = grid;

        println!("{:?}", pf.grid);
        println!("{:?}", array);
        let path = pf.find_path(&SOURCE, &TARGET);
        //        println!("RESULT {:?}", path);
    }

    //     This will only be executed when using "cargo test" and not "cargo bench"
    //        #[test]
    //        fn test_path() {
    //            let pf = PathFinder {
    //                allow_diagonal: true,
    //                heuristic: String::from("octal"),
    //                grid: vec![vec![]],
    //            };
    //
    //            let source = Point2d { x: 0, y: 0 };
    //            let target = Point2d { x: 5, y: 10 };
    //
    //            let path = pf.find_path(source, target);
    //    //        println!("RESULT mine {:?}", path);
    //        }

    #[bench]
    fn bench_jps_test(b: &mut Bencher) {
        b.iter(|| jps_test());
    }
}
