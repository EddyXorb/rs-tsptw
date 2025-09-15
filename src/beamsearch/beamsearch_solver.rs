use super::beamsearch_collection::{BeamsearchCollection, BeamsearchNode};
use super::parent_tree::ParentTreeNode;
use std::marker::PhantomData;

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

pub fn is_never_similar<T>(a: &Node<T>, b: &Node<T>) -> bool {
    false
}

pub struct BeamsearchSolver<T, F, S>
where
    T: BeamsearchNode,
    F: Fn(&Node<T>) -> Vec<T>,
    S: Fn(&Node<T>, &Node<T>) -> bool,
{
    coll: BeamsearchCollection<T>,
    expander: F,
    is_similar: S,
    params: Params,
}

impl<T, F, S> BeamsearchSolver<T, F, S>
where
    T: BeamsearchNode,
    F: Fn(&Node<T>) -> Vec<T>,
    S: Fn(&Node<T>, &Node<T>) -> bool,
{
    pub fn new(start_nodes: Vec<T>, expander: F, is_similar: S, params: Params) -> Self {
        let mut coll = BeamsearchCollection::default();
        for node in start_nodes {
            coll.add(Node::new_root(node));
        }

        Self {
            coll,
            expander,
            is_similar,
            params,
        }
    }

    pub fn solve(mut self) -> SolverResult<T> {
        let mut all_expansions: usize = 0;
        loop {
            let nr_expanded = self.expand();

            if nr_expanded == 0 {
                return SolverResult {
                    best: self.coll.get_best().cloned(),
                    nr_expansions: all_expansions,
                };
            }

            all_expansions += nr_expanded;

            self.coll.keep_best(self.params.beam_width);
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

    fn bifurcate_expander(n: &Node<TestNode>) -> Vec<TestNode> {
        if n.data().dummy_level < 10.0 {
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
            bifurcate_expander,
            is_never_similar,
            Params { beam_width: 4 },
        )
        .solve();

        let best = result.best.unwrap();

        assert_eq!(result.nr_expansions, 2 + 4 + 8 * 8);
        assert!(&best.data().dummy_fitness == &10.0);
        assert!(&best.data().dummy_level == &10.0);
    }

}
