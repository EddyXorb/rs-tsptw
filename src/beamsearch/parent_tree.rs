use std::sync::Arc;

pub struct ParentTreeNode<T> {
    parent: Option<Arc<ParentTreeNode<T>>>,
    data: T,
}

impl<T> ParentTreeNode<T> {
    pub fn new_root(data: T) -> Self {
        Self { parent: None, data }
    }

    pub fn new_child(parent: Arc<ParentTreeNode<T>>, data: T) -> Self {
        Self {
            parent: Some(parent),
            data,
        }
    }

    pub fn data(&self) -> &T {
        &self.data
    }

    pub fn is_root(&self) -> bool {
        self.parent.is_none()
    }

    pub fn parent(&self) -> Option<&Arc<Self>> {
        self.parent.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_root() {
        let root = ParentTreeNode::new_root(1.0);

        assert_eq!(*root.data(), 1.0);
    }

    #[test]
    fn create_child() {
        let root = Arc::new(ParentTreeNode::new_root(1.0));
        let child = ParentTreeNode::new_child(root.clone(), 2.0);

        assert!(Arc::ptr_eq(child.parent().unwrap(), &root));
    }
}
