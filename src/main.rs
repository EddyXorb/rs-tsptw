mod beamsearch;
mod tsp;

use std::path::PathBuf;
use tsp::TSPInstance;
use tsp::solve_tsp;

fn main() {
    let instance =
        TSPInstance::from_file(PathBuf::from("instances/SolomonPotvinBengio/rc_207.4.txt"));
    println!("{instance}");

    let result = solve_tsp(instance);
    print!("{:?}", result.unwrap().get_path());
}
