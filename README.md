# rust-python-pyo3-test

[![Actions Status](https://github.com/BurnySc2/rust-python-pyo3-test/workflows/RustBuild/badge.svg)](https://github.com/BurnySc2/rust-python-pyo3-test/actions)

- Have a rust function that takes and returns most basic python types
  - [x] Int
  - [x] Float
  - [x] String
  - [x] List
  - [x] Tuple
  - [x] Dict
  - [x] Set
- Create a struct/class that 
  - [x] can be initialized in python
  - [x] has attributes exposed to python
  - [x] has methods exposed that can be called in python
- Create a class-instance in python and then pass it to rust to
  - [ ] modify attributes
  - [ ] call python instance methods
- [x] Load a numpy array in rust and do some operation on it (e.g. sum of all elements)
- [x] Load a numpy array in rust and add a number to each element (mutable numpy array)
- [x] Load a numpy array in rust and add a number to each element and return new array (immutable numpy array)
- [ ] Be able to run tests and benchmarks

