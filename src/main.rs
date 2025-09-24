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

#[derive(Debug)]
struct SolutionResult {
    solution_type: SolutionType,
    duration_secs: f64,
}
fn main() {
    init_logger();

    let best_knowns = read_all_instances(PathBuf::from(
        "instances/SolomonPotvinBengio/best_known.txt",
    ));

    info!("Read {} instances", best_knowns.len());

    let mut solution_results = HashMap::<String, SolutionResult>::new();

    for best in best_knowns {
        info!("Going to solve {}..", &best.name);

        let start_time = std::time::Instant::now();

        let result = solve_tsp(
            best.solution.get_instance().clone(),
            beamsearch::Params {
                beam_width: 100,
                prune_similars: true,
            },
        );

        let duration_secs = start_time.elapsed().as_secs_f64();

        if let Some(sol) = result {
            let solution_type = if sol.get_time_distance().dist
                < best.solution.get_time_distance().dist - 0.01
            {
                info!("FOUND BETTER SOLUTION THAN BEST KNOWN!");
                SolutionType::Better
            } else if sol.get_time_distance().dist > best.solution.get_time_distance().dist + 0.01 {
                info!("Worse solution found.");
                SolutionType::Worse
            } else {
                info!("Equally good solution found.");
                SolutionType::Equal
            };

            solution_results.insert(
                best.name.clone(),
                SolutionResult {
                    solution_type,
                    duration_secs,
                },
            );

            info!(
                "Found solution for {} with distance {} compared to {} in best known. (took {:.2}s)",
                &best.name,
                sol.get_time_distance().dist,
                best.solution.get_time_distance().dist,
                duration_secs
            )
        } else {
            info!("Did not find a valid solution.");

            solution_results.insert(
                best.name,
                SolutionResult {
                    solution_type: SolutionType::NotFound,
                    duration_secs,
                },
            );
        }
    }

    // Write results to last_result.txt
    let mut result_content = String::new();

    // Collect into vector and sort by name
    let mut sorted_results: Vec<(&String, &SolutionResult)> = solution_results.iter().collect();
    sorted_results.sort_by_key(|(name, _)| *name);

    for (name, solution_result) in sorted_results {
        result_content.push_str(&format!(
            "{:<15} {:>8} {:>8.2}s\n",
            name, 
            format!("{:?}", solution_result.solution_type), 
            solution_result.duration_secs
        ));
    }

    std::fs::write("last_result.txt", &result_content).expect("Failed to write last_result.txt");

    info!("\n{result_content}");
}
