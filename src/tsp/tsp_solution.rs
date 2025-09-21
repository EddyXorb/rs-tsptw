use std::collections::HashSet;
use std::sync::Arc;

use super::TSPInstance;

pub struct TSPSolution {
    instance: Arc<TSPInstance>,
    path: Vec<usize>,
}

impl TSPSolution {
    pub fn new(instance: Arc<TSPInstance>, path: Vec<usize>) -> Self {
        Self { instance, path }
    }

    pub fn get_instance(&self) -> &TSPInstance {
        &self.instance
    }

    pub fn get_path(&self) -> &Vec<usize> {
        &self.path
    }

    pub fn get_cost(&self) -> f64 {
        let mut cost = self.instance.window_of(self.path[0]).0;

        for pairs in self.path.windows(2) {
            cost += self.instance.dist_from_to(pairs[0], pairs[1]);
            cost = cost.max(self.instance.window_of(pairs[1]).0); // for wait time
        }
        cost
    }

    pub fn print_times(&self) {
        let mut sum: f64 = 0.0;
        for node in self.path.windows(2) {
            let dist = self.get_instance().dist_from_to(node[0], node[1]);
            sum += dist;
            let wait_time = sum.max(self.get_instance().window_of(node[1]).0) - sum;
            sum += wait_time;
            println!(
                "{:3} -> {:3} : current time {:<7.2} wait time {:<7.2} time sum {:<7.2} time window {:?}",
                node[0],
                node[1],
                dist,
                wait_time,
                sum,
                self.get_instance().window_of(node[1])
            );
        }
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
                println!(
                    "Invalid subsolution because node {} was already visited",
                    node
                );
                return false;
            }

            time += self.instance.dist_from_to(last_visited, node);

            let (start_time, end_time) = self.instance.window_of(node);
            if time > end_time {
                println! {"Invalid subsolution in node {cnt} (which is city {node}) because end time {end_time} of time window does not contain {time}"}
                return false;
            }

            time = time.max(start_time); // if we arrive too early we have to wait.

            last_visited = node;
        }
        if let Some(last) = self.path.last()
            && self.path.len() == self.get_instance().len() + 1
            && *last != self.path[0]
        {
            println!("Invalid subsolution because last node is not start node");
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
            vec![(0.0, 101.0), (1.0, 2.0)],
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

        assert_eq!(valid_solution.get_cost(), 1.0);
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

        assert_eq!(sol.get_cost(), 100.0);
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

        assert_eq!(sol.get_cost(), 2000.0);
        assert!(sol.is_valid())
    }
}
