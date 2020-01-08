use pyo3::prelude::*;

use std::cmp::Ordering;
use std::collections::BinaryHeap;

use std::collections::HashMap;
use std::collections::HashSet;
use std::f32::consts::SQRT_2;
//use std::intrinsics::powf32;

use std::f32::EPSILON;

use pyo3::types::PyAny;
use test::convert_benchmarks_to_tests;

use indexmap::map::Entry::{Occupied, Vacant};
use indexmap::IndexMap;

use pathfinding::prelude::absdiff;
use pathfinding::prelude::astar;

#[pyclass]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Point2d {
    // For the .x and .y attributes to be accessable in python, it requires these macros
    //    #[pyo3(get, set)]
    x: i32,
    //    #[pyo3(get, set)]
    y: i32,
}

impl Point2d {
    fn add_neighbor(&self, other: (i32, i32)) -> Point2d {
        //        let (x, y) = other;
        Point2d {
            x: self.x + other.0,
            y: self.y + other.1,
        }
    }

    // These functions are just used for the pathfinding crate
    fn distance(&self, other: &Self) -> u32 {
        (absdiff(self.x, other.x) + absdiff(self.y, other.y)) as u32
    }

    fn successors(&self) -> Vec<(Point2d, u32)> {
        let x = self.x;
        let y = self.y;
        vec![
            Point2d { x: x + 1, y: y },
            Point2d { x: x - 1, y: y },
            Point2d { x: x, y: y + 1 },
            Point2d { x: x, y: y - 1 },
            Point2d { x: x + 1, y: y + 1 },
            Point2d { x: x + 1, y: y - 1 },
            Point2d { x: x - 1, y: y + 1 },
            Point2d { x: x - 1, y: y - 1 },
        ]
        .into_iter()
        .map(|p| (p, 1))
        .collect()
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

// https://doc.rust-lang.org/std/collections/binary_heap/
#[derive(Copy, Clone, Debug)]
struct Node {
    cost_to_source: f32,
    total_estimated_cost: f32,
    position: Point2d,
    came_from: Point2d,
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.total_estimated_cost - other.total_estimated_cost < EPSILON
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        other.total_estimated_cost.partial_cmp(&self.total_estimated_cost)
    }
}

// The result of this implementation doesnt seem to matter - instead what matters, is that it is implemented
impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        Ordering::Greater
    }
}

impl Eq for Node {}

fn manhattan_heuristic(source: Point2d, target: Point2d) -> f32 {
    (absdiff(source.x, target.x) + absdiff(source.y, target.y)) as f32
//    ((source.x - target.x).abs() + (source.y - target.y).abs()) as f32
}

fn octal_heuristic(source: Point2d, target: Point2d) -> f32 {
    let d = 1.0;
    let d2 = SQRT_2;
    let dx = (source.x - target.x).abs();
    let dy = (source.y - target.y).abs();
    let min = std::cmp::min(dx, dy);
    return d * (dx + dy) as f32 + (d2 - 2.0 * d) * min as f32;
}

fn euclidean_heuristic(source: Point2d, target: Point2d) -> f32 {
    let x = source.x - target.x;
    let xx = x * x;
    let y = source.y - target.y;
    let yy = y * y;
    let sum = xx + yy;
    ((xx + yy) as f32).sqrt()
}

fn construct_path(
    source: Point2d,
    target: Point2d,
    nodes_map: &HashMap<Point2d, Point2d>,
) -> Option<Vec<Point2d>> {
    let mut path = vec![];
    let mut pos = nodes_map.get(&target)?;
    loop {
        path.push(*pos);
        pos = nodes_map.get(&pos)?;
        if *pos == source {
            break;
        }
    }
    path.push(source);
    path.reverse();
    Some(path)
}

// https://doc.rust-lang.org/std/default/trait.Default.html
// https://stackoverflow.com/questions/19650265/is-there-a-faster-shorter-way-to-initialize-variables-in-a-rust-struct
//#[pyclass]
struct PathFinder {
    allow_diagonal: bool,
    heuristic: String,
    grid: Vec<Vec<i32>>,
}

// https://medium.com/@nicholas.w.swift/easy-a-star-pathfinding-7e6689c7f7b2
//#[pymethods]
impl PathFinder {
    //    #[new]
    //    fn new(obj: &PyRawObject, allow_diagonal_: bool, heuristic_: String, grid_: Vec<Vec<i32>>) {
    //        obj.init(PathFinder {
    //            allow_diagonal: allow_diagonal_,
    //            heuristic: heuristic_,
    //            grid: grid_,
    //        })
    //    }

    fn update_grid(&mut self, grid: Vec<Vec<i32>>) {
        self.grid = grid;
    }

