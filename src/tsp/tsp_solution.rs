use std::collections::HashSet;
use std::fmt::Display;
use std::iter::zip;
use std::ops::Add;
use std::sync::Arc;
use log::warn;

use super::TSPInstance;

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct TimeDist {
    pub time: f64,
    pub dist: f64,
}

impl Add for TimeDist {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        TimeDist {
            time: self.time + rhs.time,
            dist: self.dist + rhs.dist,
        }
    }
}

pub struct TSPSolution {
    instance: Arc<TSPInstance>,
    path: Vec<usize>,
}

impl Display for TSPSolution {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let time_distance_diffs = self.get_time_distance_diffs();
        let mut overall_dist = 0.0;
        let mut overall_time = 0.0;
        let mut overall_wait_time = 0.0;
        for (time_dist, start_end) in zip(time_distance_diffs, self.path.windows(2)) {
            let wait_time = time_dist.time - time_dist.dist;
            overall_dist += time_dist.dist;
            overall_time += time_dist.time;
            overall_wait_time += wait_time;

            writeln!(
                f,
                "{:3} -> {:3} : time {:<7.2} dist {:<7.2} wait time {:<7.2} time sum {:<7.2} dist sum {:<7.2} wait sum {:<7.2} time window {:?}",
                start_end[0],
                start_end[1],
                time_dist.time,
                time_dist.dist,
                wait_time.max(0.0),
                overall_time,
                overall_dist,
                overall_wait_time,
                self.instance.window_of(start_end[1])
            ).unwrap();
        }
        Ok(())
    }
}

impl TSPSolution {
    pub fn new(instance: Arc<TSPInstance>, path: Vec<usize>) -> Self {
        Self { instance, path }
    }

    pub fn get_instance(&self) -> &Arc<TSPInstance> {
        &self.instance
    }

    pub fn get_path(&self) -> &Vec<usize> {
        &self.path
    }

    pub fn get_time_distance_diffs(&self) -> Vec<TimeDist> {
        let mut time = 0.0;

        let mut out: Vec<TimeDist> = vec![];
        for pairs in self.path.windows(2) {
            let start = pairs[0];
            let end = pairs[1];
            let next_distance = self.instance.dist_from_to(start, end);
            let time_diff = (time + next_distance).max(self.instance.window_of(end).0) - time;
            time += time_diff;
            out.push(TimeDist {
                time: time_diff,
                dist: next_distance,
            });
        }
        out
    }

    pub fn get_time_distance(&self) -> TimeDist {
        self.get_time_distance_diffs().iter().fold(
            TimeDist {
                time: self.instance.window_of(self.path[0]).0,
                dist: 0.0,
            },
            |x, y| x + y.clone(),
        )
    }

    pub fn is_valid(&self) -> bool {
        // if we have only one city, this is already a roundtrip, otherwise we need one more step to get back to the deposit.
        self.path.len() == &self.instance.len() + (if self.instance.len() == 1 { 0 } else { 1 })
            && self.is_valid_subsolution()
    }

