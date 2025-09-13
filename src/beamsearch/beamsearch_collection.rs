use super::parent_tree::ParentTreeNode;

trait BeamsearchNode {
    fn fitness() -> f64;
    fn level() -> f64;
}

pub struct BeamsearchCollection<T>
where
    T: BeamsearchNode,
{
    nodes: Vec<ParentTreeNode<T>>,
}
