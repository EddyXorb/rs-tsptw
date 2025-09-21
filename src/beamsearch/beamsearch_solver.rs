use super::beamsearch_collection::BeamsearchCollection;
pub use super::beamsearch_collection::BeamsearchNode;
use super::parent_tree::ParentTreeNode;
use rayon::prelude::*;

pub type Node<T> = ParentTreeNode<T>;

pub struct Params {
    pub beam_width: usize,
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
    T: BeamsearchNode,
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
    T: BeamsearchNode,
    F: Fn(&Node<T>) -> Vec<T>,
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
            let nr_expanded = self.expand();

            let similars_removed = self.coll.remove_similars(&self.is_similar);
            all_similars_removed += all_similars_removed;

            if nr_expanded == 0 {
                return self.create_result(all_expansions, all_similars_removed);
            }

            all_expansions += nr_expanded;

            self.coll.keep_best(self.params.beam_width);

            println!(
                "Coll.-size: {}. Expanded {} and removed {} similars",
                self.coll.len(),
                nr_expanded,
                similars_removed
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

        for node in &old_coll {
            let children = (self.expander)(node);

            nr_expanded += children.len();

            for child in children {
                self.coll.add(node.new_child(child));
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
            Params { beam_width: 2 },
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
            Params { beam_width: 4 },
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
            Params { beam_width: 1000 },
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
            Params { beam_width: 2 },
        )
        .solve();
        assert!(result.best.is_none());
    }
}
