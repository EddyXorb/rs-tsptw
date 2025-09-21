mod beamsearch;
mod tsp;

use std::path::PathBuf;
use std::sync::Arc;
use tsp::{TSPInstance, TSPSolution, solve_tsp};

fn main() {
    let instance =
        TSPInstance::from_file(PathBuf::from("instances/SolomonPotvinBengio/rc_201.1.txt"));
    // println!("{instance}");

    // let sol = TSPSolution::new(
    //     Arc::new(instance),
    //     vec![
    //         0, 13, 21, 10, 23, 9, 12, 5, 6, 8, 16, 19, 25, 17, 18, 1, 24, 2, 11, 7, 4, 3, 14, 20,
    //         22, 15, 0,
    //     ],
    // );

    //rc_201.1
    // let sol = TSPSolution::new(
    //     Arc::new(instance),
    //     vec![
    //         0, 14, 18, 13, 9, 5, 4, 6, 8, 7, 16, 19, 11, 17, 1, 10, 3, 12, 2, 15, 0,
    //     ],
    // );
    // println!("{sol}");
    // print!("{}", sol.is_valid());

    // println!("Sol has cost {}", sol.get_cost());
    // sol.print_times();
    // sol.is_valid();
    let result = solve_tsp(instance, 1000);
    let sol = result.unwrap();
    println!("{sol}");
    // print!("{:?}", sol.get_path());
}
