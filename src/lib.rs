// This will will be exported when using "cargo build" command
// It should create a my_library.dll (windows) file in the target/debug folder

/* Introduction / Helper websites:
https://pyo3.rs/
https://docs.rs/pyo3/0.8.4/pyo3/index.html
*/

#![feature(cell_update)]
// Testing and benchmark crate
#![feature(test)]
extern crate test;

mod pathfinding_test;

use pyo3::prelude::*;
use pyo3::types::PyAny;
use pyo3::wrap_pyfunction;
use pyo3::PyObjectProtocol;
// https://github.com/PyO3/pyo3

#[pyclass]
#[derive(Copy, Clone, Debug)]
struct Point {
    // For the .x and .y attributes to be accessable in python, it requires these macros
//    #[pyo3(get, set)]
    x: f64,
//    #[pyo3(get, set)]
    y: f64,
}

//#[pymethods]
impl Point {
//    #[new]
//    fn new(obj: &PyRawObject, x_: f64, y_: f64) {
//        obj.init(Point { x: x_, y: y_ })
//    }
    fn distance_to(&self, other: &Point) -> f64 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt()
    }
    fn distance_to_squared(&self, other: &Point) -> f64 {
        (self.x - other.x).powi(2) + (self.y - other.y).powi(2)
    }
}
//
///// Implements the repr function for Point class: https://pyo3.rs/v0.8.4/class.html#string-conversions
#[pyproto]
impl PyObjectProtocol for Point {
    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("Point ( x: {:?}, y: {:?} )", self.x, self.y))
    }
}
//
//// Necessary function implementation to convert a Point from python to rust
//class Point:
//    def __init__(self, x, y):
//        self.x = x
//        self.y = y
//# This class can now be used in rust as Point
//
impl<'source> FromPyObject<'source> for Point {
    fn extract(ob: &'source PyAny) -> PyResult<Point> {
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

//// The name of the class can be changed here, e.g. 'name=MyPoints' and will then be available through my_library.MyPoints instead
#[pyclass(name=PointCollection)]
pub struct PointCollection {
//    #[pyo3(get, set)]
    points: Vec<Point>,
}

//#[pymethods]
impl PointCollection {
//    #[new]
//    fn new(obj: &PyRawObject, _points: Vec<&Point>) {
//        let new_vec: Vec<Point> = _points.into_iter().copied().collect();
//        obj.init(PointCollection { points: new_vec })
//    }

    fn len(&self) -> PyResult<usize> {
        Ok(self.points.len())
    }

    fn append(&mut self, _point: Point) {
        self.points.push(_point);
    }

    fn print(&self) {
        for i in self.points.clone() {
            println!("{:?}", i);
        }
    }

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

use ndarray::{ArrayD, ArrayViewD, ArrayViewMutD};
use numpy::{IntoPyArray, PyArrayDyn};
// Numpy examples

// immutable example
fn mult_with_return(a: f64, x: ArrayViewD<f64>) -> ArrayD<f64> {
    &x * a
    // Also works:
    // a * &x
}

// mutable example (no return)
fn mult_mutable(a: f64, mut x: ArrayViewMutD<f64>) {
    x *= a;
}

// wrapper of `axpy`
#[pyfunction]
fn mult_with_return_py(py: Python, a: f64, x: &PyArrayDyn<f64>) -> Py<PyArrayDyn<f64>> {
    let x = x.as_array();
    mult_with_return(a, x).into_pyarray(py).to_owned()
}

// wrapper of `mult`
#[pyfunction]
fn mult_mutable_py(_py: Python, a: f64, x: &PyArrayDyn<f64>) -> PyResult<()> {
    let x = x.as_array_mut();
    mult_mutable(a, x);
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
    m.add_wrapped(wrap_pyfunction!(mult_with_return_py))?;
    m.add_wrapped(wrap_pyfunction!(mult_mutable_py))?;

    // Classes to be exported
    m.add_class::<Point>()?;
    m.add_class::<PointCollection>()?;

    m.add_class::<pathfinding_test::jps::Point2d>()?;
    m.add_class::<pathfinding_test::jps::PathFinder>()?;

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
}
