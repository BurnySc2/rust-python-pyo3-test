use pyo3::prelude::*;

use std::cmp::Ordering;

use std::collections::BinaryHeap;
use std::collections::HashMap;
use std::collections::HashSet;

use std::f32::consts::SQRT_2;
use std::f32::EPSILON;

use pyo3::types::PyAny;

use std::ops::Sub;

use ndarray::Array;

use ndarray::Array2;

use std::fs::File;
use std::io::Read;

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

#[pyclass]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Point2d {
    x: i32,
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
        other
            .total_estimated_cost
            .partial_cmp(&self.total_estimated_cost)
    }
}

// The result of this implementation doesnt seem to matter - instead what matters, is that it is implemented
impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .total_estimated_cost
            .partial_cmp(&self.total_estimated_cost)
            .unwrap()
    }
}

impl Eq for Node {}

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
    let x = source.x - target.x;
    let xx = x * x;
    let y = source.y - target.y;
    let yy = y * y;
    let _sum = xx + yy;
    ((xx + yy) as f32).sqrt()
}

fn no_heuristic(_source: &Point2d, target: &Point2d) -> f32 {
    0.0
}

fn construct_path(
    source: &Point2d,
    target: &Point2d,
    nodes_map: &HashMap<Point2d, Point2d>,
) -> Option<Vec<Point2d>> {
    let mut path = vec![];
    path.push(*target);
    let mut pos = nodes_map.get(&target).unwrap();
    while pos != source {
        path.push(*pos);
        pos = nodes_map.get(pos).unwrap();
    }
    path.push(*source);
    path.reverse();
    Some(path)
}

// https://doc.rust-lang.org/std/default/trait.Default.html
// https://stackoverflow.com/questions/19650265/is-there-a-faster-shorter-way-to-initialize-variables-in-a-rust-struct
//#[pyclass]
struct PathFinder {
    allow_diagonal: bool,
    heuristic: String,
    grid: Array2<u8>,
    came_from_grid: Array2<u8>,
}

// https://medium.com/@nicholas.w.swift/easy-a-star-pathfinding-7e6689c7f7b2
//#[pymethods]
impl PathFinder {
    fn update_grid(&mut self, grid: Array2<u8>) {
        self.grid = grid;
    }

    fn find_path(&self, source: &Point2d, target: &Point2d) -> Option<Vec<Point2d>> {
        let mut nodes_map = HashMap::new();
        let mut closed_list = HashSet::new();

        // Add source
        let mut heap = BinaryHeap::new();
        heap.push(Node {
            cost_to_source: 0.0,
            total_estimated_cost: 0.0,
            position: *source,
            came_from: *source,
        });

        let neighbors;
        match self.allow_diagonal {
            true => {
                neighbors = vec![
                    ((0, 1), 1.0, 1),
                    ((1, 0), 1.0, 1),
                    ((-1, 0), 1.0, 1),
                    ((0, -1), 1.0, 1),
                    ((1, 1), SQRT_2, 1),
                    ((1, -1), SQRT_2, 1),
                    ((-1, 1), SQRT_2, 1),
                    ((-1, -1), SQRT_2, 1),
                ]
            }
            false => {
                neighbors = vec![
                    ((0, 1), 1.0, 1),
                    ((1, 0), 1.0, 1),
                    ((-1, 0), 1.0, 1),
                    ((0, -1), 1.0, 1),
                ]
            }
        }

        let heuristic: fn(&Point2d, &Point2d) -> f32;
        match self.heuristic.as_ref() {
            "manhattan" => heuristic = manhattan_heuristic,
            "octal" => heuristic = octal_heuristic,
            "euclidean" => heuristic = euclidean_heuristic,
            "none" => heuristic = no_heuristic,
            _ => heuristic = euclidean_heuristic,
        }

        while let Some(Node {
            cost_to_source,
            position,
            came_from,
            ..
        }) = heap.pop()
        {
            // Already checked this position
            if closed_list.contains(&position) {
                continue;
            }

            nodes_map.insert(position, came_from);

            if position == *target {
                return construct_path(&source, &target, &nodes_map);
            }

            closed_list.insert(position);

            for (neighbor, real_cost, _cost_estimate) in neighbors.iter() {
                let new_node = position.add_neighbor(*neighbor);
                // TODO add cost from grid
                //  if grid point has value == 0 (or -1?): is wall

                let new_cost_to_source = cost_to_source + *real_cost;

                // Should perhaps check if position is already in open list, but doesnt matter
                heap.push(Node {
                    cost_to_source: new_cost_to_source,
                    total_estimated_cost: new_cost_to_source + heuristic(&new_node, target),
                    position: new_node,
                    came_from: position,
                });
            }
        }
        None
    }
}

pub fn grid_setup(size: usize) -> Array2<u8> {
    // Set up a grid with size 'size' and make the borders a wall (value 1)
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

    fn astar_pf(grid: Array2<u8>) -> PathFinder {
        let came_from_grid = Array::zeros(grid.raw_dim());
        PathFinder {
            allow_diagonal: true,
            heuristic: String::from("manhattan"),
            grid,
            came_from_grid,
        }
    }

    fn astar_test(pf: &mut PathFinder, source: &Point2d, target: &Point2d) -> Option<Vec<Point2d>> {
        let path = pf.find_path(source, target);
        path
    }

    #[bench]
    fn bench_astar_test_from_file(b: &mut Bencher) {
        let result = read_grid_from_file(String::from("AutomatonLE.txt"));
        let (array, _height, _width) = result.unwrap();
        // Spawn to spawn
        let source = Point2d { x: 32, y: 51 };
        let target = Point2d { x: 150, y: 129 };
        // Main ramp to main ramp
        //        let source = Point2d { x: 32, y: 51 };
        //        let target = Point2d { x: 150, y: 129 };
        let mut pf = astar_pf(array);
        b.iter(|| astar_test(&mut pf, &source, &target));
    }

    #[bench]
    fn bench_astar_test(b: &mut Bencher) {
        let grid = grid_setup(30);
        let mut pf = astar_pf(grid);
        let source: Point2d = Point2d { x: 5, y: 5 };
        let target: Point2d = Point2d { x: 10, y: 12 };
        b.iter(|| astar_test(&mut pf, &source, &target));
    }
}
