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

use std::collections::{HashMap, HashSet};

// https://github.com/PyO3/pyo3
use pyo3::prelude::*;
use pyo3::wrap_pyfunction;
use pyo3::PyObjectProtocol;

use ndarray::{ArrayD, ArrayViewD, ArrayViewMut2, ArrayViewMutD};
use numpy::{IntoPyArray, PyArray2, PyArrayDyn, PyReadonlyArrayDyn};
use pyo3::types::{PyDict, PyList, PySet};

use blitz_path::a_star_path;
use blitz_path::jps_path;
use movingai::Coords2D;
use movingai::MovingAiMap;

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
fn tuple_interaction(_py: Python, my_tuple: (i32, i32)) -> PyResult<(i32, i32, i32)> {
    /// Input is tuple of fixed length, outputting a tuple of fixed length
    Ok((my_tuple.0, my_tuple.1, my_tuple.0 + my_tuple.1))
}

#[pyfunction]
fn add_key_to_dict(_py: Python, my_dict: &PyDict) {
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
        *my_value += 1;
    }
    my_dict
}

#[pyfunction]
fn add_element_to_set(_py: Python, my_set: &PySet) {
    /// Add an item to a set
    my_set.add(420).unwrap();
}

#[pyfunction]
fn add_element_to_set_with_return(_py: Python, mut my_set: HashSet<i32>) -> HashSet<i32> {
    /// Add an item to a set, then return
    my_set.insert(421);
    my_set
}

// Numpy examples

// fn rust_ndarray_add_two(_py: Python, mut my_array: &ArrayViewMutD<'_, i32>)  {
//     /// Read a numpy array and add 2 to each element
//     my_array *= 2;
// }

fn rust_numpy_add_2d(mut x: ArrayViewMut2<i64>, a: i64) {
    x += a;
}

#[pyfunction]
fn numpy_add_value_2d(_py: Python, x: &PyArray2<i64>, a: i64) {
    /// Add value to 2 dimensional numpy array - no return
    let b = unsafe { x.as_array_mut() };
    rust_numpy_add_2d(b, a);
}

fn rust_numpy_add(mut x: ArrayViewMutD<i64>, a: i64) {
    x += a;
}

#[pyfunction]
fn numpy_add_value(_py: Python, x: &PyArrayDyn<i64>, a: i64) {
    /// Add value to any dimensional numpy array - no return
    let b = unsafe { x.as_array_mut() };
    rust_numpy_add(b, a);
}

fn rust_numpy_add_and_return(x: ArrayViewD<'_, i64>, a: i64) -> ArrayD<i64> {
    &x + a
}

#[pyfunction]
fn numpy_add_value_with_return<'py>(
    py: Python<'py>,
    x: PyReadonlyArrayDyn<i64>,
    a: i64,
) -> &'py PyArrayDyn<i64> {
    /// Return new array with 2 added to each element
    let b = x.as_array();
    rust_numpy_add_and_return(b, a).into_pyarray(py)
}

#[pyfunction]
fn numpy_calc_sum_of_array(_py: Python, x: PyReadonlyArrayDyn<i64>) -> i64 {
    /// Return sum of any dimensional array
    let b = x.as_array();
    b.sum()
}

// #[pyfunction]
// fn numpy_add_value_with_return<'py>(_py: Python<'py>, x: PyReadonlyArrayDyn<i64>, a: i64) -> &'py ArrayViewMut2<i64> {
//     let b = x.as_array();
//     &b + a
// }

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

/// This module is a python module implemented in Rust.
/// This function name has to be the same as the lib.name declared in Cargo.toml
#[pymodule]
fn my_library(_py: Python, m: &PyModule) -> PyResult<()> {
    // Add all functions and classes (structs) here that need to be exported and callable via Python

    // Functions to be exported
    m.add_wrapped(wrap_pyfunction!(add_one))?;
    m.add_wrapped(wrap_pyfunction!(add_one_and_a_half))?;
    m.add_wrapped(wrap_pyfunction!(concatenate_string))?;
    /// List
    m.add_wrapped(wrap_pyfunction!(sum_of_list))?;
    m.add_wrapped(wrap_pyfunction!(append_to_list))?;
    m.add_wrapped(wrap_pyfunction!(double_of_list))?;
    /// Tuple
    m.add_wrapped(wrap_pyfunction!(tuple_interaction))?;
    /// Dict
    m.add_wrapped(wrap_pyfunction!(add_key_to_dict))?;
    m.add_wrapped(wrap_pyfunction!(change_key_value))?;
    m.add_wrapped(wrap_pyfunction!(change_key_value_with_return))?;
    /// Set
    m.add_wrapped(wrap_pyfunction!(add_element_to_set))?;
    m.add_wrapped(wrap_pyfunction!(add_element_to_set_with_return))?;
    /// Numpy
    m.add_wrapped(wrap_pyfunction!(numpy_add_value_2d))?;
    m.add_wrapped(wrap_pyfunction!(numpy_add_value))?;
    m.add_wrapped(wrap_pyfunction!(numpy_add_value_with_return))?;
    m.add_wrapped(wrap_pyfunction!(numpy_calc_sum_of_array))?;

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
