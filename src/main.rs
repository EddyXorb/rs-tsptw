#![allow(dead_code)]
#![allow(unused_imports)]

mod beamsearch;
mod tsp;

use std::sync::Arc;
use std::{fs::read, path::PathBuf};
use tsp::{TSPInstance, TSPSolution, solve_tsp};

pub struct BestKnown<'a> {
    pub best: TSPSolution<'a>,
    pub name: String,
}

pub fn read_all_instances(
    best_known_file: PathBuf,
) -> (Vec<Box<TSPInstance>>, Vec<BestKnown<'static>>) {
    assert!(best_known_file.is_file());

    let mut best_knowns = Vec::<BestKnown>::new();
    let mut instances = Vec::<Box<TSPInstance>>::new();

    let raw = std::fs::read_to_string(best_known_file).unwrap();
    let lines = raw.lines().skip(1);
    for line in lines {
        // rc_201.1.txt             444.54  0  14 18 13 9 5 4 6 8 7 16 19 11 17 1 10 3 12 2 15
        let cols: Vec<_> = line.split_whitespace().collect();
        let instance_file = cols[0];
        let best_dist: f64 = cols[1].parse().unwrap();
        let best_path: Vec<usize> = cols[2..]
            .to_vec()
            .into_iter()
            .map(|n| n.parse().unwrap())
            .chain(std::iter::once(0))
            .collect();

        let instance = Box::new(TSPInstance::from_file(PathBuf::from(format!(
            "instances/SolomonPotvinBengio/{}",
            &instance_file
        ))));
        instances.push(instance);

        let sol = TSPSolution::new(&instance, best_path);

        if (sol.get_time_distance().dist - best_dist).abs() > 0.1
        // check that path and best known are correctly calculated
        {
            println!(
                "Instance {instance_file} has best known distance {best_dist}, but calculatedsolution distance {}",
                sol.get_time_distance().dist
            );
        }

        best_knowns.push(BestKnown {
            best: sol,
            name: instance_file.to_string(),
        });
    }
    (instances, best_knowns)
}

fn main() {
    let (instances, best_knowns) = read_all_instances(PathBuf::from(
        "instances/SolomonPotvinBengio/best_known.txt",
    ));

    println!("Read {} instances", best_knowns.len());

    for best in best_knowns {
        println!("Going to solve {}..", best.name);

        let result = solve_tsp(
            best.best.get_instance(),
            beamsearch::Params {
                beam_width: 1000000,
                prune_similars: true,
            },
        );
        if let Some(sol) = result {
            println!("{sol}");
        } else {
            println!("Did not find a valid solution.");
        }
    }
    // println!("Sol has cost {}", sol.get_cost());
    // sol.print_times();
    // sol.is_valid();
    // let result = solve_tsp(
    //     instance,
    //     beamsearch::Params {
    //         beam_width: 1000000,
    //         prune_similars: true,
    //     },
    // );
    // if let Some(sol) = result {
    //     println!("{sol}");
    // } else {
    //     println!("Did not find a valid solution.");
    // }
    // print!("{:?}", sol.get_path());
}
