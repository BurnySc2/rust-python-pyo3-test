// This will will be exported when using "cargo build" command
// It should create a my_library.dll (windows) file in the target/debug folder

/* Introduction / Helper websites:
https://pyo3.rs/
https://docs.rs/pyo3
*/

#![feature(cell_update)]
// Testing and benchmark crate
#![feature(test)]
extern crate test;

mod pathfinding_test;

use pyo3::prelude::*;
use pyo3::wrap_pyfunction;
use pyo3::PyObjectProtocol;
// https://github.com/PyO3/pyo3

#[pyclass(name = "Point")]
#[derive(Copy, Clone, Debug)]
struct Point {
    // For the .x and .y attributes to be accessable in python, it requires these macros
    #[pyo3(get, set)]
    x: f64,
    #[pyo3(get, set)]
    y: f64,
}

#[pymethods]
impl Point {
    #[new]
    fn new(x_: f64, y_: f64) -> Self {
        Point { x: x_, y: y_ }
    }
    fn distance_to(&self, other: &Point) -> f64 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt()
    }
    fn distance_to_squared(&self, other: &Point) -> f64 {
        (self.x - other.x).powi(2) + (self.y - other.y).powi(2)
    }
}

#[pyproto]
impl PyObjectProtocol for Point {
    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("RustPoint(x: {}, y: {})", self.x, self.y))
    }
    fn __str__(&self) -> PyResult<String> {
        Ok(format!("RustPoint(x: {}, y: {})", self.x, self.y))
    }
}

/// The name of the class can be changed here, e.g. 'name=PointCollection' and will then be available through my_library.PointCollection instead
#[pyclass(name = "PointCollection")]
pub struct PointCollection {
    #[pyo3(get, set)]
    points: Vec<Point>,
}

#[pymethods]
impl PointCollection {
    #[new]
    fn new(points: Vec<Point>) -> Self {
        PointCollection { points }
    }

    #[allow(dead_code)]
    fn len(&self) -> PyResult<usize> {
        Ok(self.points.len())
    }

    #[allow(dead_code)]
    fn append(&mut self, point: Point) {
        self.points.push(point);
    }

    #[allow(dead_code)]
    fn print(&self) {
        for i in self.points.clone() {
            println!("{:?}", i);
        }
    }

    #[allow(dead_code)]
    fn closest_point(&self, other: &Point) -> Point {
        // TODO raise error when list of points is empty
        assert!(!self.points.is_empty());
        let mut iterable = self.points.clone().into_iter();
        let mut closest = iterable.next().unwrap();
        let mut distance_sq_closest = closest.distance_to_squared(other);
        for p in iterable {
            let p_distance_sq = p.distance_to_squared(other);
            if p_distance_sq < distance_sq_closest {
                closest = p;
                distance_sq_closest = p_distance_sq;
            }
        }
        closest
    }
}

#[pyproto]
impl PyObjectProtocol for PointCollection {
    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("PointCollection({:?})", self.points))
    }
    fn __str__(&self) -> PyResult<String> {
        Ok(format!("PointCollection({:?})", self.points))
    }
}

#[allow(unused_imports)]
use ndarray::{ArrayD, ArrayViewD, ArrayViewMutD};
use numpy::{IntoPyArray, PyArrayDyn, PyReadonlyArrayDyn};
use pyo3::prelude::{pymodule, PyModule, PyResult, Python};
// Numpy examples

// immutable example
fn mult_with_return_rust(a: f64, x: ArrayViewD<'_, f64>, y: ArrayViewD<'_, f64>) -> ArrayD<f64> {
    a * &x + &y
}

// wrapper of `mult_with_return_rust`
#[pyfunction]
fn mult_with_return<'py>(
    py: Python<'py>,
    a: f64,
    x: PyReadonlyArrayDyn<f64>,
    y: PyReadonlyArrayDyn<f64>,
) -> &'py PyArrayDyn<f64> {
    let x = x.as_array();
    let y = y.as_array();
    mult_with_return_rust(a, x, y).into_pyarray(py)
}

// mutable example (no return)
fn mult_without_return_rust(a: f64, mut x: ArrayViewMutD<'_, f64>) {
    x *= a;
}

// wrapper of `mult_without_return_rust`
#[pyfunction]
fn mult_without_return(_py: Python<'_>, a: f64, x: &PyArrayDyn<f64>) -> PyResult<()> {
    let x = unsafe { x.as_array_mut() };
    mult_without_return_rust(a, x);
    Ok(())
}

//mod base;
// Simple examples

/// Formats the sum of two numbers as string
#[pyfunction]
fn sum_as_string(a: usize, b: usize) -> PyResult<String> {
    Ok((a + b).to_string())
}

/// Iterative approach of calculating a factorial
#[pyfunction]
fn factorial_iter(input: u128) -> u128 {
    let mut result = 1;
    for i in 2..=input {
        result *= i
    }
    result
}

/// Recursive approach of calculating a factorial
#[pyfunction]
fn factorial(input: u128) -> u128 {
    if input == 1 {
        return 1u128;
    }
    input * factorial(input - 1)
}

/// This module is a python module implemented in Rust.
/// This function name has to be the same as the lib.name declared in Cargo.toml
#[pymodule]
fn my_library(_py: Python, m: &PyModule) -> PyResult<()> {
    // Add all functions and classes (structs) here that need to be exported and callable via Python

    // Functions to be exported
    m.add_wrapped(wrap_pyfunction!(sum_as_string))?;
    m.add_wrapped(wrap_pyfunction!(factorial))?;
    m.add_wrapped(wrap_pyfunction!(factorial_iter))?;

    m.add_wrapped(wrap_pyfunction!(mult_without_return))?;
    m.add_wrapped(wrap_pyfunction!(mult_with_return))?;

    // Classes to be exported
    m.add_class::<Point>()?;
    m.add_class::<PointCollection>()?;

    Ok(())
}

#[cfg(test)] // Only compiles when running tests
mod tests {
    use super::*;
    #[allow(unused_imports)]
    use test::Bencher;

    // This will only be executed when using "cargo test" and not "cargo bench"
    #[test]
    fn test_factorial_function() {
        assert_eq!(2, factorial(2));
        assert_eq!(6, factorial(3));
        assert_eq!(24, factorial(4));
        assert_eq!(24, factorial_iter(4));
    }

    #[bench]
    fn bench_factorial_function(b: &mut Bencher) {
        b.iter(|| factorial(2));
        b.iter(|| factorial(3));
        b.iter(|| factorial(4));
        b.iter(|| factorial_iter(4));
    }
}
