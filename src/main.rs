#![allow(dead_code)]
#![allow(unused_imports)]

mod beamsearch;
mod tsp;

use std::collections::HashMap;
use std::sync::Arc;
use std::{fs::read, path::PathBuf};
use tsp::{TSPInstance, TSPSolution, solve_tsp};

pub struct BestKnown {
    pub solution: TSPSolution,
    pub name: String,
}

pub fn read_all_instances(best_known_file: PathBuf) -> Vec<BestKnown> {
    assert!(best_known_file.is_file());

    let mut best_knowns = Vec::<BestKnown>::new();

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

        let instance = Arc::new(TSPInstance::from_file(PathBuf::from(format!(
            "instances/SolomonPotvinBengio/{}",
            &instance_file
        ))));

        let solution = TSPSolution::new(instance.clone(), best_path);

        if (solution.get_time_distance().dist - best_dist).abs() > 0.1
        // check that path and best known are correctly calculated
        {
            println!(
                "Instance {instance_file} has best known distance {best_dist}, but calculatedsolution distance {}",
                solution.get_time_distance().dist
            );
        }

        best_knowns.push(BestKnown {
            solution,
            name: instance_file.to_string(),
        });
    }
    best_knowns
}

#[derive(Debug)]
enum SolutionType {
    Better,
    Equal,
    Worse,
    NotFound,
}
fn main() {
    let best_knowns = read_all_instances(PathBuf::from(
        "instances/SolomonPotvinBengio/best_known.txt",
    ));

    println!("Read {} instances", best_knowns.len());

    let mut solutionTypes = HashMap::<String, SolutionType>::new();

    for best in best_knowns {
        println!("Going to solve {}..", &best.name);

        let result = solve_tsp(
            best.solution.get_instance().clone(),
            beamsearch::Params {
                beam_width: 1000000,
                prune_similars: true,
            },
        );
        if let Some(sol) = result {
            if sol.get_time_distance().dist < best.solution.get_time_distance().dist - 0.01 {
                println!("FOUND BETTER SOLUTION THAN BEST KNOWN!");

                solutionTypes
                    .entry(best.name.clone())
                    .or_insert(SolutionType::Better);
            } else if sol.get_time_distance().dist > best.solution.get_time_distance().dist + 0.01 {
                println!("Worse solution found.");

                solutionTypes
                    .entry(best.name.clone())
                    .or_insert(SolutionType::Worse);
            } else {
                println!("Equally good solution found.");

                solutionTypes
                    .entry(best.name.clone())
                    .or_insert(SolutionType::Equal);
            }
            println!(
                "Found solution for {} with distance {} compared to {} in best known.",
                &best.name,
                sol.get_time_distance().dist,
                best.solution.get_time_distance().dist
            )
        } else {
            println!("Did not find a valid solution.");

            solutionTypes
                .entry(best.name)
                .or_insert(SolutionType::NotFound);
        }
    }
    println!("Summary {:?}", solutionTypes);
}
