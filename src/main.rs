#![allow(dead_code)]
#![allow(unused_imports)]

mod beamsearch;
mod tsp;

use env_logger::Builder;
use log::{info, warn};
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io;
use std::io::Write;
use std::sync::Arc;
use std::{fs::read, path::PathBuf};
use tsp::{TSPInstance, TSPSolution, solve_tsp};

fn init_logger() {
    let log_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("main_run.log")
        .expect("Failed to open log file");

    Builder::from_default_env()
        .filter_level(log::LevelFilter::Debug)
        .target(env_logger::Target::Pipe(Box::new(DualWriter::new(
            log_file,
        ))))
        .format_timestamp_secs()
        .init();
}

struct DualWriter<W> {
    file: W,
}

impl<W: Write> DualWriter<W> {
    fn new(file: W) -> Self {
        DualWriter { file }
    }
}

impl<W: Write> Write for DualWriter<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        // Write to console
        io::stdout().write_all(buf)?;
        io::stdout().flush()?;

        // Write to file
        self.file.write_all(buf)?;
        self.file.flush()?;

        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        io::stdout().flush()?;
        self.file.flush()?;
        Ok(())
    }
}

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
            warn!(
                "Instance {instance_file} has best known distance {best_dist}, but calculated solution distance {}",
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
    init_logger();

    let best_knowns = read_all_instances(PathBuf::from(
        "instances/SolomonPotvinBengio/best_known.txt",
    ));

    info!("Read {} instances", best_knowns.len());

    let mut solution_types = HashMap::<String, SolutionType>::new();

    for best in best_knowns {
        info!("Going to solve {}..", &best.name);

        let result = solve_tsp(
            best.solution.get_instance().clone(),
            beamsearch::Params {
                beam_width: 1000000,
                prune_similars: true,
            },
        );
        if let Some(sol) = result {
            if sol.get_time_distance().dist < best.solution.get_time_distance().dist - 0.01 {
                info!("FOUND BETTER SOLUTION THAN BEST KNOWN!");

                solution_types
                    .entry(best.name.clone())
                    .or_insert(SolutionType::Better);
            } else if sol.get_time_distance().dist > best.solution.get_time_distance().dist + 0.01 {
                info!("Worse solution found.");

                solution_types
                    .entry(best.name.clone())
                    .or_insert(SolutionType::Worse);
            } else {
                info!("Equally good solution found.");

                solution_types
                    .entry(best.name.clone())
                    .or_insert(SolutionType::Equal);
            }
            info!(
                "Found solution for {} with distance {} compared to {} in best known.",
                &best.name,
                sol.get_time_distance().dist,
                best.solution.get_time_distance().dist
            )
        } else {
            info!("Did not find a valid solution.");

            solution_types
                .entry(best.name)
                .or_insert(SolutionType::NotFound);
        }
    }
    info!("Summary {:?}", solution_types);
}
