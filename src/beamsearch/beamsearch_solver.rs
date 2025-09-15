use super::beamsearch_collection::{BeamsearchCollection, BeamsearchNode};
use super::parent_tree::ParentTreeNode;

pub type NodeType<T> = ParentTreeNode<T>;

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
            coll.add(NodeType::new_root(node));
        }

        Self { coll, params }
    }

    pub fn solve<F>(mut self, expander: F) -> Option<NodeType<T>>
    where
        F: Fn(&NodeType<T>) -> Vec<T>,
    {
        loop {
            let old_coll = std::mem::take(&mut self.coll);

            let mut nr_expanded = 0;
            for node in &old_coll {
                let children = expander(node);
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
