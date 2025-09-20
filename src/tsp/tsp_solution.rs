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
        let mut cost = 0.0;

        for pairs in self.path.windows(2) {
            cost += self.instance.dist_from_to(pairs[0], pairs[1]);
        }
        cost
    }

    pub fn is_valid(&self) -> bool {
        self.path.len() == &self.instance.len() + 1 && self.is_valid_subsolution()
    }

    pub fn is_valid_subsolution(&self) -> bool {
        if self.path.is_empty() {
            return true;
        }

        let mut time = 0.0;
        let mut last_visited = self.path[0];
        let mut visited = HashSet::new();
        visited.insert(last_visited);

        if self.instance.window_of(last_visited).0 > 0.0 {
            println!("Invalid subsolution because first node's time_window starts at > 0.0.");
            return false;
        }

        for (cnt, &node) in self.path[1..self.path.len()].iter().enumerate() {
            if !visited.insert(node) && cnt < self.path.len() - 2 {
                println!(
                    "Invalid sobsolution because node {} was already visited",
                    node
                );
                return false;
            }

            time += self.instance.dist_from_to(last_visited, node);

            let (start_time, end_time) = self.instance.window_of(node);
            if !(start_time..=end_time).contains(&time) {
                println! {"Invalid subsolution because start/end time {start_time}/{end_time} of time window does not contain {time}"}
                return false;
            }

            last_visited = node;
        }
        if let Some(last) = self.path.last()
            && self.path.len() == self.get_instance().len() + 1
        {
            if *last != self.path[0] {
                println!("Invalid subsolution because last node is not start node");
                return false;
            }
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
            vec![(0.0, 101.0), (1.0, 100.0)],
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
        assert!(!invalid_solution.is_valid_subsolution());
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
        let invalid_solution = TSPSolution::new(create_test_instance(), vec![1]);

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
}
