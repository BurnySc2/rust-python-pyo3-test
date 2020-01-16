#![feature(cell_update)]
// Testing and benchmark crate
#![feature(test)]
extern crate test;

use std::fs::File;
//use std::io::BufReader;
use std::io::prelude::*;

pub mod pathfinding_test;

fn write_path_to_file(path: Vec<pathfinding_test::jps::Point2d>) {
    //    let mut tuple_vec = vec![];
    let mut file = File::create("path.txt").unwrap();
    for i in path.iter() {
        let pathfinding_test::jps::Point2d { x, y } = i;
        //        tuple_vec.push((x, y));
        file.write_fmt(format_args!("{},{}\n", x, y));
    }
}

fn main() {
    //    // Test on actual map AutomatonLE.txt
    let result = pathfinding_test::jps::read_grid_from_file(String::from("AutomatonLE.txt"));
    let (array, height, width) = result.unwrap();
    //        let source = pathfinding_test::jps::Point2d { x: 70, y: 100 };
    //        let target = pathfinding_test::jps::Point2d { x: 100, y: 114 };

    let source = pathfinding_test::jps::Point2d { x: 29, y: 65 };
    let target = pathfinding_test::jps::Point2d { x: 154, y: 114 };

    //                    let source = pathfinding_test::jps::Point2d { x: 32, y: 51 };
    //                    let target = pathfinding_test::jps::Point2d { x: 150, y: 129 };
    let path = pathfinding_test::jps::jps_test(array, source, target);
    println!("Path: {:?}", path);

    // Test on empty 100x100 grid
    //            let source = pathfinding_test::jps::Point2d { x: 5, y: 5 };
    //            let target = pathfinding_test::jps::Point2d { x: 10, y: 12 };
    //            let grid = pathfinding_test::jps::grid_setup(15);
    //            let path = pathfinding_test::jps::jps_test(grid, source, target);
    //            println!("Path: {:?}", path);
    if path.is_some() {
        write_path_to_file(path.unwrap());
    }
}
