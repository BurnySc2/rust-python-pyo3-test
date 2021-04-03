// This will will be exported when using "cargo build" command
// It should create a my_library.dll (windows) file in the target/debug folder

/* Introduction / Helper websites:
https://pyo3.rs/
https://docs.rs/pyo3
*/
#![allow(unused_doc_comments)]
#![feature(cell_update)]
// Testing and benchmark crate
#![feature(test)]
extern crate test;

use pyo3::prelude::*;
use pyo3::wrap_pyfunction;
use pyo3::PyObjectProtocol;
// https://github.com/PyO3/pyo3

use blitz_path::a_star_path;
use blitz_path::jps_path;
use movingai::Coords2D;
use movingai::MovingAiMap;
use std::collections::{HashMap, HashSet};

/// Class example
#[pyclass(name = "RustPoint2")]
#[derive(Copy, Clone, Debug)]
pub struct RustPoint2 {
    #[pyo3(get, set)]
    x: usize,
    #[pyo3(get, set)]
    y: usize,
}

#[pymethods]
impl RustPoint2 {
    #[new]
    fn new(x_: usize, y_: usize) -> Self {
        RustPoint2 { x: x_, y: y_ }
    }

    fn to_coords_2d(&self) -> Coords2D {
        (self.x, self.y)
    }

    fn distance_to(&self, other: &RustPoint2) -> f64 {
        (((self.x - other.x) as f64).powi(2) + ((self.y - other.y) as f64).powi(2)).sqrt()
    }

    fn distance_to_squared(&self, other: &RustPoint2) -> f64 {
        ((self.x - other.x) as f64).powi(2) + ((self.y - other.y) as f64).powi(2)
    }
}

#[pyproto]
impl PyObjectProtocol for RustPoint2 {
    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("RustPoint2(x: {}, y: {})", self.x, self.y))
    }
    fn __str__(&self) -> PyResult<String> {
        Ok(format!("RustPoint2(x: {}, y: {})", self.x, self.y))
    }
}

#[pyclass(name = "RustPixelMap")]
#[derive(Debug)]
pub struct RustPixelMap {
    map: MovingAiMap,
}

#[pymethods]
impl RustPixelMap {
    #[new]
    fn new(width_: usize, height_: usize, map_: Vec<char>) -> Self {
        RustPixelMap {
            map: MovingAiMap::new(String::from("test"), width_, height_, map_),
        }
    }

    fn jps_path(&self, start_pos: RustPoint2, goal_pos: RustPoint2) -> Vec<Coords2D> {
        if let Some(path) = jps_path(&self.map, start_pos.to_coords_2d(), goal_pos.to_coords_2d()) {
            return path.steps();
        }
        vec![]
    }

    fn astar_path(&self, start_pos: RustPoint2, goal_pos: RustPoint2) -> Vec<Coords2D> {
        if let Some(path) =
            a_star_path(&self.map, start_pos.to_coords_2d(), goal_pos.to_coords_2d())
        {
            return path.steps();
        }
        vec![]
    }
}

/// The name of the class can be changed here, e.g. 'name=PointCollection' and will then be available through my_library.PointCollection instead
#[pyclass(name = "PointCollection")]
pub struct PointCollection {
    #[pyo3(get, set)]
    points: Vec<RustPoint2>,
}

#[pymethods]
impl PointCollection {
    #[new]
    fn new(points: Vec<RustPoint2>) -> Self {
        PointCollection { points }
    }

    #[allow(dead_code)]
    fn len(&self) -> PyResult<usize> {
        Ok(self.points.len())
    }

    #[allow(dead_code)]
    fn append(&mut self, point: RustPoint2) {
        self.points.push(point);
    }

    #[allow(dead_code)]
    fn print(&self) {
        for i in self.points.clone() {
            println!("{:?}", i);
        }
    }