    fn find_path(&self, source: Point2d, target: Point2d) -> Option<Vec<Point2d>> {
        let mut nodes_map = HashMap::new();
//        let mut came_from = HashMap::new();
        let mut closed_list = HashSet::new();

        // Add source
        let mut heap = BinaryHeap::new();
        heap.push(Node {
            cost_to_source: 0.0,
            total_estimated_cost: 0.0,
            position: source,
            came_from: source,
        });

        let neighbors;
        if self.allow_diagonal {
            neighbors = vec![
                ((0, 1), 1.0, 1),
                ((1, 0), 1.0, 1),
                ((-1, 0), 1.0, 1),
                ((0, -1), 1.0, 1),
                ((1, 1), SQRT_2, 1),
                ((1, -1), SQRT_2, 1),
                ((-1, 1), SQRT_2, 1),
                ((-1, -1), SQRT_2, 1),
            ];
        } else {
            neighbors = vec![((0, 1), 1.0, 1), ((1, 0), 1.0, 1), ((-1, 0), 1.0, 1), ((0, -1), 1.0, 1)];
        }

        // TODO octal heuristic
        let heuristic: fn(Point2d, Point2d) -> f32;
        match self.heuristic.as_ref() {
            "manhattan" => heuristic = manhattan_heuristic,
            "octal" => heuristic = octal_heuristic,
            _ => heuristic = euclidean_heuristic,
        }


//        while let Some(Node {cost_to_source,total_estimated_cost,position,came_from }) = heap.pop() {
        while let Some(current) = heap.pop() {
            // Already checked this position
            if closed_list.contains(&current.position) {
                continue;
            }

            nodes_map.insert(current.position, current.came_from);

            if current.position == target {
                // Construct path
                return construct_path(source, target, &nodes_map);
            }

            closed_list.insert(current.position);

            for (neighbor, real_cost, cost_estimate) in neighbors.iter() {
                let new_node = current.position.add_neighbor(*neighbor);
                // TODO add cost from grid
                //  if grid point has value == 0 (or -1?): is wall

                let new_cost_to_source = current.cost_to_source + *real_cost;
                let estimate_cost = heuristic(new_node, target);
                let total_estimated_cost = new_cost_to_source + estimate_cost;

                // Should perhaps check if position is already in open list, but doesnt matter
                heap.push(Node {
                    cost_to_source: new_cost_to_source,
                    total_estimated_cost: total_estimated_cost,
                    position: new_node,
                    came_from: current.position,
                });
            }
        }
        None
    }
}

static SOURCE: Point2d = Point2d { x: 0, y: 0 };
static TARGET: Point2d = Point2d { x: 50, y: 100 };

#[cfg(test)] // Only compiles when running tests
mod tests {
    use super::*;
    #[allow(unused_imports)]
    use test::Bencher;

    fn manhattan_test() {
        let mut pf = PathFinder {
            allow_diagonal: true,
            heuristic: String::from("manhattan"),
            grid: vec![vec![]],
        };
        // TODO use array2d https://docs.rs/array2d/0.2.1/array2d/
        // https://www.programming-idioms.org/idiom/26/create-a-2-dimensional-array/448/rust
        // https://stackoverflow.com/a/27984550/10882657
        let grid: Vec<Vec<i32>> = vec![
            vec![1, 1, 1, 1, 1],
            vec![1, 1, 1, 1, 1],
            vec![1, 1, 1, 1, 1],
            vec![1, 1, 1, 1, 1],
            vec![1, 1, 1, 1, 1],
        ];
        pf.update_grid(grid.clone());
        let path = pf.find_path(SOURCE, TARGET);
        //        println!("RESULT {:?}", path);
    }

    fn octal_test() {
        let pf = PathFinder {
            allow_diagonal: true,
            heuristic: String::from("octal"),
            grid: vec![vec![]],
        };
        let path = pf.find_path(SOURCE, TARGET);
        //        println!("RESULT {:?}", path);
    }

    fn euclidean_test() {
        let pf = PathFinder {
            allow_diagonal: true,
            heuristic: String::from("euclidean"),
            grid: vec![vec![]],
        };
        let path = pf.find_path(SOURCE, TARGET);
        //        println!("RESULT {:?}", path);
    }

    fn crate_pathfinding_astar_test() {
        let path = astar(
            &SOURCE,
            |p| p.successors(),
            |p| p.distance(&TARGET),
            |p| *p == TARGET,
        );
        //        println!("RESULT crate {:?}", path);
    }

    // This will only be executed when using "cargo test" and not "cargo bench"
    #[test]
    fn test_path() {
        let pf = PathFinder {
            allow_diagonal: true,
            heuristic: String::from("octal"),
            grid: vec![vec![]],
        };

        let source = Point2d { x: 0, y: 0 };
        let target = Point2d { x: 5, y: 10 };

        let path = pf.find_path(source, target);
        println!("RESULT mine {:?}", path);
    }

    #[bench]
    fn bench_manhattan_test(b: &mut Bencher) {
        b.iter(|| manhattan_test());
    }

    #[bench]
    fn bench_octal_test(b: &mut Bencher) {
        b.iter(|| octal_test());
    }

    #[bench]
    fn bench_euclidean_test(b: &mut Bencher) {
        b.iter(|| euclidean_test());
    }

    #[bench]
    fn bench_crate_pathfinding_astar_test(b: &mut Bencher) {
        b.iter(|| crate_pathfinding_astar_test());
    }
}
