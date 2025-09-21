use super::beamsearch_collection::BeamsearchCollection;
pub use super::beamsearch_collection::BeamsearchNode;
use super::parent_tree::ParentTreeNode;
use rayon::prelude::*;
use std::time::Instant;

pub type Node<T> = ParentTreeNode<T>;

pub struct Params {
    pub beam_width: usize,
    pub prune_similars: bool,
}

pub struct SolverResult<T>
where
    T: BeamsearchNode,
{
    pub best: Option<Node<T>>,
    pub nr_expansions: usize,
}

pub fn is_never_similar<T>(_a: &Node<T>, _b: &Node<T>) -> bool {
    false
}

pub struct BeamsearchSolver<T, F, S, V>
where
    T: BeamsearchNode + Send + Sync,
    F: Fn(&Node<T>) -> Vec<T>,
    S: Fn(&Node<T>, &Node<T>) -> bool,
    V: Fn(&Node<T>) -> bool,
{
    coll: BeamsearchCollection<T>,
    expander: F,
    is_similar: S,
    is_valid_solution: V,
    params: Params,
}

impl<T, F, S, V> BeamsearchSolver<T, F, S, V>
where
    T: BeamsearchNode + Send + Sync,
    F: Fn(&Node<T>) -> Vec<T> + Send + Sync,
    S: Fn(&Node<T>, &Node<T>) -> bool,
    V: Fn(&Node<T>) -> bool,
{
    pub fn new(
        start_nodes: Vec<T>,
        expander: F,
        is_similar: S,
        is_valid_solution: V,
        params: Params,
    ) -> Self {
        let mut coll = BeamsearchCollection::default();
        for node in start_nodes {
            coll.add(Node::new_root(node));
        }

        Self {
            coll,
            expander,
            is_similar,
            is_valid_solution,
            params,
        }
    }

    pub fn solve(mut self) -> SolverResult<T> {
        let mut all_expansions: usize = 0;
        let mut all_similars_removed: usize = 0;

        loop {
            let iteration_start = Instant::now();

            let expand_start = Instant::now();
            let nr_expanded = self.expand();
            let expand_duration = expand_start.elapsed();

            let similar_start = Instant::now();
            let similars_removed = if self.params.prune_similars {
                self.coll.remove_similars(&self.is_similar)
            } else {
                0
            };
            let similar_duration = similar_start.elapsed();
            all_similars_removed += similars_removed;

            if nr_expanded == 0 {
                return self.create_result(all_expansions, all_similars_removed);
            }

            all_expansions += nr_expanded;

            let keep_best_start = Instant::now();
            self.coll.keep_best(self.params.beam_width);
            let keep_best_duration = keep_best_start.elapsed();

            let iteration_duration = iteration_start.elapsed();

            println!(
                "Coll.-size: {}. Expanded {} (in {:.0}ms) and removed {} similars (in {:.0}ms), shrinked (in {:.0}ms), total time {:.0}ms",
                self.coll.len(),
                nr_expanded,
                expand_duration.as_secs_f64() * 1000.0,
                similars_removed,
                similar_duration.as_secs_f64() * 1000.0,
                keep_best_duration.as_secs_f64() * 1000.0,
                iteration_duration.as_secs_f64() * 1000.0
            );
        }
    }

    fn create_result(self, all_expansions: usize, all_similars_removed: usize) -> SolverResult<T> {
        println!(
            "Finished. Expanded {} and removed {} similars.",
            all_expansions, all_similars_removed
        );

        let best = self.coll.get_best().cloned();
        if best.is_some() && (&self.is_valid_solution)(&best.unwrap()) {
            return SolverResult {
                best: self.coll.get_best().cloned(),
                nr_expansions: all_expansions,
            };
        }
        SolverResult {
            best: None,
            nr_expansions: all_expansions,
        }
    }

    fn expand(&mut self) -> usize {
        let old_coll = std::mem::take(&mut self.coll);

        let mut nr_expanded = 0;

        let results: Vec<_> = old_coll
            .par_iter()
            .map(|node| {
                let children = (self.expander)(node);
                children
                    .into_iter()
                    .map(|child| node.new_child(child))
                    .collect::<Vec<_>>()
            })
            .collect();

        for expanded_children in results {
            nr_expanded += expanded_children.len();
            for child in expanded_children {
                self.coll.add(child)
            }
        }

        if nr_expanded == 0 {
            self.coll = old_coll;
        }

        nr_expanded
    }
}

#[cfg(test)]
mod tests {

    use crate::beamsearch::beamsearch_solver::{BeamsearchSolver, Node, Params, is_never_similar};

    use super::super::mocks::TestNode;

    fn base_expander(n: &Node<TestNode>) -> Vec<TestNode> {
        if n.data().dummy_level < 2.0 {
            return vec![TestNode {
                dummy_fitness: n.data().dummy_fitness + 1.0,
                dummy_level: n.data().dummy_level + 1.0,
            }];
        }
        return vec![];
    }

    fn bifurcate_expander<const LAST_LEVEL: i32>(n: &Node<TestNode>) -> Vec<TestNode> {
        if n.data().dummy_level < LAST_LEVEL as f64 {
            return vec![
                TestNode {
                    dummy_fitness: n.data().dummy_fitness + 1.0,
                    dummy_level: n.data().dummy_level + 1.0,
                },
                TestNode {
                    dummy_fitness: n.data().dummy_fitness + 1.0,
                    dummy_level: n.data().dummy_level + 1.0,
                },
            ];
        }
        return vec![];
    }

    #[test]
    fn test_simple_solve() {
        let result = BeamsearchSolver::new(
            vec![TestNode::default()],
            base_expander,
            is_never_similar,
            |_n| true,
            Params {
                beam_width: 2,
                prune_similars: true,
            },
        )
        .solve()
        .best
        .unwrap();

        assert!(&result.data().dummy_fitness == &2.0);
        assert!(&result.data().dummy_level == &2.0);
    }

    #[test]
    fn test_big_solve() {
        let result = BeamsearchSolver::new(
            vec![TestNode::default()],
            bifurcate_expander::<10>,
            is_never_similar,
            |_n| true,
            Params {
                beam_width: 4,
                prune_similars: true,
            },
        )
        .solve();

        let best = result.best.unwrap();

        assert_eq!(result.nr_expansions, 2 + 4 + 8 * 8);
        assert!(&best.data().dummy_fitness == &10.0);
        assert!(&best.data().dummy_level == &10.0);
    }

    #[test]
    fn test_is_similar_effectively_prunes() {
        let result = BeamsearchSolver::new(
            vec![TestNode::default()],
            bifurcate_expander::<10>,
            |x, y| x.data() == y.data(),
            |_n| true,
            Params {
                beam_width: 1000,
                prune_similars: true,
            },
        )
        .solve();

        assert_eq!(result.nr_expansions, 10 * 2);
    }

    #[test]
    fn test_is_valid_solution_checks_invalid_solution() {
        let result = BeamsearchSolver::new(
            vec![TestNode::default()],
            base_expander,
            is_never_similar,
            |_n| false,
            Params {
                beam_width: 2,
                prune_similars: true,
            },
        )
        .solve();
        assert!(result.best.is_none());
    }
}
