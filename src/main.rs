mod tsp;

use std::path::PathBuf;
use tsp::TSPInstance;

fn main() {
    println!(
        "{}",
        TSPInstance::create_from_file(PathBuf::from("instances/SolomonPotvinBengio/rc_201.1.txt"))
    )
}
