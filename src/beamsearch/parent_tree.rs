use std::sync::Arc;

struct InnerParentTreeNode<T> {
    parent: Option<ParentTreeNode<T>>,
    data: T,
}

pub struct ParentTreeNode<T> {
    inner: Arc<InnerParentTreeNode<T>>,
}

impl<T> ParentTreeNode<T> {
    pub fn new_root(data: T) -> Self {
        Self {
            inner: Arc::new(InnerParentTreeNode { parent: None, data }),
        }
    }

    pub fn new_child(&self, data: T) -> Self {
        let inner = Arc::new(InnerParentTreeNode {
            parent: Some(ParentTreeNode {
                inner: Arc::clone(&self.inner),
            }),
            data,
        });

        Self { inner }
    }

    pub fn data(&self) -> &T {
        &self.inner.data
    }

    pub fn is_root(&self) -> bool {
        self.inner.parent.is_none()
    }

    pub fn parent(&self) -> Option<&ParentTreeNode<T>> {
        self.inner.parent.as_ref()
    }

    pub fn parents(&self) -> impl Iterator<Item = &ParentTreeNode<T>> {
        std::iter::successors(self.parent(), |node| node.parent())
    }

    pub fn ancestors(&self) -> impl Iterator<Item = &ParentTreeNode<T>> {
        std::iter::successors(Some(self), |node| node.parent())
    }
}

impl<T> Clone for ParentTreeNode<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use super::*;

    #[test]
    fn create_root() {
        let root = ParentTreeNode::new_root(1.0);

        assert_eq!(*root.data(), 1.0);
    }

    #[test]
    fn create_child() {
        let root = ParentTreeNode::new_root(1.0);
        let child = root.new_child(2.0);

        assert!(Arc::ptr_eq(&child.parent().unwrap().inner, &root.inner));
    }

    #[test]
    fn parents() {
        let root = ParentTreeNode::new_root(1.0);
        let child = root.new_child(2.0);

        let parents: Vec<&ParentTreeNode<f64>> = child.parents().collect();

        assert_eq!(parents.len(), 1);
        assert_eq!(*parents[0].data(), 1.0);
    }

    #[test]
    fn ancestors() {
        let root = ParentTreeNode::new_root(1.0);
        let child = root.new_child(2.0);

        let ancestors: Vec<f64> = child.ancestors().map(|n| *n.data()).collect();

        assert_eq!(ancestors.len(), 2);
        assert_eq!(ancestors, vec![2.0, 1.0]);
    }

    #[test]
    fn bigger_tree_works() {
        let root = ParentTreeNode::new_root(1);

        let mut child = root;
        for i in 2..=10 {
            let _child = child.new_child(i * 100);
            child = child.new_child(i);
        }

        let ancestors: Vec<i32> = child.ancestors().map(|n| *n.data()).collect();

        assert_eq!(ancestors.len(), 10);
        assert_eq!(ancestors, (1..=10).rev().collect::<Vec<i32>>());
    }

    #[test]
    fn unused_nodes_are_effectively_deleted() {
        //Acts as counter for nodes
        let root_counter = Rc::new(0);
        let child_counter = Rc::new(0);

        let root = ParentTreeNode::new_root(root_counter.clone());

        {
            let mut child = root;

            for _i in 1..10 {
                child = child.new_child(child_counter.clone());
            }

            assert_eq!(Rc::strong_count(&root_counter), 2);
            assert_eq!(Rc::strong_count(&child_counter), 10);
        }

        assert_eq!(Rc::strong_count(&root_counter), 1);
        assert_eq!(Rc::strong_count(&child_counter), 1);
    }

    #[test]
    fn is_root() {
        let root = ParentTreeNode::new_root(1.0);
        let child = root.new_child(2.0);

        assert!(root.is_root());
        assert!(!child.is_root());
    }
}
