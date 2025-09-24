use std::sync::Arc;

use super::super::beamsearch::beamsearch_solver::{BeamsearchNode, BeamsearchSolver, Node, Params};
use super::tsp_instance::TSPInstance;
use super::tsp_solution::TSPSolution;
use super::tsp_utility::calc_commutative_hash;

struct TSPNode {
    pub time: f64,
    pub dist: f64,
    pub target: usize,
    pub visited_node_hash: u32,
}

impl BeamsearchNode for TSPNode {
    fn fitness(&self) -> f64 {
        self.dist
    }

    fn level(&self) -> f64 {
        self.time
    }
}

fn make_tsp_solution_from_node(instance: Arc<TSPInstance>, node: &Node<TSPNode>) -> TSPSolution {
    let mut path: Vec<usize> = node.ancestors().map(|node| node.data().target).collect();
    path.reverse();
    TSPSolution::new(instance, path)
}

fn expand(node: &Node<TSPNode>, instance: &TSPInstance) -> Vec<TSPNode> {
    let time = node.data().time;
    let dist = node.data().dist;

    let last_target = node.data().target;
    let visited_nodes: Vec<usize> = node.ancestors().map(|x| x.data().target).collect();

    let reached_end = || {
        visited_nodes.len()
            == if instance.len() > 1 {
                instance.len() + 1
            } else {
                1
            }
    };

    if reached_end() {
        return Vec::new();
    }

    let remaining_nodes: Vec<_> = (0..instance.len())
        .filter(|i| {
            !visited_nodes.contains(i)
                || (visited_nodes.len() == instance.len() && i == visited_nodes.last().unwrap())
        })
        .collect();

    if remaining_nodes
        .iter()
        .any(|i| instance.window_of(*i).1 < time)
    // if one of the nodes cannot be targeted in future because of missed window
    {
        return Vec::new();
    }

    let max_possible_time = remaining_nodes
        .iter()
        .map(|n| instance.window_of(*n).1 as i32)
        .min()
        .unwrap() as f64;

    let get_next_time_for = |next_target| {
        (node.data().time + instance.dist_from_to(last_target, next_target))
            .max(instance.window_of(next_target).0)
    };

    let expanded_nodes: Vec<_> = remaining_nodes
        .into_iter()
        .filter(|next_target| get_next_time_for(*next_target) <= max_possible_time)
        .map(|next_target| TSPNode {
            time: get_next_time_for(next_target),
            target: next_target,
            dist: dist + instance.dist_from_to(last_target, next_target),
            visited_node_hash: calc_commutative_hash(node.data().visited_node_hash, next_target),
        })
        .collect();

    expanded_nodes
}

fn is_similar(a: &Node<TSPNode>, b: &Node<TSPNode>) -> bool {
    if a.data().target != b.data().target {
        return false;
    }

    let mut a_cities: Vec<_> = a.ancestors().map(|node| node.data().target).collect();
    let mut b_cities: Vec<_> = b.ancestors().map(|node| node.data().target).collect();

    a_cities.sort();
    b_cities.sort();

    a_cities == b_cities
}

pub fn solve_tsp(instance: Arc<TSPInstance>, params: Params) -> Option<TSPSolution> {
    let start_node = TSPNode {
        time: instance.window_of(0).0,
        target: 0,
        dist: 0.0,
        visited_node_hash: 1,
    };

    let result = BeamsearchSolver::new(
        vec![start_node],
        |node| expand(node, &instance),
        |x, y| x.data().target == y.data().target && (x.data().time - y.data().time).abs() < 1.0,
        |n| n.data().visited_node_hash,
        |n| make_tsp_solution_from_node(instance.clone(), n).is_valid(),
        params,
    )
    .solve();

    if result.best.is_none() {
        return None;
    }

    let best_node = result.best.unwrap();
    println!(
        "Found best result with distance {},  nr_expansions {} and time {}",
        &best_node.data().dist,
        result.nr_expansions,
        &best_node.data().time
    );

    let solution = make_tsp_solution_from_node(instance, &best_node);

    println! {"Best solution: {:?}",solution.get_path()};
    assert!(solution.is_valid_subsolution());

    Some(solution)
}

#[cfg(test)]
mod tests {
    use super::super::super::tsp::TimeDist;
    use super::*;

    fn create_test_instance() -> TSPInstance {
        // Optimal: 0 -> 2 -> 1 -> 0, with total cost 1200 and 4 / 100 waiting times in first two steps
        TSPInstance::new(
            3,
            vec![
                vec![0.0, 1000.0, 1.0],
                vec![1000.0, 0.0, 1000.0],
                vec![1000.0, 100.0, 0.0],
            ],
            vec![(0.0, 1200.0), (200.0, 2000.0), (5.0, 2000.0)],
        )
    }

    fn create_small_instance() -> TSPInstance {
        TSPInstance::new(
            2,
            vec![vec![0.0, 1.0], vec![2.0, 0.0]],
            vec![(0.0, 1.0), (2.0, 100.0)],
        )
    }

    #[test]
    pub fn expander_works() {
        let instance = create_small_instance();
        let node = Node::new_root(TSPNode {
            time: 0.0,
            target: 0,
            dist: 0.0,
            visited_node_hash: 0,
        });
        let expanded = expand(&node, &instance);

        assert_eq!(expanded.len(), 1);
        let node = &expanded[0];
        assert_eq!(node.time, 2.0);
        assert_eq!(node.target, 1);
        assert_eq!(node.dist, 1.0);
    }

    #[test]
    pub fn expander_does_not_consider_nodes_too_far_in_future() {
        // we start at 0. We can only expand 1.
        // 2 would result in time 2001, because of window 2000, which would make 1 unfullfillable
        // 3 would result in time 2001, because of distance 2000, which would make 1 unfullfillable
        let instance = TSPInstance::new(
            4,
            vec![
                vec![0.0, 0.0, 0.0, 2001.0],
                vec![0.0, 0.0, 0.0, 0.0],
                vec![0.0, 0.0, 0.0, 0.0],
                vec![0.0, 0.0, 0.0, 0.0],
            ],
            vec![
                (0.0, 3000.0),
                (1000.0, 2000.0),
                (2001.0, 3000.0),
                (0.0, 4000.0),
            ],
        );
        let node = Node::new_root(TSPNode {
            time: 0.0,
            target: 0,
            dist: 0.0,
            visited_node_hash: 0,
        });
        let expanded = expand(&node, &instance);

        assert_eq!(expanded.len(), 1);
        let node = &expanded[0];
        assert_eq!(node.time, 1000.0);
        assert_eq!(node.target, 1);
        assert_eq!(node.dist, 0.0);
    }

    #[test]
    pub fn simple_test() {
        let instance = create_test_instance();

        let result = solve_tsp(
            Arc::new(instance),
            Params {
                beam_width: 100,
                prune_similars: true,
            },
        );

        assert!(result.is_some());

        let sol = result.unwrap();

        println!("{sol}");
        assert_eq!(*sol.get_path(), vec![0, 2, 1, 0]);
        assert_eq!(
            sol.get_time_distance(),
            TimeDist {
                time: 1200.0,
                dist: 1101.0
            }
        );
        assert!(sol.is_valid());
    }
}
