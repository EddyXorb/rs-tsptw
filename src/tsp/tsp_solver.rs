use std::sync::Arc;

use super::super::beamsearch::beamsearch_solver::{BeamsearchNode, BeamsearchSolver, Node, Params};
use super::tsp_instance::TSPInstance;
use super::tsp_solution::TSPSolution;

struct TSPNode {
    pub time: f64,
    pub target: usize,
    pub distance: f64,
}

impl BeamsearchNode for TSPNode {
    fn fitness(&self) -> f64 {
        self.distance
    }

    fn level(&self) -> f64 {
        self.time
    }
}

fn expander(node: &Node<TSPNode>, instance: &TSPInstance) -> Vec<TSPNode> {
    let time = node.data().time;
    let last_target = node.data().target;
    let visited_nodes: Vec<usize> = node.ancestors().map(|x| x.data().target).collect();
    let remaining_nodes = (0..instance.len()).filter(|i| {
        !visited_nodes.contains(i)
            && instance.window_of_contains(*i, time + instance.dist_from_to(last_target, *i))
    });

    remaining_nodes
        .map(|next_target| TSPNode {
            time: instance.dist_from_to(last_target, next_target),
            target: next_target,
            distance: node.data().distance + instance.dist_from_to(last_target, next_target),
        })
        .collect()
}

pub fn solve_tsp(instance: TSPInstance) -> Option<TSPSolution> {
    let start_node = TSPNode {
        time: 0.0,
        target: 0,
        distance: 0.0,
    };

    let result = BeamsearchSolver::new(
        vec![start_node],
        |node| expander(node, &instance),
        |x, y| false,
        Params { beam_width: 100 },
    )
    .solve();

    if !result.best.is_some() {
        return None;
    }

    let best_node = result.best.unwrap();
    println!(
        "Found best result with nr_expansions {} and distance {}",
        result.nr_expansions,
        &best_node.data().distance
    );
    let mut path: Vec<usize> = best_node
        .ancestors()
        .map(|node| node.data().target)
        .collect();
    path.reverse();

    let solution = TSPSolution::new(Arc::new(instance), path);

    println! {"Best solution: {:?}",solution.get_path()};
    assert!(solution.is_valid_subsolution());

    Some(solution)
}

#[cfg(test)]
mod tests {
    use crate::{
        beamsearch::beamsearch_solver::Node,
        tsp::{
            TSPInstance,
            tsp_solver::{TSPNode, expander, solve_tsp},
        },
    };

    fn create_test_instance() -> TSPInstance {
        TSPInstance::new(
            3,
            vec![
                vec![0.0, 1000.0, 1.0],
                vec![1000.0, 0.0, 1000.0],
                vec![1000.0, 1000.0, 0.0],
            ],
            vec![(0.0, 1.0), (1.0, 2000.0), (1.0, 2000.0)],
        )
    }

    fn create_small_instance() -> TSPInstance {
        TSPInstance::new(
            2,
            vec![vec![0.0, 1.0], vec![2.0, 0.0]],
            vec![(0.0, 1.0), (1.0, 100.0)],
        )
    }

    #[test]
    pub fn expander_works() {
        let instance = create_small_instance();
        let node = Node::new_root(TSPNode {
            time: 0.0,
            target: 0,
            distance: 0.0,
        });
        let expanded = expander(&node, &instance);

        assert_eq!(expanded.len(), 1);
        let node = &expanded[0];
        assert_eq!(node.distance, 1.0);
        assert_eq!(node.time, 1.0);
        assert_eq!(node.target, 1);
    }

    #[test]
    pub fn simple_test() {
        let instance = create_test_instance();

        let result = solve_tsp(instance);

        assert!(result.is_some());

        let sol = result.unwrap();

        print!("{:?}", sol.get_instance());
        assert_eq!(*sol.get_path(), vec![0, 2, 1]);
        assert_eq!(sol.get_cost(), 1001.0);
        assert!(sol.is_valid());
    }
}