    #[allow(dead_code)]
    fn closest_point(&self, other: &RustPoint2) -> RustPoint2 {
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

/// Primitive function examples
#[pyfunction]
fn add_one(_py: Python, value: i128) -> PyResult<i128> {
    Ok(value + 1)
}

#[pyfunction]
fn add_one_and_a_half(_py: Python, value: f64) -> PyResult<f64> {
    Ok(value + 1.5)
}

#[pyfunction]
fn concatenate_string(_py: Python, value: String) -> PyResult<String> {
    /// Concatenating 2 strings
    let mut concat = value;
    concat.push_str(" world!");
    Ok(concat)
}

#[pyfunction]
fn sum_of_list(_py: Python, my_list: Vec<i32>) -> PyResult<i32> {
    /// Looping over a list and calculating the sum
    // let sum: i32 = my_list.iter().sum();
    // let sum: Option<i32> = my_list.iter().reduce(|a, b| &a+&b);
    let sum: Option<i32> = my_list.into_iter().reduce(|a, b| a + b);
    if let Some(my_sum) = sum {
        return Ok(my_sum);
    }
    Ok(0)
}

#[pyfunction]
fn append_to_list(_py: Python, my_list: &PyList) {
    /// Mutating a list without return statement
    my_list.append(420).unwrap();
}

#[pyfunction]
fn double_of_list(_py: Python, my_list: Vec<i32>) -> PyResult<Vec<i32>> {
    /// Returning a new list where each element is doubled
    let my_list_double = my_list.into_iter().map(|x| &x * 2).collect();
    Ok(my_list_double)
}

#[pyfunction]
fn add_key_to_dict(_py: Python, my_dict: &PyDict) {
    #[allow(unused_doc_comments)]
    /// Adding a key
    my_dict.set_item("test", "hello").unwrap();
}

#[pyfunction]
fn change_key_value(_py: Python, my_dict: &PyDict) {
    /// Change a value in a dict
    let my_option = my_dict.get_item("hello");
    if let Some(my_value) = my_option {
        let value = my_value.extract::<i32>().unwrap();
        my_dict.set_item("hello", value + 1).unwrap();
    }
}

#[pyfunction]
fn change_key_value_with_return(
    _py: Python,
    mut my_dict: HashMap<String, i32>,
) -> HashMap<String, i32> {
    /// Change a value in a dict, then return it
    if let Some(my_value) = my_dict.get_mut("hello") {
        *my_value = *my_value + 1;
    }
    my_dict
}

#[pyfunction]
fn add_element_to_set(_py: Python, my_set: &PySet) {
    /// Add an item in a set
    my_set.add(420).unwrap();
}

#[pyfunction]
fn add_element_to_set_with_return(_py: Python, mut my_set: HashSet<i32>) -> HashSet<i32> {
    /// Add an item in a set then return
    my_set.insert(421);
    my_set
}

#[allow(unused_imports)]
use ndarray::{ArrayD, ArrayViewD, ArrayViewMutD};
#[allow(unused_imports)]
use pyo3::types::PyList;
// use numpy::{IntoPyArray, PyArrayDyn, PyReadonlyArrayDyn};
use pyo3::types::{PyDict, PySet};

// Numpy examples

// // immutable example
// fn mult_with_return_rust(a: f64, x: ArrayViewD<'_, f64>, y: ArrayViewD<'_, f64>) -> ArrayD<f64> {
//     a * &x + &y
// }
//
// // wrapper of `mult_with_return_rust`
// #[pyfunction]
// fn mult_with_return<'py>(
//     py: Python<'py>,
//     a: f64,
//     x: PyReadonlyArrayDyn<f64>,
//     y: PyReadonlyArrayDyn<f64>,
// ) -> &'py PyArrayDyn<f64> {
//     let x = x.as_array();
//     let y = y.as_array();
//     mult_with_return_rust(a, x, y).into_pyarray(py)
// }
//
// // mutable example (no return)
// fn mult_without_return_rust(a: f64, mut x: ArrayViewMutD<'_, f64>) {
//     x *= a;
// }
//
// // wrapper of `mult_without_return_rust`
// #[pyfunction]
// fn mult_without_return(_py: Python<'_>, a: f64, x: &PyArrayDyn<f64>) -> PyResult<()> {
//     let x = unsafe { x.as_array_mut() };
//     mult_without_return_rust(a, x);
//     Ok(())
// }

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
    m.add_wrapped(wrap_pyfunction!(add_one))?;
    m.add_wrapped(wrap_pyfunction!(add_one_and_a_half))?;
    m.add_wrapped(wrap_pyfunction!(concatenate_string))?;
    m.add_wrapped(wrap_pyfunction!(sum_of_list))?;
    m.add_wrapped(wrap_pyfunction!(append_to_list))?;
    m.add_wrapped(wrap_pyfunction!(double_of_list))?;
    m.add_wrapped(wrap_pyfunction!(add_key_to_dict))?;
    m.add_wrapped(wrap_pyfunction!(change_key_value))?;
    m.add_wrapped(wrap_pyfunction!(change_key_value_with_return))?;
    m.add_wrapped(wrap_pyfunction!(add_element_to_set))?;
    m.add_wrapped(wrap_pyfunction!(add_element_to_set_with_return))?;

    m.add_wrapped(wrap_pyfunction!(sum_as_string))?;
    m.add_wrapped(wrap_pyfunction!(factorial))?;
    m.add_wrapped(wrap_pyfunction!(factorial_iter))?;

    // m.add_wrapped(wrap_pyfunction!(mult_without_return))?;
    // m.add_wrapped(wrap_pyfunction!(mult_with_return))?;

    // Classes to be exported
    // Linking error on linux if you import a local module/crate
    m.add_class::<RustPoint2>()?;
    m.add_class::<RustPixelMap>()?;
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
