use super::parent_tree::ParentTreeNode;

pub trait BeamsearchNode {
    fn fitness(&self) -> f64;
    fn level(&self) -> f64;
}

pub struct BeamsearchCollection<T>
where
    T: BeamsearchNode,
{
    nodes: Vec<ParentTreeNode<T>>,
    sorted: bool,
}

impl<T> Default for BeamsearchCollection<T>
where
    T: BeamsearchNode,
{
    fn default() -> Self {
        Self {
            nodes: Default::default(),
            sorted: true,
        }
    }
}

impl<T> BeamsearchCollection<T>
where
    T: BeamsearchNode,
{
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = &ParentTreeNode<T>> {
        self.nodes.iter()
    }

    pub fn add(&mut self, node: ParentTreeNode<T>) {
        self.nodes.push(node);
        self.sorted = false;
    }

    pub fn sort(&mut self) {
        if self.sorted {
            return;
        }
        self.nodes
            .sort_by(|a, b| a.data().fitness().total_cmp(&b.data().fitness()));

        self.sorted = true;
    }
    pub fn keep_best(&mut self, target_size: usize) -> usize {
        if target_size >= self.len() {
            return 0;
        }

        self.sort();

        let deleted = self.len() - target_size;
        self.nodes.truncate(target_size);
        deleted
    }

    pub fn get_best(&self) -> Option<&ParentTreeNode<T>> {
        if self.sorted {
            return self.nodes.first();
        }
        self.nodes
            .iter()
            .min_by(|a, b| a.data().fitness().total_cmp(&b.data().fitness()))
    }
}

impl<T> IntoIterator for BeamsearchCollection<T>
where
    T: BeamsearchNode,
{
    type Item = ParentTreeNode<T>;
    type IntoIter = std::vec::IntoIter<ParentTreeNode<T>>;

    fn into_iter(self) -> Self::IntoIter {
        self.nodes.into_iter()
    }
}

impl<'a, T> IntoIterator for &'a BeamsearchCollection<T>
where
    T: BeamsearchNode,
{
    type Item = &'a ParentTreeNode<T>;
    type IntoIter = std::slice::Iter<'a, ParentTreeNode<T>>;

    fn into_iter(self) -> Self::IntoIter {
        self.nodes.iter()
    }
}

#[cfg(test)]
mod tests {

    use std::iter::zip;

    use super::*;
    use rand::{Rng, SeedableRng, rngs::StdRng};
    struct TestNode {
        dummy_fitness: f64,
        dummy_level: f64,
    }

    impl Default for TestNode {
        fn default() -> Self {
            Self {
                dummy_fitness: 0.0,
                dummy_level: 0.0,
            }
        }
    }

    impl BeamsearchNode for TestNode {
        fn fitness(&self) -> f64 {
            self.dummy_fitness
        }

        fn level(&self) -> f64 {
            self.dummy_level
        }
    }

    fn create_test_collection(size: usize) -> BeamsearchCollection<TestNode> {
        let mut rng = StdRng::seed_from_u64(42);

        let root = ParentTreeNode::new_root(TestNode {
            dummy_fitness: 0.0,
            dummy_level: 0.0,
        });
        let mut coll = BeamsearchCollection::<TestNode>::default();

        for _ in 0..size {
            coll.add(root.new_child(TestNode {
                dummy_fitness: rng.random_range(0.0..100.0),
                dummy_level: rng.random_range(0.0..100.0),
            }));
        }

        coll
    }

    #[test]
    fn test_into_iter() {
        let coll = create_test_collection(1);

        for _node in &coll {}
    }

    #[test]
    fn test_iter() {
        let coll = create_test_collection(1);

        for _node in coll.iter() {}
    }

    #[test]
    fn test_len() {
        for i in 0..4 {
            let coll = create_test_collection(i);

            assert_eq!(coll.len(), i)
        }
    }

    #[test]
    fn test_sort() {
        let mut coll = create_test_collection(10);

        coll.sort();

        let mut last_fitness = 0.0; // works because we generate only fitnesses between 0..100
        for node in &coll {
            assert!(node.data().fitness() >= last_fitness);
            last_fitness = node.data().fitness();
            println!("{}", last_fitness)
        }
    }

    #[test]
    fn test_keep_best() {
        let mut coll = create_test_collection(10);

        let mut fitnesses: Vec<f64> = vec![];

        for node in &coll {
            fitnesses.push(node.data().fitness());
        }

        fitnesses.sort_by(|a, b| a.total_cmp(b));
        fitnesses.truncate(5);

        coll.keep_best(5);

        for (node, expected_fitness) in zip(&coll, fitnesses) {
            assert_eq!(node.data().fitness(), expected_fitness);
        }
    }

    #[test]
    fn test_get_best() {
        let coll = create_test_collection(10);

        let best = coll.get_best().unwrap();

        for node in &coll {
            assert!(best.data().fitness() <= node.data().fitness());
        }
    }
}
