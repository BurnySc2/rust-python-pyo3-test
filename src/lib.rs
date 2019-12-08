// This will will be exported when using "cargo build" command
// It should create a my_library.dll (windows) file in the target/debug folder


// Testing and benchmark crate
#![feature(test)]
extern crate test;

use pyo3::prelude::*;
use pyo3::wrap_pyfunction;
// https://github.com/PyO3/pyo3


#[pyclass]
pub struct Point2d {
    x: f64,
    y: f64,
}


#[pymethods]
impl Point2d {
    #[new]
    fn new(obj: &PyRawObject, x_: f64, y_: f64) {
        obj.init(Point2d { x: x_, y: y_ })
    }

    #[staticmethod]
    fn origin() -> Point2d {
        Point2d { x: 0.0, y: 0.0 }
    }

    fn distance_to(&self, other: &Point2d) -> f64 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt()
    }

    fn distance_to_squared(&self, other: &Point2d) -> f64 {
        (self.x - other.x).powi(2) + (self.y - other.y).powi(2)
    }
}


#[pyfunction]
/// Formats the sum of two numbers as string
fn sum_as_string(a: usize, b: usize) -> PyResult<String> {
    Ok((a + b).to_string())
}


// Iterative approach
//#[pyfunction]
//fn factorial(input: u128) -> u128 {
//    let mut result = 1;
//    for i in 2..input+1 {
//        result *= i
//    }
//    result
//}


// Fairly identical to the function above, recursive approach
#[pyfunction]
fn factorial(input: u128) -> u128 {
    if input == 1 {
        return 1u128
    }
    input * factorial(input - 1)
}

#[pyfunction]
fn distance(x0: f32, y0: f32, x1: f32, y1: f32) -> f32 {
    (x1 - x0).powi(2) + (y1 - y0).powi(2).sqrt()
}

#[pyfunction]
fn distance_squared(x0: f32, y0: f32, x1: f32, y1: f32) -> f32 {
    (x1 - x0).powi(2) + (y1 - y0).powi(2)
}


/// This module is a python module implemented in Rust.
/// This function name has to be the same as the lib.name declared in Cargo.toml
#[pymodule]
fn my_library(_py: Python, m: &PyModule) -> PyResult<()> {
    // Add all functions and classes (structs) here that need to be exported and callable via Python
    m.add_wrapped(wrap_pyfunction!(sum_as_string))?;
    m.add_wrapped(wrap_pyfunction!(factorial))?;
    m.add_wrapped(wrap_pyfunction!(distance))?;
    m.add_wrapped(wrap_pyfunction!(distance_squared))?;
    m.add_class::<Point2d>()?;

    Ok(())
}


#[cfg(test)] // Only compiles when running tests
mod tests {
    use super::*;
    use test::Bencher;

    // This will only be executed when using "cargo test" and not "cargo bench"
    #[test]
    fn test_factorial_function() {
        assert_eq!(2, factorial(2));
        assert_eq!(6, factorial(3));
        assert_eq!(24, factorial(4));
    }
}