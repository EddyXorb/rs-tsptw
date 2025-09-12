use std::collections::HashSet;
use std::hash::Hash;
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

    pub fn is_valid(&self) -> bool {
        if &self.path.len() < &self.instance.len() {
            return false;
        }
        return self.is_valid_subsolution();
    }

    pub fn is_valid_subsolution(&self) -> bool {
        if self.path.len() < self.instance.len() {
            return false;
        }

        let mut time: f64 = 0.0;
        let mut last_visited = self.path[0];
        let mut visited = HashSet::new();
        visited.insert(last_visited);
        if self.instance.window_of(last_visited).0 > 0.0 {
            return false;
        }

        for node in &self.path[1..] {
            if visited.contains(node) {
                return false;
            }

            visited.insert(*node);
            time += self.instance.dist_from_to(last_visited, *node);
            last_visited = *node;

            let current_window = self.instance.window_of(*node);
            if time < current_window.0 || current_window.1 < time {
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
            vec![vec![1.0, 1.0], vec![2.0, 2.0]],
            vec![(0.0, 1.0), (1.0, 100.0)],
        ))
    }

    #[test]
    fn test_valid_solution() {
        let valid_solution = TSPSolution::new(create_test_instance(), vec![0, 1]);

        assert!(valid_solution.is_valid_subsolution());
        assert!(valid_solution.is_valid());

        let invalid_solution = TSPSolution::new(create_test_instance(), vec![1, 0]);

        assert!(!invalid_solution.is_valid());
        assert!(!invalid_solution.is_valid_subsolution());
    }
}
