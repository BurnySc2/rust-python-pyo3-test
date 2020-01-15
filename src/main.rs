#![feature(cell_update)]
// Testing and benchmark crate
#![feature(test)]
extern crate test;

pub mod pathfinding_test;

fn main() {
    let result = pathfinding_test::jps::read_grid_from_file(String::from("src/AutomatonLE.txt"));
    let (array, height, width) = result.unwrap();
    let source = pathfinding_test::jps::Point2d{x:32, y:51};
    let target = pathfinding_test::jps::Point2d{x:150, y:129};
    let path = pathfinding_test::jps::jps_test(array, source, target);
    println!("Path: {:?}", path);
//
//    let grid = pathfinding_test::jps::grid_setup(100);
//    pathfinding_test::jps::jps_test(grid);
}
