mod beamsearch;
mod tsp;

use std::path::PathBuf;
use std::sync::Arc;
use tsp::solve_tsp;
use tsp::{TSPInstance, TSPSolution};

fn main() {
    let instance =
        TSPInstance::from_file(PathBuf::from("instances/SolomonPotvinBengio/rc_201.1.txt"));
    //println!("{instance}");

    let sol = TSPSolution::new(
        Arc::new(instance),
        vec![
            0, 14, 18, 13, 9, 5, 4, 6, 8, 7, 16, 19, 11, 17, 1, 10, 3, 12, 2, 15, 0,
        ],
    );

    println!("Sol has cost {}", sol.get_cost());
    sol.print_times();
    sol.is_valid();
    // let result = solve_tsp(instance);
    // print!("{:?}", result.unwrap().get_path());
}
