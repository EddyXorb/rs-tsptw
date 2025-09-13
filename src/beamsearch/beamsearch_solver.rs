use crate::beamsearch::parent_tree::ParentTreeNode;

use super::beamsearch_collection::{BeamsearchCollection, BeamsearchNode};

pub struct Params {
    beam_width: usize,
}

pub struct BeamsearchSolver<T>
where
    T: BeamsearchNode,
{
    coll: BeamsearchCollection<T>,
    params: Params,
}

impl<T> BeamsearchSolver<T>
where
    T: BeamsearchNode,
{
    pub fn new(start_nodes: Vec<T>, params: Params) -> Self {
        let mut coll = BeamsearchCollection::default();
        for node in start_nodes {
            coll.add(ParentTreeNode::new_root(node));
        }

        Self { coll, params }
    }

    pub fn solve<F>(mut self, expander: F) -> Option<ParentTreeNode<T>>
    where
        F: Fn(&T) -> Vec<T>,
    {
        loop {
            let old_coll = std::mem::take(&mut self.coll);

            let mut nr_expanded = 0;
            for node in &old_coll {
                let children = expander(node.data());
                nr_expanded += children.len();

                for child in children {
                    self.coll.add(node.new_child(child));
                }
            }

            if nr_expanded == 0 {
                let best = old_coll.get_best()?;
                return Some(best.clone());
            }

            self.coll.keep_best(self.params.beam_width);
        }
    }
}