    pub fn is_valid_subsolution(&self) -> bool {
        if self.path.is_empty() {
            return true;
        }

        let mut last_visited = self.path[0];
        let mut time = f64::max(0.0, self.instance.window_of(last_visited).0);
        let mut visited = HashSet::new();
        visited.insert(last_visited);

        for (cnt, &node) in self.path[1..self.path.len()].iter().enumerate() {
            if !visited.insert(node) && cnt < self.path.len() - 2 {
                warn!(
                    "Invalid subsolution because node {} was already visited",
                    node
                );
                return false;
            }

            time += self.instance.dist_from_to(last_visited, node);

            let (start_time, end_time) = self.instance.window_of(node);
            if time > end_time {
                warn!("Invalid subsolution in node {cnt} (which is city {node}) because end time {end_time} of time window does not contain {time}");
                return false;
            }

            time = time.max(start_time); // if we arrive too early we have to wait.

            last_visited = node;
        }
        if let Some(last) = self.path.last()
            && self.path.len() == self.get_instance().len() + 1
            && *last != self.path[0]
        {
            warn!("Invalid subsolution because last node is not start node");
            return false;
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_instance() -> Arc<TSPInstance> {
        Arc::new(TSPInstance::new(
            2,
            vec![vec![0.0, 1.0], vec![2.0, 0.0]],
            vec![(0.0, 101.0), (2.0, 2.0)],
        ))
    }

    #[test]
    fn test_valid_solution() {
        let valid_solution = TSPSolution::new(create_test_instance(), vec![0, 1, 0]);

        assert!(valid_solution.is_valid_subsolution());
        assert!(valid_solution.is_valid());
    }

    #[test]
    fn test_invalid_solution() {
        let invalid_solution = TSPSolution::new(create_test_instance(), vec![1, 0, 1]);

        assert!(!invalid_solution.is_valid());
    }

    #[test]
    fn test_invalid_solution_because_no_roundtrip() {
        let invalid_solution = TSPSolution::new(create_test_instance(), vec![0, 1]);

        assert!(!invalid_solution.is_valid());
    }

    #[test]
    fn test_valid_subsolution() {
        let valid_solution = TSPSolution::new(create_test_instance(), vec![0]);

        assert!(valid_solution.is_valid_subsolution());
    }

    #[test]
    fn test_invalid_subsolution() {
        let invalid_solution = TSPSolution::new(create_test_instance(), vec![1, 0, 1]);

        assert!(!invalid_solution.is_valid_subsolution());
    }

    #[test]
    fn test_empty_subsolution_is_valid() {
        let empty_solution = TSPSolution::new(create_test_instance(), vec![]);

        assert!(empty_solution.is_valid_subsolution());
    }

    #[test]
    fn test_cost_works() {
        let valid_solution = TSPSolution::new(create_test_instance(), vec![0, 1]);

        assert_eq!(
            valid_solution.get_time_distance(),
            TimeDist {
                time: 2.0,
                dist: 1.0
            }
        );
    }

    #[test]
    fn test_special_case_single_city_is_valid() {
        let single_city_instance =
            Arc::new(TSPInstance::new(1, vec![vec![0.0]], vec![(0.0, 100.0)]));
        let sol = TSPSolution::new(single_city_instance, vec![0]);

        assert!(sol.is_valid())
    }

    #[test]
    fn test_lower_bound_of_time_window_makes_visitor_wait_if_arrives_too_early_for_single_city() {
        let single_city_instance =
            Arc::new(TSPInstance::new(1, vec![vec![0.0]], vec![(100.0, 101.0)]));
        let sol = TSPSolution::new(single_city_instance, vec![0]);

        assert_eq!(
            sol.get_time_distance(),
            TimeDist {
                time: 100.0,
                dist: 0.0
            }
        );
        assert!(sol.is_valid())
    }

    #[test]
    fn test_lower_bound_of_time_window_makes_visitor_wait_if_arrives_too_early() {
        let two_city_instance = Arc::new(TSPInstance::new(
            2,
            vec![vec![0.0, 0.0], vec![0.0, 0.0]],
            vec![(0.0, 2000.0), (2000.0, 3000.0)],
        ));
        let sol = TSPSolution::new(two_city_instance, vec![0, 1, 0]);

        assert_eq!(
            sol.get_time_distance(),
            TimeDist {
                time: 2000.0,
                dist: 0.0
            }
        );
        assert!(sol.is_valid())
    }

    #[test]
    fn test_time_distance_diffs() {
        let instance = create_test_instance();
        let sol = TSPSolution::new(instance, vec![0, 1, 0]);

        let time_distances = sol.get_time_distance_diffs();
        assert_eq!(
            time_distances,
            vec![
                TimeDist {
                    time: 2.0,
                    dist: 1.0
                },
                TimeDist {
                    time: 2.0,
                    dist: 2.0
                }
            ]
        );
    }
}
